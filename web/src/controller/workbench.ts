import {
  rendererEventFormat,
  rendererFrameFormat,
  type Change,
  type ChatMessage,
  type ContextPin,
  type GrantSummary,
  type PendingApproval,
  type RendererAction,
  type RendererEvent,
  type RendererFrame,
  type RendererHandle,
  type RendererTarget,
  type TreeProjection,
} from "../renderer/protocol";

export type WorkbenchSnapshot = {
  session: RendererFrame["session"];
  panes: RendererFrame["workbench"]["panes"];
  focused_pane?: string;
  open_files: string[];
  pending_approvals: PendingApproval[];
  context_pins: ContextPin[];
  grants: GrantSummary[];
  activity: RendererFrame["activity"];
  preparation?: RendererFrame["preparation"];
  projections?: RendererFrame["workbench"]["projections"];
  tree?: TreeProjection;
};

export type FileResult = {
  path: string;
  language: string;
  content: string;
  version: string;
  read_only: boolean;
};

export type ChangeSet = { baseline_commit: string; changes: Change[]; truncated: boolean };

/** A stable, operator-safe refusal from a host implementation. */
export class WorkbenchRefusal extends Error {
  readonly code: string;
  readonly retryable: boolean;

  constructor(code: string, message: string, retryable = false) {
    super(message);
    this.name = "WorkbenchRefusal";
    this.code = code;
    this.retryable = retryable;
  }
}

/**
 * Product-supplied authority and observation boundary. Implementations may know URLs and bearer
 * credentials; the controller and renderer targets do not.
 */
export interface WorkbenchHostPort {
  snapshot(signal: AbortSignal): Promise<WorkbenchSnapshot>;
  tree(path: string, cursor: string | undefined, signal: AbortSignal): Promise<TreeProjection>;
  openFile(path: string, signal: AbortSignal): Promise<FileResult>;
  saveFile(path: string, content: string, version: string, signal: AbortSignal): Promise<FileResult>;
  focusPane(paneId: string, signal: AbortSignal): Promise<void>;
  closePane(paneId: string, signal: AbortSignal): Promise<void>;
  changes(signal: AbortSignal): Promise<ChangeSet>;
  submitPrompt(
    content: string,
    onDelta: (markdownDelta: string) => void,
    signal: AbortSignal,
  ): Promise<ChatMessage>;
  approve(planDigest: string, signal: AbortSignal): Promise<void>;
  deny(planDigest: string, signal: AbortSignal): Promise<void>;
  pinContext(source: string, signal: AbortSignal): Promise<void>;
  removeContextPin(pinId: string, signal: AbortSignal): Promise<void>;
  openTerminal(columns: number, rows: number, signal: AbortSignal): Promise<string>;
  terminalInput(terminalId: string, data: string, signal: AbortSignal): Promise<void>;
  terminalResize(
    terminalId: string,
    columns: number,
    rows: number,
    signal: AbortSignal,
  ): Promise<void>;
}

type Listener = (frame: RendererFrame) => void;

/**
 * Framework-neutral workbench orchestration. It owns browser projection state and calls only the
 * explicit host port; renderer targets receive immutable frames and no transport authority.
 */
export class WorkbenchController {
  readonly #port: WorkbenchHostPort;
  readonly #abort = new AbortController();
  readonly #listeners = new Set<Listener>();
  readonly #assistantSequences = new Map<string, number>();
  #frame?: RendererFrame;
  #snapshotRequest = 0;
  #snapshotApplied = 0;
  #destroyed = false;

  constructor(port: WorkbenchHostPort) {
    this.#port = port;
  }

  frame(): RendererFrame | undefined {
    return this.#frame;
  }

