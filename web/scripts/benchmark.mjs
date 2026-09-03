import { spawn, spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { performance } from "node:perf_hooks";
import { chromium } from "playwright";

const targets = ["vanilla", "vue"];
const temporaryRoot = mkdtempSync(join(tmpdir(), "agentide-renderer-benchmark-"));
const builds = {};

try {
  for (const target of targets) {
    const output = join(temporaryRoot, target);
    const started = performance.now();
    const result = spawnSync("npm", ["exec", "vite", "build", "--", "--outDir", output], {
      cwd: process.cwd(),
      encoding: "utf8",
      env: { ...process.env, AGENTIDE_RENDERER_TARGET: target },
    });
    if (result.status !== 0) throw new Error(result.stderr || result.stdout);
    const entry = result.stdout.match(/assets\/(?:vanilla|vue)-[^ ]+\.js/g)?.at(-1);
    if (!entry) throw new Error(`missing ${target} benchmark bundle`);
    builds[target] = {
      milliseconds: performance.now() - started,
      javascript_bytes: statSync(join(output, entry)).size,
    };
  }

  const build = spawnSync("npm", ["run", "build"], {
    cwd: process.cwd(),
    encoding: "utf8",
  });
  if (build.status !== 0) throw new Error(build.stderr || build.stdout);
  const preview = spawn(
    "npm",
    ["exec", "vite", "preview", "--", "--host", "127.0.0.1", "--port", "4187"],
    { cwd: process.cwd(), stdio: "ignore" },
  );
  try {
    let ready = false;
    for (let attempt = 0; attempt < 100; attempt += 1) {
      try {
        const response = await fetch("http://127.0.0.1:4187/");
        if (response.ok) {
          ready = true;
          break;
        }
      } catch {
        // The bounded loop below retries while Vite starts.
      }
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
    if (!ready) throw new Error("renderer benchmark preview did not start");

    const browser = await chromium.launch({ headless: true });
    const runtime = {};
    try {
      for (const target of targets) {
        const samples = [];
        for (let iteration = 0; iteration < 10; iteration += 1) {
          const page = await browser.newPage();
          const started = performance.now();
          await page.goto(`http://127.0.0.1:4187/renderers/${target}/?fixture=1`, {
            waitUntil: "networkidle",
          });
          const navigation = performance.now() - started;
          const observed = await page.evaluate(() => {
            const mount = performance.getEntriesByName("agentide.renderer.mount").at(-1)?.duration ?? 0;
            const update = window.__agentideRendererBenchmark?.update(50) ?? 0;
            const memory = "memory" in performance
              ? /** @type {{usedJSHeapSize: number}} */ (performance.memory).usedJSHeapSize
              : null;
            return { mount, update, memory };
          });
          samples.push({ navigation, ...observed });
          await page.close();
        }
        samples.sort((left, right) => left.navigation - right.navigation);
        runtime[target] = samples[Math.floor(samples.length / 2)];
      }
    } finally {
      await browser.close();
    }
    process.stdout.write(
      `${JSON.stringify({ format: "agentide.renderer-benchmark/1", builds, runtime }, null, 2)}\n`,
    );
  } finally {
    preview.kill("SIGTERM");
  }
} finally {
  rmSync(temporaryRoot, { recursive: true, force: true });
}
