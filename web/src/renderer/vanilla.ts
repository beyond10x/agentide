import type { EditorAdapterHandle } from "../adapters/editor";
import type { TerminalAdapterHandle } from "../adapters/terminal";
import type { RendererAdapters } from "./dependencies";
import { renderMarkdown } from "./markdown";
import {
  action,
  rendererActionFormat,
  rendererEventFormat,
  rendererFrameFormat,
  rendererProtocolFormat,
  type PaneProjection,
  type RendererActionInput,
  type RendererFrame,
  type RendererHandle,
  type RendererTarget,
} from "./protocol";
import {
  defaultTerminalSize,
  focusedPane,
  glyph,
  installTheme,
  paneProjection,
  uninstallTheme,
} from "./shared";

type LocalState = { palette: boolean; promptDraft: string };

function escape(value: string): string {
  return value.replace(
    /[&<>'"]/g,
    (character) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "'": "&#39;", '"': "&quot;" })[
        character
      ] ?? character,
  );
}

function projectionBody(projection: PaneProjection | undefined, adapters: RendererAdapters, draft: string): string {
  if (!projection) return '<p class="empty">Select an observation to populate this pane.</p>';
  switch (projection.kind) {
    case "editor":
      return adapters.editor
        ? `<div class="editor-leaf" data-editor-leaf data-path="${escape(projection.document.path)}"></div>`
        : `<pre>${escape(projection.document.content)}</pre>`;
    case "diff":
      return `<section class="change-list"><p><code>${escape(projection.baseline_commit)}</code>${projection.truncated ? " · truncated" : ""}</p>${projection.changes.map((change) => `<article><strong>${escape(change.status)}</strong> ${escape(change.path)}${change.patch ? `<pre>${escape(change.patch)}</pre>` : ""}</article>`).join("")}</section>`;
    case "chat":
      return `<section class="agent-chat"><div class="chat-transcript" aria-live="polite">${projection.messages.map((message) => `<article class="chat-message ${message.role}" data-state="${message.state}"><small>${message.role}</small><div class="markdown">${renderMarkdown(message.markdown)}</div></article>`).join("")}</div><form data-chat-form><textarea name="prompt" aria-label="Message the agent" placeholder="Ask the agent…">${escape(draft)}</textarea><button type="submit">Send</button></form></section>`;
    case "terminal":
      return adapters.terminal
        ? `<div class="terminal-leaf" data-terminal-leaf data-terminal-id="${escape(projection.terminal_id)}"></div>`
        : `<p class="empty">Terminal ${escape(projection.state)} · ${projection.columns}×${projection.rows}</p>`;
    case "refusal":
      return `<section class="pane-refusal" role="alert"><strong>${escape(projection.code)}</strong><p>${escape(projection.message)}</p>${projection.retryable ? "<small>Retryable</small>" : ""}</section>`;
    case "empty":
      return `<p class="empty">${escape(projection.message)}</p>`;
  }
}

function explorer(frame: RendererFrame): string {
  const tree = frame.workbench.tree;
  if (!tree) {
    return '<p class="empty"><button data-load-tree="">Load workspace</button></p>';
  }
  const entries = tree.entries
    .map((entry) => `<button class="file" data-tree-path="${escape(entry.path)}" data-tree-kind="${entry.kind}"><span>${entry.kind === "directory" ? "▸" : "◫"}</span>${escape(entry.name)}</button>`)
    .join("");
  const more = tree.next_cursor
    ? `<button class="file" data-load-tree="${escape(tree.root)}" data-tree-cursor="${escape(tree.next_cursor)}">Load more…</button>`
    : "";
  return entries || more ? entries + more : '<p class="empty">This directory is empty.</p>';
}

