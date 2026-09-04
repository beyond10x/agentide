export {
  rendererActionFormat,
  rendererEventFormat,
  rendererFrameFormat,
  rendererProtocolFormat,
} from "../generated/renderer-protocol";

import {
  rendererActionFormat,
  rendererEventFormat,
  rendererFrameFormat,
  rendererProtocolFormat,
} from "../generated/renderer-protocol";

export type PaneKind =
  | "editor"
  | "diff"
  | "terminal"
  | "chat"
  | "timeline"
  | "agents"
  | "approvals"
  | "evidence";

export type Pane = {
  id: string;
  kind: PaneKind;
  title: string;
  path?: string;
  line?: number;
  column?: number;
};

export type TreeEntry = {
  path: string;
  name: string;
  kind: "file" | "directory" | "symlink";
};

export type TreeProjection = {
  kind: "tree";
  root: string;
  entries: TreeEntry[];
  next_cursor?: string;
};

export type EditorDocument = {
  path: string;
  language: string;
  content: string;
  version: string;
  read_only: boolean;
  dirty: boolean;
};

export type EditorProjection = { kind: "editor"; document: EditorDocument };

export type Change = {
  path: string;
  status: "added" | "modified" | "deleted" | "renamed" | "untracked";
  patch?: string;
};

export type DiffProjection = {
  kind: "diff";
  baseline_commit: string;
  changes: Change[];
  truncated: boolean;
};

export type ChatMessage = {
  id: string;
  role: "user" | "assistant" | "system";
  markdown: string;
  state: "streaming" | "complete" | "failed";
  created_at: string;
};

export type ChatProjection = { kind: "chat"; messages: ChatMessage[] };

export type TerminalProjection = {
  kind: "terminal";
  terminal_id: string;
  state: "opening" | "open" | "closed" | "failed";
  columns: number;
  rows: number;
};

export type RefusalProjection = {
  kind: "refusal";
  code: string;
  message: string;
  retryable: boolean;
};

export type EmptyProjection = { kind: "empty"; message: string };

export type PaneProjection =
  | EditorProjection
  | DiffProjection
  | ChatProjection
  | TerminalProjection
  | RefusalProjection
  | EmptyProjection;

export type PendingApproval = {
  digest: string;
  intent: string;
  risk?: string;
  approval_required: boolean;
};

export type ContextPin = { id: string; label: string; source: string };
export type GrantSummary = {
  id: string;
  capability: string;
  state: "active" | "expired" | "revoked";
};

export type Activity = {
  sequence: number;
  at: string;
  kind: string;
  intent?: string;
};

export type RendererFrame = {
  format: typeof rendererFrameFormat;
  session: {
    id: string;
    objective: string;
    status: "preparing" | "active" | "closed" | "completed" | "failed" | "superseded";
    cursor: number;
  };
  preparation?: { stage: string; message: string; retryable: boolean };
  workbench: {
    panes: Pane[];
    focused_pane?: string;
    open_files: string[];
    projections: Record<string, PaneProjection>;
    tree?: TreeProjection;
  };
  pending_approvals: PendingApproval[];
  context_pins: ContextPin[];
  grants: GrantSummary[];
  activity: Activity[];
  notice?: string;
};

export type RendererEvent =
  | {
      format: typeof rendererEventFormat;
      kind: "assistant_delta";
      message_id: string;
      sequence: number;
      markdown_delta: string;
    }
  | {
      format: typeof rendererEventFormat;
      kind: "terminal_output";
      terminal_id: string;
      sequence: number;
      bytes: Uint8Array;
    }
  | { format: typeof rendererEventFormat; kind: "notice"; message: string };

export type RendererAction =
  | { format: typeof rendererActionFormat; kind: "refresh" }
  | { format: typeof rendererActionFormat; kind: "load_tree"; path: string; cursor?: string }
  | { format: typeof rendererActionFormat; kind: "open_file"; path: string }
  | {
      format: typeof rendererActionFormat;
      kind: "edit_file";
      path: string;
      content: string;
      version: string;
    }
  | {
      format: typeof rendererActionFormat;
      kind: "save_file";
      path: string;
      content: string;
      version: string;
    }
  | { format: typeof rendererActionFormat; kind: "focus_pane"; pane_id: string }
  | { format: typeof rendererActionFormat; kind: "close_pane"; pane_id: string }
  | { format: typeof rendererActionFormat; kind: "show_diff" }
  | { format: typeof rendererActionFormat; kind: "approve"; plan_digest: string }
  | { format: typeof rendererActionFormat; kind: "deny"; plan_digest: string }
  | { format: typeof rendererActionFormat; kind: "submit_prompt"; content: string }
  | { format: typeof rendererActionFormat; kind: "pin_context"; source: string }
  | { format: typeof rendererActionFormat; kind: "remove_context_pin"; pin_id: string }
  | { format: typeof rendererActionFormat; kind: "open_terminal"; columns: number; rows: number }
  | {
      format: typeof rendererActionFormat;
      kind: "terminal_input";
      terminal_id: string;
      data: string;
    }
  | {
      format: typeof rendererActionFormat;
      kind: "terminal_resize";
      terminal_id: string;
      columns: number;
      rows: number;
    };

export type RendererActionInput = RendererAction extends infer Candidate
  ? Candidate extends { format: typeof rendererActionFormat }
    ? Omit<Candidate, "format">
    : never
  : never;

export type RendererTargetManifest = {
  format: typeof rendererProtocolFormat;
  id: string;
  framework: string;
  frame_format: typeof rendererFrameFormat;
  event_format: typeof rendererEventFormat;
  action_format: typeof rendererActionFormat;
};

export type RendererOptions = {
  frame: RendererFrame;
  dispatch: (action: RendererAction) => void;
};

export interface RendererHandle {
  update(frame: RendererFrame): void;
  deliver(event: RendererEvent): void;
  destroy(): void;
}

export interface RendererTarget {
  readonly manifest: RendererTargetManifest;
  mount(container: HTMLElement, options: RendererOptions): RendererHandle;
}

export function action(value: RendererActionInput): RendererAction {
  return { format: rendererActionFormat, ...value } as RendererAction;
}
