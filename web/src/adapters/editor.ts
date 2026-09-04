import type { EditorDocument } from "../renderer/protocol";

export type EditorAdapterHandle = {
  update(document: EditorDocument): void;
  focus(): void;
  destroy(): void;
};

export type EditorAdapter = {
  mount(
    container: HTMLElement,
    document: EditorDocument,
    onChange: (content: string, version: string) => void,
  ): EditorAdapterHandle;
};

type MonacoModel = { dispose(): void; getValue(): string; setValue(value: string): void };
type MonacoEditor = {
  dispose(): void;
  focus(): void;
  updateOptions(options: { readOnly: boolean }): void;
  onDidChangeModelContent(listener: () => void): { dispose(): void };
};

export type MonacoUri = object;

export type MonacoApi = {
  Uri: { parse(value: string): MonacoUri };
  editor: {
    createModel(value: string, language: string, uri: MonacoUri): MonacoModel;
    create(
      container: HTMLElement,
      options: { model: MonacoModel; readOnly: boolean; automaticLayout: boolean },
    ): MonacoEditor;
    setModelLanguage(model: MonacoModel, language: string): void;
  };
};

/** Adapts Monaco as a leaf renderer; it owns no fetch, persistence, auth, or workspace state. */
export function monacoAdapter(monaco: MonacoApi): EditorAdapter {
  return {
    mount(container, initial, onChange) {
      let document = initial;
      let applyingProjection = false;
      const encodedPath = initial.path.split("/").map(encodeURIComponent).join("/");
      const model = monaco.editor.createModel(
        initial.content,
        initial.language,
        monaco.Uri.parse(`agentide://workspace/${encodedPath}`),
      );
      const editor = monaco.editor.create(container, {
        model,
        readOnly: initial.read_only,
        automaticLayout: true,
      });
      const changes = editor.onDidChangeModelContent(() => {
        if (!applyingProjection) onChange(model.getValue(), document.version);
      });
      return {
        update(next) {
          if (next.path !== document.path) {
            throw new Error("renderer.editor_path_changed: mount a new editor for a new path");
          }
          if (next.language !== document.language) {
            monaco.editor.setModelLanguage(model, next.language);
          }
          editor.updateOptions({ readOnly: next.read_only });
          document = next;
          if (model.getValue() !== next.content) {
            applyingProjection = true;
            try {
              model.setValue(next.content);
            } finally {
              applyingProjection = false;
            }
          }
        },
        focus: () => editor.focus(),
        destroy() { changes.dispose(); editor.dispose(); model.dispose(); },
      };
    },
  };
}