function body(frame: RendererFrame, adapters: RendererAdapters, local: LocalState): string {
  const pane = focusedPane(frame);
  const projection = pane ? paneProjection(frame, pane) : undefined;
  const selectedView = pane?.kind;
  const canvas = pane
    ? `<header class="pane-title"><div><span class="eyebrow">${escape(pane.kind)}</span><h2>${escape(pane.title)}${projection?.kind === "editor" && projection.document.dirty ? '<span class="pane-dirty" aria-label="Unsaved changes"> ●</span>' : ""}</h2></div><span>${pane.line ? `line ${pane.line}` : "workspace view"}</span></header>${projectionBody(projection, adapters, local.promptDraft)}`
    : '<section class="welcome"><div class="mark">A</div><p class="eyebrow">Agent-native workbench</p><h2>The session is the interface.</h2><p>Files, diffs, processes, approvals, agents, and evidence are projections of one replayable event stream.</p><div class="actions"><button data-action="open">Open file <kbd>O</kbd></button><button data-action="diff">Show changes <kbd>D</kbd></button></div></section>';
  const preparation = frame.preparation
    ? `<div class="preparation" role="status"><strong>${escape(frame.preparation.stage)}</strong> ${escape(frame.preparation.message)}</div>`
    : "";
  return `<header class="topbar"><a class="brand" href="/"><span>A</span>AgentIDE <em>vanilla</em></a><div class="session"><strong>${escape(frame.session.objective)}</strong><small>${escape(frame.session.status)} · event ${frame.session.cursor} · ${escape(frame.session.id.slice(-8))}</small></div><button class="command" data-action="palette">Command <kbd>⌘ K</kbd></button></header>${preparation}<main class="shell"><aside class="rail"><nav aria-label="Session views"><button class="${selectedView === "editor" ? "selected" : ""}" data-action="explorer" title="Workspace explorer" aria-label="Workspace explorer">◫</button><button class="${selectedView === "diff" ? "selected" : ""}" data-action="diff" title="Workspace changes" aria-label="Workspace changes">±</button><button class="${selectedView === "chat" ? "selected" : ""}" data-action="chat" title="Agent chat" aria-label="Agent chat">◎</button><button class="${selectedView === "terminal" ? "selected" : ""}" data-action="terminal" title="Terminal" aria-label="Terminal">›_</button></nav><button class="settings" title="Bindings" aria-label="Bindings">⌘</button></aside><aside class="explorer"><div class="section-title"><span>WORKSPACE</span><button data-load-tree="" title="Refresh workspace">↻</button></div><div>${explorer(frame)}</div><div class="section-title"><span>OPEN FILES</span><button data-action="open" title="Open file">+</button></div><div>${frame.workbench.open_files.length ? frame.workbench.open_files.map((path) => `<button class="file" data-path="${escape(path)}"><span>◫</span>${escape(path)}</button>`).join("") : '<p class="empty">No files open. Press <kbd>O</kbd>.</p>'}</div><div class="section-title"><span>APPROVALS</span></div><div>${frame.pending_approvals.length ? frame.pending_approvals.map((plan) => `<article class="approval"><small>exact plan</small><strong>${escape(plan.intent)}</strong><code>${escape(plan.digest.slice(0, 16))}…</code><button data-approve="${escape(plan.digest)}">Approve exact plan</button><button class="deny" data-deny="${escape(plan.digest)}">Deny</button></article>`).join("") : '<p class="empty good">No effects waiting for authority.</p>'}</div></aside><section class="workbench"><div class="tabs">${frame.workbench.panes.map((item) => `<button class="tab ${item.id === frame.workbench.focused_pane ? "active" : ""}" data-pane="${escape(item.id)}"><span>${glyph(item.kind)}</span>${escape(item.title)}<i data-close="${escape(item.id)}">×</i></button>`).join("")}</div><div class="canvas">${canvas}</div><div class="notice" role="status">${escape(frame.notice ?? "")}</div></section><aside class="context"><div class="section-title"><span>SESSION TIMELINE</span><button data-action="refresh">↻</button></div><ol>${frame.activity.slice(-12).reverse().map((event) => `<li><span>${event.sequence}</span><div><strong>${escape(event.intent ?? event.kind)}</strong><small>${escape(event.kind)} · ${escape(new Date(event.at).toLocaleTimeString())}</small></div></li>`).join("")}</ol><div class="section-title"><span>CONTEXT</span></div><div>${frame.context_pins.length ? frame.context_pins.map((pin) => `<article class="context-pin"><div><strong>${escape(pin.label)}</strong><small>${escape(pin.source)}</small></div><button data-remove-pin="${escape(pin.id)}" aria-label="Remove ${escape(pin.label)} from context">×</button></article>`).join("") : '<p class="empty">No pinned context.</p>'}</div><div class="section-title"><span>CAPABILITIES</span></div>${frame.grants.map((grant) => `<p class="grant" data-state="${grant.state}">${escape(grant.capability)}</p>`).join("")}</aside></main><footer><span><b>●</b> Substrate boundary</span><span>Semantic actions · exact approvals · durable replay</span><span>${rendererFrameFormat}</span></footer><dialog class="palette"><div><span>⌘</span><input autocomplete="off" placeholder="Open file, show diff, refresh…" /></div><p><kbd>Enter</kbd> run · <kbd>Esc</kbd> close</p></dialog>`;
}