  subscribe(listener: Listener): () => void {
    this.#assertAlive();
    this.#listeners.add(listener);
    if (this.#frame) listener(this.#frame);
    return () => this.#listeners.delete(listener);
  }

  async start(): Promise<void> {
    await this.refresh();
  }

  async refresh(): Promise<void> {
    this.#assertAlive();
    const request = ++this.#snapshotRequest;
    const snapshot = await this.#port.snapshot(this.#abort.signal);
    if (this.#destroyed || request < this.#snapshotApplied) return;
    this.#snapshotApplied = request;
    const previous = this.#frame;
    const paneIds = new Set(snapshot.panes.map((pane) => pane.id));
    const previousProjections = Object.fromEntries(
      Object.entries(previous?.workbench.projections ?? {}).filter(([paneId]) => paneIds.has(paneId)),
    );
    const projections = snapshot.projections
      ? Object.fromEntries(
          Object.entries(snapshot.projections).filter(([paneId]) => paneIds.has(paneId)),
        )
      : previousProjections;
    for (const [paneId, projection] of Object.entries(previousProjections)) {
      if (
        (projection.kind === "editor" && projection.document.dirty) ||
        (projection.kind === "chat" && projection.messages.some((message) => message.state === "streaming"))
      ) {
        projections[paneId] = projection;
      }
    }
    this.#publish({
      format: rendererFrameFormat,
      session: snapshot.session,
      preparation: snapshot.preparation,
      workbench: {
        panes: snapshot.panes,
        focused_pane: snapshot.focused_pane,
        open_files: snapshot.open_files,
        projections,
        tree: snapshot.tree ?? previous?.workbench.tree,
      },
      pending_approvals: snapshot.pending_approvals,
      context_pins: snapshot.context_pins,
      grants: snapshot.grants,
      activity: snapshot.activity,
    });
  }

