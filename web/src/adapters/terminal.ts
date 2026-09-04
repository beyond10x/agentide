export type TerminalAdapterHandle = {
  write(bytes: Uint8Array): void;
  focus(): void;
  resize(columns: number, rows: number): void;
  destroy(): void;
};

export type TerminalAdapter = {
  mount(
    container: HTMLElement,
    onInput: (data: string) => void,
    onResize: (columns: number, rows: number) => void,
  ): TerminalAdapterHandle;
};

export type GhosttyTerminal = {
  open(container: HTMLElement): void;
  write(data: Uint8Array): void;
  focus(): void;
  resize(columns: number, rows: number): void;
  dispose(): void;
  onData(listener: (data: string) => void): { dispose(): void };
  onResize(listener: (size: { cols: number; rows: number }) => void): { dispose(): void };
};

/** Adapts Ghostty as a leaf renderer; terminal transport remains exclusively host-owned. */
export function ghosttyAdapter(createTerminal: () => GhosttyTerminal): TerminalAdapter {
  return {
    mount(container, onInput, onResize) {
      const terminal = createTerminal();
      terminal.open(container);
      const input = terminal.onData(onInput);
      const resize = terminal.onResize((size) => onResize(size.cols, size.rows));
      return {
        write: (bytes) => terminal.write(bytes),
        focus: () => terminal.focus(),
        resize: (columns, rows) => terminal.resize(columns, rows),
        destroy() { input.dispose(); resize.dispose(); terminal.dispose(); },
      };
    },
  };
}
