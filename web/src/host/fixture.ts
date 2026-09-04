import { rendererFrameFormat, type RendererFrame, type RendererTarget } from "../renderer/protocol";

declare global {
  interface Window {
    __agentideRendererBenchmark?: {
      update(iterations: number): number;
      destroy(): void;
    };
  }
}

function frame(cursor = 1): RendererFrame {
  const files = Array.from(
    { length: 200 },
    (_, index) => `src/module-${String(index).padStart(3, "0")}.rs`,
  );
  return {
    format: rendererFrameFormat,
    session: {
      id: "session-renderer-benchmark",
      objective: "Compare transport-neutral renderer targets",
      status: "active",
      cursor,
    },
    workbench: {
      panes: files.slice(0, 24).map((path, index) => ({
        id: `pane-${index}`,
        kind: index % 5 === 0 ? "diff" : "editor",
        title: path.split("/").at(-1) ?? path,
        path,
      })),
      focused_pane: `pane-${cursor % 24}`,
      open_files: files,
      projections: {
        [`pane-${cursor % 24}`]: { kind: "empty", message: "renderer benchmark observation" },
      },
    },
    pending_approvals: Array.from({ length: 8 }, (_, index) => ({
      digest: String(index).padStart(64, "a"),
      intent: index % 2 ? "code_write" : "process_run",
      risk: "workspace_write",
      approval_required: true,
    })),
    activity: Array.from({ length: 100 }, (_, index) => ({
      sequence: index + 1,
      at: "2026-09-03T12:00:00Z",
      kind: "intent.completed",
      intent: index % 3 ? "code_read" : "diff_show",
    })),
    context_pins: [],
    grants: [],
  };
}

export function mountFixtureHost(target: RendererTarget, container: HTMLElement): void {
  const started = performance.now();
  const handle = target.mount(container, { frame: frame(), dispatch: () => undefined });
  const mounted = performance.now() - started;
  performance.measure("agentide.renderer.mount", { start: started, duration: mounted });
  window.__agentideRendererBenchmark = {
    update(iterations) {
      const updateStarted = performance.now();
      for (let index = 0; index < iterations; index += 1) handle.update(frame(index + 2));
      return (performance.now() - updateStarted) / iterations;
    },
    destroy() {
      handle.destroy();
    },
  };
}