  async dispatch(command: RendererAction): Promise<void> {
    this.#assertAlive();
    const signal = this.#abort.signal;
    try {
      this.#clearNotice();
      switch (command.kind) {
        case "refresh":
          await this.refresh();
          return;
        case "load_tree": {
          const page = await this.#port.tree(command.path, command.cursor, signal);
          const current = this.#requiredFrame().workbench.tree;
          this.#patchWorkbench({
            tree:
              command.cursor && current?.root === page.root
                ? { ...page, entries: [...current.entries, ...page.entries] }
                : page,
          });
          return;
        }
        case "open_file": {
          const existing = this.#editorProjection(command.path);
          if (existing?.document.dirty) {
            const pane = this.#editorPane(command.path);
            await this.#port.focusPane(pane, signal);
            await this.refresh();
            return;
          }
          const document = await this.#port.openFile(command.path, signal);
          await this.refresh();
          this.#setProjection(this.#editorPane(command.path), {
            kind: "editor",
            document: { ...document, dirty: false },
          });
          return;
        }
        case "edit_file": {
          const projection = this.#editorProjection(command.path);
          if (!projection || projection.document.version !== command.version) {
            throw new WorkbenchRefusal(
              "renderer.editor_version_stale",
              "The editor draft no longer matches the observed file version.",
            );
          }
          if (projection.document.read_only) {
            throw new WorkbenchRefusal(
              "renderer.editor_read_only",
              "This file is read-only in the current workspace.",
            );
          }
          this.#setProjection(this.#editorPane(command.path), {
            kind: "editor",
            document: { ...projection.document, content: command.content, dirty: true },
          });
          return;
        }
        case "save_file": {
          const document = await this.#port.saveFile(
            command.path,
            command.content,
            command.version,
            signal,
          );
          this.#setProjection(this.#editorPane(command.path), {
            kind: "editor",
            document: { ...document, dirty: false },
          });
          await this.refresh();
          return;
        }
        case "show_diff": {
          const changeSet = await this.#port.changes(signal);
          await this.refresh();
          const pane = this.#requiredFrame().workbench.panes.find(
            (candidate) => candidate.kind === "diff",
          );
          if (!pane) {
            throw new WorkbenchRefusal(
              "renderer.diff_pane_missing",
              "The host did not expose a changes pane.",
              true,
            );
          }
          if (this.#requiredFrame().workbench.focused_pane !== pane.id) {
            await this.#port.focusPane(pane.id, signal);
            await this.refresh();
          }
          this.#setProjection(pane.id, { kind: "diff", ...changeSet });
          return;
        }
        case "submit_prompt":
          await this.#submitPrompt(command.content, signal);
          return;
        case "approve":
          await this.#port.approve(command.plan_digest, signal);
          break;
        case "deny":
          await this.#port.deny(command.plan_digest, signal);
          break;
        case "pin_context":
          await this.#port.pinContext(command.source, signal);
          break;
        case "remove_context_pin":
          await this.#port.removeContextPin(command.pin_id, signal);
          break;
        case "open_terminal": {
          const terminalId = await this.#port.openTerminal(command.columns, command.rows, signal);
          await this.refresh();
          const pane = this.#requiredFrame().workbench.panes.find(
            (candidate) => candidate.kind === "terminal",
          );
          if (!pane) {
            throw new WorkbenchRefusal(
              "renderer.terminal_pane_missing",
              "The host did not expose the opened terminal pane.",
              true,
            );
          }
          if (this.#requiredFrame().workbench.focused_pane !== pane.id) {
            await this.#port.focusPane(pane.id, signal);
            await this.refresh();
          }
          this.#setProjection(pane.id, {
            kind: "terminal",
            terminal_id: terminalId,
            state: "open",
            columns: command.columns,
            rows: command.rows,
          });
          return;
        }
        case "terminal_input":
          await this.#port.terminalInput(command.terminal_id, command.data, signal);
          return;
        case "terminal_resize":
          await this.#port.terminalResize(
            command.terminal_id,
            command.columns,
            command.rows,
            signal,
          );
          return;
        case "focus_pane":
          await this.#port.focusPane(command.pane_id, signal);
          break;
        case "close_pane":
          await this.#port.closePane(command.pane_id, signal);
          break;
      }
      await this.refresh();
    } catch (error) {
      if (signal.aborted || this.#destroyed) return;
      this.#showError(error);
    }
  }

  deliver(event: RendererEvent): void {
    if (this.#destroyed) return;
    if (event.kind === "notice") {
      this.#publish({ ...this.#requiredFrame(), notice: event.message });
      return;
    }
    if (event.kind === "assistant_delta") {
      const pane = this.#chatPane();
      const projection = this.#requiredFrame().workbench.projections[pane];
      const previousSequence = this.#assistantSequences.get(event.message_id) ?? 0;
      if (projection?.kind !== "chat" || event.sequence !== previousSequence + 1) return;
      const message = projection.messages.find((candidate) => candidate.id === event.message_id);
      if (!message || message.state !== "streaming") return;
      this.#assistantSequences.set(event.message_id, event.sequence);
      this.#setProjection(pane, {
        kind: "chat",
        messages: projection.messages.map((candidate) =>
          candidate.id === event.message_id
            ? { ...candidate, markdown: candidate.markdown + event.markdown_delta }
            : candidate,
        ),
      });
    }
  }

  mount(target: RendererTarget, container: HTMLElement): RendererHandle {
    const handle = target.mount(container, {
      frame: this.#requiredFrame(),
      dispatch: (command) => void this.dispatch(command),
    });
    const listener: Listener = (frame) => handle.update(frame);
    this.#listeners.add(listener);
    const unsubscribe = () => this.#listeners.delete(listener);
    return {
      update: (frame) => handle.update(frame),
      deliver: (event) => {
        this.deliver(event);
        handle.deliver(event);
      },
      destroy: () => {
        unsubscribe();
        handle.destroy();
      },
    };
  }

  destroy(): void {
    if (this.#destroyed) return;
    this.#destroyed = true;
    this.#abort.abort();
    this.#listeners.clear();
    this.#assistantSequences.clear();
  }

  async #submitPrompt(content: string, signal: AbortSignal): Promise<void> {
    const pane = this.#chatPane();
    const createdAt = new Date().toISOString();
    this.#appendChat(pane, {
      id: `draft-${crypto.randomUUID()}`,
      role: "user",
      markdown: content,
      state: "complete",
      created_at: createdAt,
    });
    const messageId = `stream-${crypto.randomUUID()}`;
    this.#assistantSequences.set(messageId, 0);
    this.#appendChat(pane, {
      id: messageId,
      role: "assistant",
      markdown: "",
      state: "streaming",
      created_at: createdAt,
    });
    let sequence = 0;
    try {
      const completed = await this.#port.submitPrompt(
        content,
        (markdownDelta) =>
          this.deliver({
            format: rendererEventFormat,
            kind: "assistant_delta",
            message_id: messageId,
            sequence: ++sequence,
            markdown_delta: markdownDelta,
          }),
        signal,
      );
      this.#replaceChat(pane, messageId, completed);
      this.#assistantSequences.delete(messageId);
      await this.refresh();
    } catch (error) {
      this.#markChatFailed(pane, messageId);
      this.#assistantSequences.delete(messageId);
      throw error;
    }
  }

  #showError(error: unknown): void {
    const frame = this.#requiredFrame();
    const message = error instanceof Error ? error.message : String(error);
    const pane = frame.workbench.focused_pane;
    const projection = pane ? frame.workbench.projections[pane] : undefined;
    if (
      pane &&
      error instanceof WorkbenchRefusal &&
      (!projection || projection.kind === "empty" || projection.kind === "refusal")
    ) {
      this.#setProjection(pane, {
        kind: "refusal",
        code: error.code,
        message,
        retryable: error.retryable,
      });
    }
    this.#publish({ ...this.#requiredFrame(), notice: message });
  }

  #clearNotice(): void {
    if (this.#frame?.notice) this.#publish({ ...this.#frame, notice: undefined });
  }

  #requiredFrame(): RendererFrame {
    if (!this.#frame) throw new Error("workbench controller has not started");
    return this.#frame;
  }

  #assertAlive(): void {
    if (this.#destroyed) throw new Error("workbench controller has been destroyed");
  }

  #publish(frame: RendererFrame): void {
    this.#frame = frame;
    for (const listener of this.#listeners) listener(frame);
  }

  #patchWorkbench(patch: Partial<RendererFrame["workbench"]>): void {
    const frame = this.#requiredFrame();
    this.#publish({ ...frame, workbench: { ...frame.workbench, ...patch } });
  }

  #setProjection(pane: string, projection: RendererFrame["workbench"]["projections"][string]): void {
    const frame = this.#requiredFrame();
    this.#patchWorkbench({ projections: { ...frame.workbench.projections, [pane]: projection } });
  }

  #editorPane(path: string): string {
    const pane = this.#requiredFrame().workbench.panes.find(
      (candidate) => candidate.kind === "editor" && candidate.path === path,
    );
    if (!pane) {
      throw new WorkbenchRefusal(
        "renderer.editor_pane_missing",
        "The host did not expose an editor pane for the opened file.",
        true,
      );
    }
    return pane.id;
  }

  #editorProjection(path: string) {
    const pane = this.#requiredFrame().workbench.panes.find(
      (candidate) => candidate.kind === "editor" && candidate.path === path,
    );
    const projection = pane ? this.#requiredFrame().workbench.projections[pane.id] : undefined;
    return projection?.kind === "editor" ? projection : undefined;
  }

  #chatPane(): string {
    const pane = this.#requiredFrame().workbench.panes.find(
      (candidate) => candidate.kind === "chat",
    );
    if (!pane) {
      throw new WorkbenchRefusal(
        "renderer.chat_pane_missing",
        "The host did not expose an agent conversation pane.",
        true,
      );
    }
    return pane.id;
  }

  #appendChat(pane: string, message: ChatMessage): void {
    const current = this.#requiredFrame().workbench.projections[pane];
    const messages = current?.kind === "chat" ? current.messages : [];
    this.#setProjection(pane, { kind: "chat", messages: [...messages, message] });
  }

  #replaceChat(pane: string, id: string, completed: ChatMessage): void {
    const current = this.#requiredFrame().workbench.projections[pane];
    if (current?.kind !== "chat") return;
    this.#setProjection(pane, {
      kind: "chat",
      messages: current.messages.map((message) => (message.id === id ? completed : message)),
    });
  }

  #markChatFailed(pane: string, id: string): void {
    const current = this.#requiredFrame().workbench.projections[pane];
    if (current?.kind !== "chat") return;
    this.#setProjection(pane, {
      kind: "chat",
      messages: current.messages.map((message) =>
        message.id === id ? { ...message, state: "failed" } : message,
      ),
    });
  }
}
