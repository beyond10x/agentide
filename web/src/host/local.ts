import {
  rendererEventFormat,
  rendererFrameFormat,
  type PaneProjection,
  type RendererAction,
  type RendererFrame,
  type RendererTarget,
} from "../renderer/protocol";

type Snapshot = {
  session_id: string;
  objective: string;
  status: RendererFrame["session"]["status"];
  cursor: number;
  workbench: RendererFrame["workbench"];
  pending_approvals: RendererFrame["pending_approvals"];
};

type JournalEvent = RendererFrame["activity"][number] & { payload: unknown };

async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    headers: { "content-type": "application/json" },
    ...init,
  });
  const value = (await response.json()) as { message?: string };
  if (!response.ok) throw new Error(value.message ?? `request failed (${response.status})`);
  return value as T;
}

export async function mountLocalHost(target: RendererTarget, container: HTMLElement): Promise<() => void> {
  let observation: PaneProjection | undefined;
  let handle: ReturnType<RendererTarget["mount"]> | undefined;
  let alive = true;

  const readFrame = async (): Promise<RendererFrame> => {
    const [snapshot, events] = await Promise.all([
      api<Snapshot>("/api/snapshot"),
      api<JournalEvent[]>("/api/events"),
    ]);
    const frame: RendererFrame = {
      format: rendererFrameFormat,
      session: {
        id: snapshot.session_id,
        objective: snapshot.objective,
        status: snapshot.status,
        cursor: snapshot.cursor,
      },
      workbench: {
        ...snapshot.workbench,
        projections: { ...snapshot.workbench.projections },
      },
      context_pins: [],
      grants: [],
      pending_approvals: snapshot.pending_approvals,
      activity: events.map(({ sequence, at, kind, intent }) => ({ sequence, at, kind, intent })),
    };
    const pane = snapshot.workbench.focused_pane;
    if (pane && observation) frame.workbench.projections[pane] = observation;
    return frame;
  };

  const refresh = async () => {
    const frame = await readFrame();
    if (!alive) return;
    if (handle) handle.update(frame);
    else handle = target.mount(container, { frame, dispatch: (action) => void dispatch(action) });
  };

  const call = async (intent: string, input: Record<string, unknown> = {}) => {
    await api(`/api/intents/${intent}/call`, {
      method: "POST",
      body: JSON.stringify({ input }),
    });
    await refresh();
  };

  const dispatch = async (action: RendererAction) => {
    try {
      switch (action.kind) {
        case "refresh":
          await refresh();
          break;
        case "open_file": {
          await call("file_open", { path: action.path });
          const result = await api<{ path: string; content: string }>(`/api/intents/code_read/call`, {
            method: "POST",
            body: JSON.stringify({ input: { path: action.path } }),
          });
          observation = {
            kind: "editor",
            document: {
              path: result.path,
              content: result.content,
              language: "plaintext",
              version: "local",
              read_only: false,
              dirty: false,
            },
          };
          await refresh();
          break;
        }
        case "focus_pane":
          await call("pane_focus", { pane_id: action.pane_id });
          break;
        case "close_pane":
          await call("pane_close", { pane_id: action.pane_id });
          break;
        case "show_diff": {
          await call("diff_show");
          const result = await api<{ baseline_commit?: string; changes?: RendererFrame["workbench"]["projections"][string] }>("/api/intents/code_changes/call", {
            method: "POST",
            body: JSON.stringify({ input: {} }),
          });
          observation = result.changes ?? {
            kind: "diff",
            baseline_commit: result.baseline_commit ?? "unknown",
            changes: [],
            truncated: false,
          };
          await refresh();
          break;
        }
        case "approve":
          await api(`/api/approvals/${action.plan_digest}`, { method: "POST", body: "{}" });
          await refresh();
          break;
        case "deny":
          handle?.deliver({
            format: rendererEventFormat,
            kind: "notice",
            message: "This host does not expose denial yet.",
          });
          break;
        default:
          handle?.deliver({
            format: rendererEventFormat,
            kind: "notice",
            message: `The local host does not bind ${action.kind}.`,
          });
      }
    } catch (error) {
      handle?.deliver({ format: rendererEventFormat, kind: "notice", message: String(error) });
    }
  };

  await refresh();
  const timer = window.setInterval(() => void refresh(), 2_000);
  return () => {
    alive = false;
    window.clearInterval(timer);
    handle?.destroy();
  };
}
