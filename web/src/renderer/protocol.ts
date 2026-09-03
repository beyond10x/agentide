export const rendererProtocolFormat = "agentide.renderer-target/1" as const;
export const rendererFrameFormat = "agentide.renderer-frame/1" as const;
export const rendererEventFormat = "agentide.renderer-event/1" as const;
export const rendererActionFormat = "agentide.renderer-action/1" as const;

export type Pane = {
  id: string;
  kind: string;
  title: string;
  path?: string;
  line?: number;
  column?: number;
};

export type PendingApproval = {
  digest: string;
  intent: string;
  risk?: string;
  approval_required: boolean;
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
    status: string;
    cursor: number;
  };
  workbench: {
    panes: Pane[];
    focused_pane?: string;
    open_files: string[];
  };
  pending_approvals: PendingApproval[];
  activity: Activity[];
  observation?: unknown;
  notice?: string;
};

export type RendererEvent =
  | { format: typeof rendererEventFormat; kind: "text_delta"; text: string }
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
  | { format: typeof rendererActionFormat; kind: "open_file"; path: string }
  | { format: typeof rendererActionFormat; kind: "focus_pane"; pane_id: string }
  | { format: typeof rendererActionFormat; kind: "close_pane"; pane_id: string }
  | { format: typeof rendererActionFormat; kind: "show_diff" }
  | { format: typeof rendererActionFormat; kind: "approve"; plan_digest: string }
  | { format: typeof rendererActionFormat; kind: "deny"; plan_digest: string }
  | { format: typeof rendererActionFormat; kind: "submit_prompt"; content: string }
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
