import { describe, expect, it, vi } from "vitest";
import { monacoAdapter, type MonacoApi } from "../src/adapters/editor";
import { ghosttyAdapter, type GhosttyTerminal } from "../src/adapters/terminal";

describe("monacoAdapter", () => {
  it("keeps transport out, applies host projections silently, and forwards only operator edits", () => {
    let content = "one";
    let change: (() => void) | undefined;
    const setLanguage = vi.fn();
    const updateOptions = vi.fn();
    const parse = vi.fn(() => ({}));
    const model = {
      dispose: vi.fn(),
      getValue: () => content,
      setValue: vi.fn((value: string) => {
        content = value;
        change?.();
      }),
    };
    const editor = {
      dispose: vi.fn(),
      focus: vi.fn(),
      updateOptions,
      onDidChangeModelContent(listener: () => void) {
        change = listener;
        return { dispose: vi.fn() };
      },
    };
    const monaco: MonacoApi = {
      Uri: { parse },
      editor: {
        createModel: vi.fn(() => model),
        create: vi.fn(() => editor),
        setModelLanguage: setLanguage,
      },
    };
    const onChange = vi.fn();
    const handle = monacoAdapter(monaco).mount(
      document.createElement("div"),
      {
        path: "docs/a #1.md",
        language: "markdown",
        content,
        version: "v1",
        read_only: false,
        dirty: false,
      },
      onChange,
    );

    handle.update({
      path: "docs/a #1.md",
      language: "plaintext",
      content: "two",
      version: "v2",
      read_only: true,
      dirty: false,
    });
    expect(onChange).not.toHaveBeenCalled();
    expect(parse).toHaveBeenCalledWith("agentide://workspace/docs/a%20%231.md");
    expect(setLanguage).toHaveBeenCalledWith(model, "plaintext");
    expect(updateOptions).toHaveBeenCalledWith({ readOnly: true });

    content = "operator edit";
    change?.();
    expect(onChange).toHaveBeenCalledWith("operator edit", "v2");
    handle.destroy();
    expect(editor.dispose).toHaveBeenCalledOnce();
    expect(model.dispose).toHaveBeenCalledOnce();
  });
});

describe("ghosttyAdapter", () => {
  it("forwards semantic input and resize events and tears down every subscription", () => {
    let input: ((data: string) => void) | undefined;
    let resize: ((size: { cols: number; rows: number }) => void) | undefined;
    const inputDispose = vi.fn();
    const resizeDispose = vi.fn();
    const terminal: GhosttyTerminal = {
      open: vi.fn(),
      write: vi.fn(),
      focus: vi.fn(),
      resize: vi.fn(),
      dispose: vi.fn(),
      onData(listener) {
        input = listener;
        return { dispose: inputDispose };
      },
      onResize(listener) {
        resize = listener;
        return { dispose: resizeDispose };
      },
    };
    const onInput = vi.fn();
    const onResize = vi.fn();
    const handle = ghosttyAdapter(() => terminal).mount(
      document.createElement("div"),
      onInput,
      onResize,
    );
    input?.("ls\n");
    resize?.({ cols: 120, rows: 40 });
    handle.write(new Uint8Array([65]));
    expect(onInput).toHaveBeenCalledWith("ls\n");
    expect(onResize).toHaveBeenCalledWith(120, 40);
    expect(terminal.write).toHaveBeenCalledWith(new Uint8Array([65]));
    handle.destroy();
    expect(inputDispose).toHaveBeenCalledOnce();
    expect(resizeDispose).toHaveBeenCalledOnce();
    expect(terminal.dispose).toHaveBeenCalledOnce();
  });
});