/** Builds a Vanilla DOM target over the same optional Monaco and Ghostty leaves as Vue. */
export function createVanillaRenderer(adapters: RendererAdapters = {}): RendererTarget {
  return {
    manifest: {
      format: rendererProtocolFormat,
      id: "vanilla",
      framework: "vanilla-dom",
      frame_format: rendererFrameFormat,
      event_format: rendererEventFormat,
      action_format: rendererActionFormat,
    },
    mount(container, options): RendererHandle {
      let frame = options.frame;
      const local: LocalState = { palette: false, promptDraft: "" };
      let editor: EditorAdapterHandle | undefined;
      let terminal: TerminalAdapterHandle | undefined;
      let terminalId: string | undefined;
      let terminalSequence = 0;
      installTheme(container);
      const dispatch = (value: RendererActionInput) => options.dispatch(action(value));
      const teardownLeaves = () => {
        editor?.destroy();
        terminal?.destroy();
        editor = undefined;
        terminal = undefined;
        terminalId = undefined;
        terminalSequence = 0;
      };
      const mountLeaves = () => {
        const pane = focusedPane(frame);
        const projection = pane ? paneProjection(frame, pane) : undefined;
        if (projection?.kind === "editor" && adapters.editor) {
          const element = container.querySelector<HTMLElement>("[data-editor-leaf]");
          if (element) {
            editor = adapters.editor.mount(element, projection.document, (content, version) =>
              dispatch({ kind: "edit_file", path: projection.document.path, content, version }),
            );
          }
        } else if (projection?.kind === "terminal" && adapters.terminal) {
          const element = container.querySelector<HTMLElement>("[data-terminal-leaf]");
          if (element) {
            terminalId = projection.terminal_id;
            terminal = adapters.terminal.mount(
              element,
              (data) => dispatch({ kind: "terminal_input", terminal_id: projection.terminal_id, data }),
              (columns, rows) => dispatch({ kind: "terminal_resize", terminal_id: projection.terminal_id, columns, rows }),
            );
            terminal.resize(projection.columns, projection.rows);
          }
        }
      };
      const render = () => {
        teardownLeaves();
        container.innerHTML = body(frame, adapters, local);
        const palette = container.querySelector<HTMLDialogElement>("dialog.palette");
        if (local.palette && palette && !palette.open) palette.showModal();
        mountLeaves();
        const transcript = container.querySelector<HTMLElement>(".chat-transcript");
        if (transcript) transcript.scrollTop = transcript.scrollHeight;
      };
      const click = (event: Event) => {
        const target = event.target as HTMLElement;
        const close = target.closest<HTMLElement>("[data-close]")?.dataset.close;
        const pane = target.closest<HTMLElement>("[data-pane]")?.dataset.pane;
        const path = target.closest<HTMLElement>("[data-path]")?.dataset.path;
        const treeTarget = target.closest<HTMLElement>("[data-tree-path]");
        const loadTarget = target.closest<HTMLElement>("[data-load-tree]");
        const approve = target.closest<HTMLElement>("[data-approve]")?.dataset.approve;
        const deny = target.closest<HTMLElement>("[data-deny]")?.dataset.deny;
        const removePin = target.closest<HTMLElement>("[data-remove-pin]")?.dataset.removePin;
        const localAction = target.closest<HTMLElement>("[data-action]")?.dataset.action;
        if (close) dispatch({ kind: "close_pane", pane_id: close });
        else if (pane) dispatch({ kind: "focus_pane", pane_id: pane });
        else if (treeTarget?.dataset.treePath !== undefined) {
          const treePath = treeTarget.dataset.treePath;
          if (treeTarget.dataset.treeKind === "directory") dispatch({ kind: "load_tree", path: treePath });
          else dispatch({ kind: "open_file", path: treePath });
        } else if (loadTarget?.dataset.loadTree !== undefined) {
          dispatch({ kind: "load_tree", path: loadTarget.dataset.loadTree, cursor: loadTarget.dataset.treeCursor });
        } else if (path) dispatch({ kind: "open_file", path });
        else if (approve) dispatch({ kind: "approve", plan_digest: approve });
        else if (deny) dispatch({ kind: "deny", plan_digest: deny });
        else if (removePin) dispatch({ kind: "remove_context_pin", pin_id: removePin });
        else if (localAction === "refresh") dispatch({ kind: "refresh" });
        else if (localAction === "explorer") {
          const editorPane = frame.workbench.panes.find((candidate) => candidate.kind === "editor");
          if (editorPane) dispatch({ kind: "focus_pane", pane_id: editorPane.id });
          else dispatch({ kind: "load_tree", path: "" });
        }
        else if (localAction === "diff") dispatch({ kind: "show_diff" });
        else if (localAction === "chat") {
          const chatPane = frame.workbench.panes.find((candidate) => candidate.kind === "chat");
          if (chatPane) dispatch({ kind: "focus_pane", pane_id: chatPane.id });
        }
        else if (localAction === "terminal") {
          const terminalPane = frame.workbench.panes.find((candidate) => candidate.kind === "terminal");
          if (terminalPane) dispatch({ kind: "focus_pane", pane_id: terminalPane.id });
          else dispatch({ kind: "open_terminal", ...defaultTerminalSize });
        }
        else if (localAction === "open") {
          const requested = window.prompt("Workspace-relative file path");
          if (requested) dispatch({ kind: "open_file", path: requested });
        } else if (localAction === "palette") {
          local.palette = true;
          render();
        }
      };
      const input = (event: Event) => {
        const target = event.target;
        if (target instanceof HTMLTextAreaElement && target.name === "prompt") {
          local.promptDraft = target.value;
        }
      };
      const submit = (event: Event) => {
        const form = (event.target as HTMLElement).closest<HTMLFormElement>("[data-chat-form]");
        if (!form) return;
        event.preventDefault();
        const content = local.promptDraft.trim();
        if (content) {
          local.promptDraft = "";
          dispatch({ kind: "submit_prompt", content });
        }
      };
      const keydown = (event: KeyboardEvent) => {
        if (!(event.metaKey || event.ctrlKey) || event.key.toLowerCase() !== "s") return;
        const pane = focusedPane(frame);
        const projection = pane ? paneProjection(frame, pane) : undefined;
        if (projection?.kind !== "editor" || !projection.document.dirty) return;
        event.preventDefault();
        dispatch({
          kind: "save_file",
          path: projection.document.path,
          content: projection.document.content,
          version: projection.document.version,
        });
      };
      container.addEventListener("click", click);
      container.addEventListener("input", input);
      container.addEventListener("submit", submit);
      container.addEventListener("keydown", keydown);
      render();
      return {
        update(next) {
          const previousProjection = focusedPane(frame) ? paneProjection(frame, focusedPane(frame)!) : undefined;
          const nextPane = focusedPane(next);
          const nextProjection = nextPane ? paneProjection(next, nextPane) : undefined;
          const leafOnly =
            editor &&
            previousProjection?.kind === "editor" &&
            nextProjection?.kind === "editor" &&
            previousProjection.document.path === nextProjection.document.path &&
            frame.session === next.session &&
            frame.workbench.panes === next.workbench.panes &&
            frame.pending_approvals === next.pending_approvals &&
            frame.activity === next.activity;
          frame = next;
          if (leafOnly) {
            editor?.update(nextProjection.document);
            const heading = container.querySelector<HTMLElement>(".pane-title h2");
            const marker = heading?.querySelector<HTMLElement>(".pane-dirty");
            if (nextProjection.document.dirty && heading && !marker) {
              const dirty = document.createElement("span");
              dirty.className = "pane-dirty";
              dirty.ariaLabel = "Unsaved changes";
              dirty.textContent = " ●";
              heading.append(dirty);
            } else if (!nextProjection.document.dirty) {
              marker?.remove();
            }
          } else render();
        },
        deliver(event) {
          if (event.kind === "notice") {
            frame = { ...frame, notice: event.message };
            render();
          } else if (
            event.kind === "terminal_output" &&
            event.terminal_id === terminalId &&
            event.sequence === terminalSequence + 1
          ) {
            terminalSequence = event.sequence;
            terminal?.write(event.bytes);
          }
        },
        destroy() {
          container.removeEventListener("click", click);
          container.removeEventListener("input", input);
          container.removeEventListener("submit", submit);
          container.removeEventListener("keydown", keydown);
          teardownLeaves();
          container.replaceChildren();
          uninstallTheme(container);
        },
      };
    },
  };
}

export const vanillaRenderer = createVanillaRenderer();
