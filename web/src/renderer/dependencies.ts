import type { EditorAdapter } from "../adapters/editor";
import type { TerminalAdapter, TerminalAdapterHandle } from "../adapters/terminal";

/** Optional framework-neutral leaves supplied to either renderer target. */
export type RendererAdapters = {
  editor?: EditorAdapter;
  terminal?: TerminalAdapter;
};

export type TerminalRegistration = (
  terminalId: string,
  handle: TerminalAdapterHandle | undefined,
) => void;
