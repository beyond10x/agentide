import { describe, expect, it, vi } from "vitest";
import {
  WorkbenchController,
  WorkbenchRefusal,
  type WorkbenchHostPort,
} from "../src/controller/workbench";
import {
  rendererActionFormat,
  rendererEventFormat,
  type ChatMessage,
} from "../src/renderer/protocol";
import { renderMarkdown } from "../src/renderer/markdown";

function port(): WorkbenchHostPort {
  return {
    snapshot: vi.fn(async () => ({
      session: { id: "session-1", objective: "Ship the workspace", status: "active" as const, cursor: 1 },
      panes: [
        { id: "editor-readme", kind: "editor" as const, title: "README.md", path: "README.md" },
        { id: "chat", kind: "chat" as const, title: "Agent" },
        { id: "changes", kind: "diff" as const, title: "Changes" },
      ],
      focused_pane: "chat",
      open_files: ["README.md"],
      pending_approvals: [],
      context_pins: [],
      grants: [],
      activity: [],
    })),
    tree: vi.fn(async () => ({
      kind: "tree" as const, root: "", entries: [{ path: "README.md", name: "README.md", kind: "file" as const }],
    })),
    openFile: vi.fn(async (path) => ({
      path, language: "markdown", content: "# AgentIDE", version: "sha256:one", read_only: false,
    })),
    saveFile: vi.fn(async (path, content) => ({
      path, language: "markdown", content, version: "sha256:two", read_only: false,
    })),
    focusPane: vi.fn(async () => undefined),
    closePane: vi.fn(async () => undefined),
    changes: vi.fn(async () => ({ baseline_commit: "a".repeat(40), changes: [], truncated: false })),
    submitPrompt: vi.fn(async (_content, onDelta) => {
      onDelta("**live**");
      return {
        id: "answer-1", role: "assistant" as const, markdown: "**live**", state: "complete" as const,
        created_at: "2026-09-04T20:00:00Z",
      };
    }),
    approve: vi.fn(async () => undefined),
    deny: vi.fn(async () => undefined),
    pinContext: vi.fn(async () => undefined),
    removeContextPin: vi.fn(async () => undefined),
    openTerminal: vi.fn(async () => "terminal-1"),
    terminalInput: vi.fn(async () => undefined),
    terminalResize: vi.fn(async () => undefined),
  };
}

