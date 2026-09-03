import { nextTick } from "vue";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  rendererActionFormat,
  rendererFrameFormat,
  rendererProtocolFormat,
  type RendererAction,
  type RendererFrame,
  type RendererTarget,
} from "../src/renderer/protocol";
import { vanillaRenderer } from "../src/renderer/vanilla";
import { vueRenderer } from "../src/renderer/vue";

const targets: RendererTarget[] = [vanillaRenderer, vueRenderer];

function fixture(objective = "Refactor the renderer boundary"): RendererFrame {
  return {
    format: rendererFrameFormat,
    session: {
      id: "session-renderer-conformance",
      objective,
      status: "active",
      cursor: 42,
    },
    workbench: {
      panes: [{ id: "pane-readme", kind: "editor", title: "README.md", path: "README.md" }],
      focused_pane: "pane-readme",
      open_files: ["README.md"],
    },
    pending_approvals: [
      {
        digest: "a".repeat(64),
        intent: "code_write",
        risk: "workspace_write",
        approval_required: true,
      },
    ],
    activity: [
      {
        sequence: 42,
        at: "2026-09-03T12:00:00Z",
        kind: "intent.completed",
        intent: "code_read",
      },
    ],
    observation: { path: "README.md", content: "# AgentIDE" },
  };
}

describe.each(targets)("$manifest.id renderer target", (target) => {
  it("declares the released renderer protocol", () => {
    expect(target.manifest).toMatchObject({
      format: rendererProtocolFormat,
      frame_format: rendererFrameFormat,
      action_format: rendererActionFormat,
    });
  });

  it("renders, updates, emits typed actions, and tears down", async () => {
    const container = document.createElement("div");
    document.body.append(container);
    const actions: RendererAction[] = [];
    const handle = target.mount(container, { frame: fixture(), dispatch: (value) => actions.push(value) });
    await nextTick();

    expect(container.textContent).toContain("Refactor the renderer boundary");
    expect(container.textContent).toContain("README.md");
    expect(container.textContent).toContain("code_write");
    expect(container.querySelector("main.shell")).not.toBeNull();
    expect(container.querySelector("aside.explorer")).not.toBeNull();
    expect(container.querySelector("aside.context")).not.toBeNull();

    const file = Array.from(container.querySelectorAll("button")).find((candidate) =>
      candidate.textContent?.includes("README.md"),
    );
    expect(file).toBeDefined();
    file?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(actions.at(-1)).toEqual({
      format: rendererActionFormat,
      kind: "open_file",
      path: "README.md",
    });

    handle.update(fixture("Updated objective"));
    await nextTick();
    expect(container.textContent).toContain("Updated objective");
    handle.deliver({
      format: "agentide.renderer-event/1",
      kind: "notice",
      message: "Host observation",
    });
    await nextTick();
    expect(container.textContent).toContain("Host observation");

    handle.destroy();
    await nextTick();
    expect(container.childElementCount).toBe(0);
    container.remove();
  });
});

it("keeps renderer targets free of transport and persistence APIs", async () => {
  const { readFile } = await import("node:fs/promises");
  const { resolve } = await import("node:path");
  const sources = await Promise.all([
    readFile(resolve(process.cwd(), "src/renderer/vanilla.ts"), "utf8"),
    readFile(resolve(process.cwd(), "src/renderer/vue.ts"), "utf8"),
  ]);
  const forbidden = ["fetch(", "XMLHttpRequest", "WebSocket", "EventSource", "localStorage", "sessionStorage"];
  for (const source of sources) {
    for (const token of forbidden) expect(source).not.toContain(token);
  }
});

afterEach(() => {
  vi.restoreAllMocks();
  document.body.replaceChildren();
});
