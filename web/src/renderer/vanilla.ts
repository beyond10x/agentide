import {
  action,
  rendererActionFormat,
  rendererEventFormat,
  rendererFrameFormat,
  rendererProtocolFormat,
  type RendererAction,
  type RendererActionInput,
  type RendererFrame,
  type RendererHandle,
  type RendererTarget,
} from "./protocol";
import { focusedPane, glyph, installTheme, paneObservation } from "./shared";

type LocalState = { palette: boolean };

function escape(value: string): string {
  return value.replace(
    /[&<>'"]/g,
    (character) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "'": "&#39;", '"': "&quot;" })[
        character
      ] ?? character,
  );
}

function body(frame: RendererFrame): string {
  const pane = focusedPane(frame);
  const canvas = pane
    ? `<header class="pane-title"><div><span class="eyebrow">${escape(pane.kind)}</span><h2>${escape(pane.title)}</h2></div><span>${pane.line ? `line ${pane.line}` : "workspace view"}</span></header><pre>${escape(paneObservation(frame, pane))}</pre>`
    : `<section class="welcome"><div class="mark">A</div><p class="eyebrow">Agent-native workbench</p><h2>The session is the interface.</h2><p>Files, diffs, processes, approvals, agents, and evidence are projections of one replayable event stream.</p><div class="actions"><button data-action="open">Open file <kbd>O</kbd></button><button data-action="diff">Show changes <kbd>D</kbd></button></div></section>`;
  return `<header class="topbar"><a class="brand" href="/"><span>A</span>AgentIDE <em>vanilla</em></a><div class="session"><strong>${escape(frame.session.objective)}</strong><small>${escape(frame.session.status)} · event ${frame.session.cursor} · ${escape(frame.session.id.slice(-8))}</small></div><button class="command" data-action="palette">Command <kbd>⌘ K</kbd></button></header><main class="shell"><aside class="rail"><nav aria-label="Session views"><button class="selected" title="Workspace">◫</button><button data-action="diff" title="Changes">±</button><button title="Processes">›_</button><button title="Agents">◎</button><button title="Evidence">✓</button></nav><button class="settings" title="Bindings">⌘</button></aside><aside class="explorer"><div class="section-title"><span>OPEN FILES</span><button data-action="open" title="Open file">+</button></div><div>${frame.workbench.open_files.length ? frame.workbench.open_files.map((path) => `<button class="file" data-path="${escape(path)}"><span>◫</span>${escape(path)}</button>`).join("") : `<p class="empty">No files open. Press <kbd>O</kbd>.</p>`}</div><div class="section-title"><span>APPROVALS</span></div><div>${frame.pending_approvals.length ? frame.pending_approvals.map((plan) => `<article class="approval"><small>exact plan</small><strong>${escape(plan.intent)}</strong><code>${escape(plan.digest.slice(0, 16))}…</code><button data-approve="${escape(plan.digest)}">Approve exact plan</button><button class="deny" data-deny="${escape(plan.digest)}">Deny</button></article>`).join("") : `<p class="empty good">No effects waiting for authority.</p>`}</div></aside><section class="workbench"><div class="tabs">${frame.workbench.panes.map((item) => `<button class="tab ${item.id === frame.workbench.focused_pane ? "active" : ""}" data-pane="${escape(item.id)}"><span>${glyph(item.kind)}</span>${escape(item.title)}<i data-close="${escape(item.id)}">×</i></button>`).join("")}</div><div class="canvas">${canvas}</div><div class="notice" role="status">${escape(frame.notice ?? "")}</div></section><aside class="context"><div class="section-title"><span>SESSION TIMELINE</span><button data-action="refresh">↻</button></div><ol>${frame.activity.slice(-12).reverse().map((event) => `<li><span>${event.sequence}</span><div><strong>${escape(event.intent ?? event.kind)}</strong><small>${escape(event.kind)} · ${escape(new Date(event.at).toLocaleTimeString())}</small></div></li>`).join("")}</ol></aside></main><footer><span><b>●</b> Substrate boundary</span><span>Semantic actions · exact approvals · durable replay</span><span>${rendererFrameFormat}</span></footer><dialog class="palette"><div><span>⌘</span><input autocomplete="off" placeholder="Open file, show diff, refresh…" /></div><p><kbd>Enter</kbd> run · <kbd>Esc</kbd> close</p></dialog>`;
}

export const vanillaRenderer: RendererTarget = {
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
    const local: LocalState = { palette: false };
    installTheme(container);
    const render = () => {
      container.innerHTML = body(frame);
      const palette = container.querySelector<HTMLDialogElement>("dialog.palette");
      if (local.palette && palette && !palette.open) palette.showModal();
    };
    const dispatch = (value: RendererActionInput) => options.dispatch(action(value));
    const click = (event: Event) => {
      const target = event.target as HTMLElement;
      const close = target.closest<HTMLElement>("[data-close]")?.dataset.close;
      const pane = target.closest<HTMLElement>("[data-pane]")?.dataset.pane;
      const path = target.closest<HTMLElement>("[data-path]")?.dataset.path;
      const approve = target.closest<HTMLElement>("[data-approve]")?.dataset.approve;
      const deny = target.closest<HTMLElement>("[data-deny]")?.dataset.deny;
      const localAction = target.closest<HTMLElement>("[data-action]")?.dataset.action;
      if (close) dispatch({ kind: "close_pane", pane_id: close });
      else if (pane) dispatch({ kind: "focus_pane", pane_id: pane });
      else if (path) dispatch({ kind: "open_file", path });
      else if (approve) dispatch({ kind: "approve", plan_digest: approve });
      else if (deny) dispatch({ kind: "deny", plan_digest: deny });
      else if (localAction === "refresh") dispatch({ kind: "refresh" });
      else if (localAction === "diff") dispatch({ kind: "show_diff" });
      else if (localAction === "open") {
        const requested = window.prompt("Workspace-relative file path");
        if (requested) dispatch({ kind: "open_file", path: requested });
      } else if (localAction === "palette") {
        local.palette = true;
        render();
      }
    };
    container.addEventListener("click", click);
    render();
    return {
      update(next) {
        frame = next;
        render();
      },
      deliver(event) {
        if (event.kind === "notice") {
          frame = { ...frame, notice: event.message };
          render();
        }
      },
      destroy() {
        container.removeEventListener("click", click);
        container.replaceChildren();
      },
    };
  },
};