describe("WorkbenchController", () => {
  it("resolves files lazily and retains typed projections across host snapshots", async () => {
    const host = port();
    const controller = new WorkbenchController(host);
    await controller.start();
    await controller.dispatch({ format: rendererActionFormat, kind: "open_file", path: "README.md" });
    expect(host.openFile).toHaveBeenCalledOnce();
    expect(controller.frame()?.workbench.projections["editor-readme"]).toMatchObject({
      kind: "editor", document: { content: "# AgentIDE", dirty: false },
    });
  });

  it("keeps unsaved bytes browser-local across refreshes and save refusals", async () => {
    const host = port();
    host.saveFile = vi.fn(async () => {
      throw new WorkbenchRefusal("workspace.file_conflict", "The file changed remotely.");
    });
    const controller = new WorkbenchController(host);
    await controller.start();
    await controller.dispatch({ format: rendererActionFormat, kind: "open_file", path: "README.md" });
    await controller.dispatch({
      format: rendererActionFormat,
      kind: "edit_file",
      path: "README.md",
      content: "local draft",
      version: "sha256:one",
    });
    await controller.refresh();
    await controller.dispatch({
      format: rendererActionFormat,
      kind: "save_file",
      path: "README.md",
      content: "local draft",
      version: "sha256:one",
    });
    expect(controller.frame()?.workbench.projections["editor-readme"]).toMatchObject({
      kind: "editor",
      document: { content: "local draft", dirty: true },
    });
    expect(controller.frame()?.notice).toBe("The file changed remotely.");
  });

  it("publishes assistant markdown deltas before the completed message", async () => {
    const controller = new WorkbenchController(port());
    const frames: string[] = [];
    await controller.start();
    controller.subscribe((frame) => {
      const projection = frame.workbench.projections.chat;
      if (projection?.kind === "chat") frames.push(projection.messages.at(-1)?.markdown ?? "");
    });
    await controller.dispatch({ format: rendererActionFormat, kind: "submit_prompt", content: "What changed?" });
    expect(frames).toContain("**live**");
    expect(controller.frame()?.workbench.projections.chat).toMatchObject({
      kind: "chat", messages: [
        { role: "user", markdown: "What changed?", state: "complete" },
        { id: "answer-1", state: "complete" },
      ],
    });
  });

  it("marks a partial assistant message failed when the host stream refuses", async () => {
    const host = port();
    host.submitPrompt = vi.fn(async (_content, onDelta) => {
      onDelta("partial");
      throw new WorkbenchRefusal("agent.turn_refused", "The model grant expired.", true);
    });
    const controller = new WorkbenchController(host);
    await controller.start();
    await controller.dispatch({
      format: rendererActionFormat,
      kind: "submit_prompt",
      content: "continue",
    });
    expect(controller.frame()?.workbench.projections.chat).toMatchObject({
      kind: "chat",
      messages: [
        { role: "user", markdown: "continue", state: "complete" },
        { role: "assistant", markdown: "partial", state: "failed" },
      ],
    });
    expect(controller.frame()?.notice).toBe("The model grant expired.");
  });

  it("delegates durable pane transitions to the host", async () => {
    const host = port();
    const controller = new WorkbenchController(host);
    await controller.start();
    await controller.dispatch({ format: rendererActionFormat, kind: "focus_pane", pane_id: "editor-readme" });
    await controller.dispatch({ format: rendererActionFormat, kind: "close_pane", pane_id: "editor-readme" });
    expect(host.focusPane).toHaveBeenCalledWith("editor-readme", expect.any(AbortSignal));
    expect(host.closePane).toHaveBeenCalledWith("editor-readme", expect.any(AbortSignal));
  });

  it("applies ordered external deltas once without giving the renderer a host port", async () => {
    const host = port();
    let finish: ((message: ChatMessage) => void) | undefined;
    host.submitPrompt = vi.fn(() => new Promise<ChatMessage>((resolve) => { finish = resolve; }));
    const controller = new WorkbenchController(host);
    await controller.start();
    const turn = controller.dispatch({ format: rendererActionFormat, kind: "submit_prompt", content: "stream" });
    await vi.waitFor(() => {
      const projection = controller.frame()?.workbench.projections.chat;
      expect(projection?.kind === "chat" ? projection.messages : []).toHaveLength(2);
    });
    const projection = controller.frame()?.workbench.projections.chat;
    const messageId = projection?.kind === "chat" ? projection.messages[1]?.id : undefined;
    expect(messageId).toBeDefined();
    controller.deliver({
      format: rendererEventFormat,
      kind: "assistant_delta",
      message_id: messageId!,
      sequence: 1,
      markdown_delta: " more",
    });
    controller.deliver({
      format: rendererEventFormat,
      kind: "assistant_delta",
      message_id: messageId!,
      sequence: 1,
      markdown_delta: " duplicate",
    });
    expect(controller.frame()?.workbench.projections.chat).toMatchObject({
      messages: [{ role: "user" }, { markdown: " more" }],
    });
    finish?.({
      id: "answer-1",
      role: "assistant",
      markdown: " more",
      state: "complete",
      created_at: "2026-09-04T20:00:00Z",
    });
    await turn;
  });
});

describe("renderMarkdown", () => {
  it("renders a Markdown subset and escapes active content", () => {
    const html = renderMarkdown("# Result\n\n**ready** <script>alert(1)</script> [docs](javascript:bad)");
    expect(html).toContain("<h1>Result</h1>");
    expect(html).toContain("<strong>ready</strong>");
    expect(html).toContain("&lt;script&gt;");
    expect(html).not.toContain('href="javascript:');
  });
});
