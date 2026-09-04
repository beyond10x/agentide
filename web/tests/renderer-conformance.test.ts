import { createApp, h, nextTick } from "vue";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  rendererActionFormat,
  rendererEventFormat,
  rendererFrameFormat,
  rendererProtocolFormat,
  type RendererAction,
  type RendererFrame,
  type RendererTarget,
} from "../src/renderer/protocol";
import { createVanillaRenderer, vanillaRenderer } from "../src/renderer/vanilla";
import { AgentIdeVueWorkbench, createVueRenderer, vueRenderer } from "../src/renderer/vue";

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
      projections: {
        "pane-readme": {
          kind: "editor",
          document: {
            path: "README.md",
            language: "markdown",
            content: "# AgentIDE",
            version: "fixture",
            read_only: false,
            dirty: false,
          },
        },
      },
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
    context_pins: [
      { id: "pin-readme", label: "README context", source: "README.md:1-20" },
    ],
    grants: [{ id: "grant-read", capability: "workspace.read", state: "active" }],
  };
}

describe.each(targets)("$manifest.id renderer target", (target) => {
  it("declares the released renderer protocol", () => {
    expect(target.manifest).toMatchObject({
      format: rendererProtocolFormat,
      frame_format: rendererFrameFormat,
      event_format: rendererEventFormat,
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
    expect(container.querySelector("nav[aria-label='Session views']")).not.toBeNull();
    expect(container.textContent).toContain("README context");
    expect(container.textContent).toContain("workspace.read");
    expect(container.dataset.surfaceProfile).toBeDefined();
    expect(container.style.getPropertyValue("--bg")).toMatch(/^#[0-9a-f]{6}$/i);

    container
      .querySelector<HTMLButtonElement>("button[aria-label='Remove README context from context']")
      ?.click();
    expect(actions.at(-1)).toEqual({
      format: rendererActionFormat,
      kind: "remove_context_pin",
      pin_id: "pin-readme",
    });

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
      format: "agentide.renderer-event/2",
      kind: "notice",
      message: "Host observation",
    });
    await nextTick();
    expect(container.textContent).toContain("Host observation");

    handle.destroy();
    await nextTick();
    expect(container.childElementCount).toBe(0);
    expect(container.classList).not.toContain("agentide-root");
    expect(container.dataset.surfaceProfile).toBeUndefined();
    expect(container.style.getPropertyValue("--bg")).toBe("");
    container.remove();
  });
});

it.each(targets)("$manifest.id renders incomplete Markdown safely while streaming", async (target) => {
  const frame = fixture();
  frame.workbench.panes = [{ id: "chat", kind: "chat", title: "Agent" }];
  frame.workbench.focused_pane = "chat";
  frame.workbench.projections = {
    chat: {
      kind: "chat",
      messages: [{
        id: "message-1",
        role: "assistant",
        markdown: "# Live\n<script>alert(1)</script>\n```ts\nconst answer = 4",
        state: "streaming",
        created_at: "2026-09-04T20:00:00Z",
      }],
    },
  };
  const container = document.createElement("div");
  document.body.append(container);
  const handle = target.mount(container, { frame, dispatch: vi.fn() });
  await nextTick();
  expect(container.querySelector("script")).toBeNull();
  expect(container.textContent).toContain("<script>alert(1)</script>");
  expect(container.querySelector(".chat-transcript[aria-live='polite']")).not.toBeNull();
  expect(container.querySelector("textarea[aria-label='Message the agent']")).not.toBeNull();
  handle.destroy();
  container.remove();
});

it("identifies the imperative Vue target with its released renderer protocol", async () => {
  const container = document.createElement("div");
  document.body.append(container);
  const handle = vueRenderer.mount(container, { frame: fixture(), dispatch: vi.fn() });
  await nextTick();

  const surface = container.querySelector<HTMLElement>("[data-agentide-renderer='vue']");
  expect(surface?.dataset.agentideRendererProtocol).toBe(rendererProtocolFormat);

  handle.destroy();
  container.remove();
});

it("composes host-owned views without acquiring transport authority", async () => {
  const container = document.createElement("div");
  document.body.append(container);
  const app = createApp({
    render: () =>
      h(
        AgentIdeVueWorkbench,
        { bottomOpen: true, bottomHeight: 280 },
        {
          titlebar: () => "Hosted session",
          explorer: () => h("button", { type: "button" }, "README.md"),
          center: () => h("article", "Host-owned Monaco projection"),
          inspector: () => h("dl", [h("dt", "Actor"), h("dd", "server-derived")]),
          bottom: () => h("div", "Host-owned terminal renderer"),
          overlay: () => h("dialog", "Exact-plan approval"),
        },
      ),
  });
  app.mount(container);
  await nextTick();

  const shell = container.querySelector<HTMLElement>("[data-agentide-renderer='vue']");
  expect(shell?.dataset.agentideRendererProtocol).toBe(rendererProtocolFormat);
  expect(container.querySelector(".workbench-grid")?.classList).not.toContain(
    "terminal-collapsed",
  );
  expect(
    container
      .querySelector<HTMLElement>(".workbench-grid")
      ?.style.getPropertyValue("--terminal-height"),
  ).toBe("280px");
  expect(container.textContent).toContain("Host-owned Monaco projection");
  expect(container.textContent).toContain("Host-owned terminal renderer");

  app.unmount();
  container.remove();
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

it("exports host-embeddable styles without global document selectors", async () => {
  const { readFile } = await import("node:fs/promises");
  const { resolve } = await import("node:path");
  const [styles, packageSource] = await Promise.all([
    readFile(resolve(process.cwd(), "src/styles.css"), "utf8"),
    readFile(resolve(process.cwd(), "../package.json"), "utf8"),
  ]);
  expect(styles).not.toMatch(/^\s*(?::root|body|html|#app|\*)\b/m);
  expect(styles).not.toContain("100vh");
  expect(JSON.parse(packageSource).exports["./styles"]).toBe("./web/src/styles.css");
});

it("mounts the same framework-neutral editor and terminal leaves in both targets", async () => {
  for (const createTarget of [createVanillaRenderer, createVueRenderer]) {
    const editorHandle = { update: vi.fn(), focus: vi.fn(), destroy: vi.fn() };
    const terminalHandle = {
      write: vi.fn(),
      focus: vi.fn(),
      resize: vi.fn(),
      destroy: vi.fn(),
    };
    const editor = { mount: vi.fn(() => editorHandle) };
    const terminal = { mount: vi.fn(() => terminalHandle) };
    const target = createTarget({ editor, terminal });
    const container = document.createElement("div");
    document.body.append(container);
    const editorRenderer = target.mount(container, { frame: fixture(), dispatch: vi.fn() });
    await nextTick();
    expect(editor.mount).toHaveBeenCalledOnce();
    editorRenderer.destroy();
    expect(editorHandle.destroy).toHaveBeenCalledOnce();

    const terminalFrame = fixture();
    terminalFrame.workbench.panes = [{ id: "terminal", kind: "terminal", title: "Terminal" }];
    terminalFrame.workbench.focused_pane = "terminal";
    terminalFrame.workbench.projections = {
      terminal: {
        kind: "terminal",
        terminal_id: "terminal-1",
        state: "open",
        columns: 100,
        rows: 30,
      },
    };
    const terminalRenderer = target.mount(container, { frame: terminalFrame, dispatch: vi.fn() });
    await nextTick();
    expect(terminal.mount).toHaveBeenCalledOnce();
    terminalRenderer.deliver({
      format: "agentide.renderer-event/2",
      kind: "terminal_output",
      terminal_id: "terminal-1",
      sequence: 1,
      bytes: new Uint8Array([65]),
    });
    expect(terminalHandle.write).toHaveBeenCalledWith(new Uint8Array([65]));
    terminalRenderer.destroy();
    expect(terminalHandle.destroy).toHaveBeenCalledOnce();
    container.remove();
  }
});

it("makes every supported rail destination accessible and actionable in both targets", async () => {
  for (const target of targets) {
    const frame = fixture();
    frame.workbench.panes.push(
      { id: "chat", kind: "chat", title: "Agent" },
      { id: "changes", kind: "diff", title: "Changes" },
    );
    const actions: RendererAction[] = [];
    const container = document.createElement("div");
    document.body.append(container);
    const handle = target.mount(container, {
      frame,
      dispatch: (value) => actions.push(value),
    });
    await nextTick();

    for (const label of ["Workspace explorer", "Workspace changes", "Agent chat", "Terminal"]) {
      expect(container.querySelector(`button[aria-label='${label}']`)).not.toBeNull();
    }
    container.querySelector<HTMLButtonElement>("button[aria-label='Workspace explorer']")?.click();
    container.querySelector<HTMLButtonElement>("button[aria-label='Workspace changes']")?.click();
    container.querySelector<HTMLButtonElement>("button[aria-label='Agent chat']")?.click();
    container.querySelector<HTMLButtonElement>("button[aria-label='Terminal']")?.click();
    expect(actions.slice(-4)).toEqual([
      { format: rendererActionFormat, kind: "focus_pane", pane_id: "pane-readme" },
      { format: rendererActionFormat, kind: "show_diff" },
      { format: rendererActionFormat, kind: "focus_pane", pane_id: "chat" },
      {
        format: rendererActionFormat,
        kind: "open_terminal",
        columns: 120,
        rows: 30,
      },
    ]);

    handle.update({
      ...frame,
      workbench: {
        ...frame.workbench,
        panes: [
          ...frame.workbench.panes,
          { id: "terminal", kind: "terminal", title: "Terminal" },
        ],
      },
    });
    await nextTick();
    container.querySelector<HTMLButtonElement>("button[aria-label='Terminal']")?.click();
    expect(actions.at(-1)).toEqual({
      format: rendererActionFormat,
      kind: "focus_pane",
      pane_id: "terminal",
    });

    handle.destroy();
    container.remove();
  }
});

afterEach(() => {
  vi.restoreAllMocks();
  document.body.replaceChildren();
});
