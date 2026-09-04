import {
  createApp,
  defineComponent,
  h,
  onBeforeUnmount,
  onMounted,
  ref,
  shallowRef,
  watch,
  type CSSProperties,
  type PropType,
  type VNode,
} from "vue";
import type { EditorAdapter, EditorAdapterHandle } from "../adapters/editor";
import type { TerminalAdapter, TerminalAdapterHandle } from "../adapters/terminal";
import type { RendererAdapters, TerminalRegistration } from "./dependencies";
import {
  action,
  rendererActionFormat,
  rendererEventFormat,
  rendererFrameFormat,
  rendererProtocolFormat,
  type RendererAction,
  type RendererActionInput,
  type EditorDocument,
  type PaneProjection,
  type RendererFrame,
  type RendererHandle,
  type RendererOptions,
  type RendererTarget,
} from "./protocol";
import { defaultTerminalSize, focusedPane, glyph, installTheme, uninstallTheme } from "./shared";
import { renderMarkdown } from "./markdown";

/**
 * Pure Vue composition shell for hosts that provide richer workspace projections.
 *
 * The shell owns only layout and accessibility landmarks. All observations, effects,
 * transport, persistence, and authorization remain the responsibility of the host.
 */
export const AgentIdeVueWorkbench = defineComponent({
  name: "AgentIdeVueWorkbench",
  props: {
    bottomOpen: { type: Boolean, default: false },
    bottomHeight: { type: Number, default: 320 },
    explorerLabel: { type: String, default: "Workspace explorer" },
    inspectorLabel: { type: String, default: "Workspace inspector" },
    centerLabel: { type: String, default: "Workspace canvas" },
    bottomLabel: { type: String, default: "Workspace bottom panel" },
  },
  setup(props, { slots }) {
    return () => {
      const style = {
        "--terminal-height": props.bottomOpen
          ? `${String(Math.max(0, props.bottomHeight))}px`
          : "2.45rem",
      } as CSSProperties;
      return h(
        "main",
        {
          class: "agentide-root agentide-vue-workbench",
          "data-agentide-renderer": "vue",
          "data-agentide-renderer-protocol": rendererProtocolFormat,
        },
        [
          h("header", { class: "workbench-titlebar" }, slots.titlebar?.()),
          slots.notices?.(),
          h(
            "div",
            {
              class: ["workbench-grid", { "terminal-collapsed": !props.bottomOpen }],
              style,
            },
            [
              h(
                "aside",
                { class: "workbench-explorer", "aria-label": props.explorerLabel },
                slots.explorer?.(),
              ),
              h(
                "section",
                { class: "workbench-center", "aria-label": props.centerLabel },
                slots.center?.(),
              ),
              h(
                "aside",
                { class: "workbench-inspector", "aria-label": props.inspectorLabel },
                slots.inspector?.(),
              ),
              h(
                "section",
                {
                  class: ["workbench-terminal", { collapsed: !props.bottomOpen }],
                  "aria-label": props.bottomLabel,
                },
                slots.bottom?.(),
              ),
            ],
          ),
          slots.overlay?.(),
        ],
      );
    };
  },
});

const EditorLeaf = defineComponent({
  name: "AgentIdeEditorLeaf",
  props: {
    adapter: { type: Object as PropType<EditorAdapter>, required: true },
    document: { type: Object as PropType<EditorDocument>, required: true },
    dispatch: { type: Function as PropType<(action: RendererAction) => void>, required: true },
  },
  setup(props) {
    const element = ref<HTMLElement>();
    let handle: EditorAdapterHandle | undefined;
    onMounted(() => {
      if (!element.value) return;
      handle = props.adapter.mount(element.value, props.document, (content, version) =>
        props.dispatch(action({
          kind: "edit_file",
          path: props.document.path,
          content,
          version,
        })),
      );
    });
    watch(() => props.document, (document) => handle?.update(document));
    onBeforeUnmount(() => handle?.destroy());
    return () => h("div", {
      ref: element,
      class: "editor-leaf",
      "data-path": props.document.path,
    });
  },
});

const TerminalLeaf = defineComponent({
  name: "AgentIdeTerminalLeaf",
  props: {
    adapter: { type: Object as PropType<TerminalAdapter>, required: true },
    projection: {
      type: Object as PropType<Extract<PaneProjection, { kind: "terminal" }>>,
      required: true,
    },
    dispatch: { type: Function as PropType<(action: RendererAction) => void>, required: true },
    register: { type: Function as PropType<TerminalRegistration>, required: true },
  },
  setup(props) {
    const element = ref<HTMLElement>();
    let handle: TerminalAdapterHandle | undefined;
    let terminalId = props.projection.terminal_id;
    const unmount = () => {
      props.register(terminalId, undefined);
      handle?.destroy();
      handle = undefined;
    };
    const mount = () => {
      if (!element.value) return;
      terminalId = props.projection.terminal_id;
      handle = props.adapter.mount(
        element.value,
        (data) => props.dispatch(action({ kind: "terminal_input", terminal_id: terminalId, data })),
        (columns, rows) => props.dispatch(action({
          kind: "terminal_resize",
          terminal_id: terminalId,
          columns,
          rows,
        })),
      );
      handle.resize(props.projection.columns, props.projection.rows);
      props.register(terminalId, handle);
    };
    onMounted(mount);
    watch(() => props.projection, (projection, previous) => {
      if (projection.terminal_id !== previous.terminal_id) {
        unmount();
        mount();
      } else if (projection.columns !== previous.columns || projection.rows !== previous.rows) {
        handle?.resize(projection.columns, projection.rows);
      }
    });
    onBeforeUnmount(unmount);
    return () => h("div", {
      ref: element,
      class: "terminal-leaf",
      "data-terminal-id": props.projection.terminal_id,
    });
  },
});

const AgentIdeVueSurface = defineComponent({
  name: "AgentIdeVueSurface",
  props: {
    frame: { type: Object as PropType<RendererFrame>, required: true },
    dispatch: {
      type: Function as PropType<(action: RendererAction) => void>,
      required: true,
    },
    adapters: {
      type: Object as PropType<RendererAdapters>,
      default: () => ({} satisfies RendererAdapters),
    },
    registerTerminal: {
      type: Function as PropType<TerminalRegistration>,
      default: () => undefined,
    },
  },
  setup(props) {
    let promptDraft = "";
    const send = (value: RendererActionInput) => props.dispatch(action(value));
    const button = (
      label: string,
      onClick: () => void,
      className?: string,
      children: VNode[] = [],
    ) => h("button", { class: className, type: "button", onClick }, [label, ...children]);
    const projectionNode = (projection: PaneProjection | undefined): VNode => {
      if (!projection) return h("p", { class: "empty" }, "Select an observation to populate this pane.");
      switch (projection.kind) {
        case "editor":
          return props.adapters.editor
            ? h(EditorLeaf, {
                key: projection.document.path,
                adapter: props.adapters.editor,
                document: projection.document,
                dispatch: props.dispatch,
              })
            : h("pre", projection.document.content);
        case "diff":
          return h("section", { class: "change-list" }, [
            h("p", [h("code", projection.baseline_commit), projection.truncated ? " · truncated" : ""]),
            ...projection.changes.map((change) => h("article", [
              h("strong", change.status),
              ` ${change.path}`,
              change.patch ? h("pre", change.patch) : undefined,
            ])),
          ]);
        case "chat":
          return h("section", { class: "agent-chat" }, [
            h("div", { class: "chat-transcript", "aria-live": "polite" },
              projection.messages.map((message) =>
                h(
                  "article",
                  {
                    key: message.id,
                    class: ["chat-message", message.role],
                    "data-state": message.state,
                  },
                  [
                    h("small", message.role),
                    h("div", { class: "markdown", innerHTML: renderMarkdown(message.markdown) }),
                  ],
                ),
              ),
            ),
            h("form", {
              key: "prompt",
              onSubmit: (event: Event) => {
                event.preventDefault();
                const content = promptDraft.trim();
                if (content) {
                  promptDraft = "";
                  send({ kind: "submit_prompt", content });
                }
              },
            }, [
              h("textarea", {
                name: "prompt",
                value: promptDraft,
                "aria-label": "Message the agent",
                placeholder: "Ask the agent…",
                onInput: (event: Event) => {
                  promptDraft = (event.currentTarget as HTMLTextAreaElement).value;
                },
              }),
              h("button", { type: "submit" }, "Send"),
            ]),
          ]);
        case "terminal":
          return props.adapters.terminal
            ? h(TerminalLeaf, {
                key: projection.terminal_id,
                adapter: props.adapters.terminal,
                projection,
                dispatch: props.dispatch,
                register: props.registerTerminal ?? (() => undefined),
              })
            : h("p", { class: "empty" }, `Terminal ${projection.state} · ${projection.columns}×${projection.rows}`);
        case "refusal":
          return h("section", { class: "pane-refusal", role: "alert" }, [
            h("strong", projection.code),
            h("p", projection.message),
            projection.retryable ? h("small", "Retryable") : undefined,
          ]);
        case "empty":
          return h("p", { class: "empty" }, projection.message);
      }
    };
    const saveFocusedDraft = (event: KeyboardEvent, frame: RendererFrame) => {
      if (!(event.metaKey || event.ctrlKey) || event.key.toLowerCase() !== "s") return;
      const pane = focusedPane(frame);
      const projection = pane ? frame.workbench.projections[pane.id] : undefined;
      if (projection?.kind !== "editor" || !projection.document.dirty) return;
      event.preventDefault();
      send({
        kind: "save_file",
        path: projection.document.path,
        content: projection.document.content,
        version: projection.document.version,
      });
    };
    return () => {
      const frame = props.frame;
      const pane = focusedPane(frame);
      const projection = pane ? frame.workbench.projections[pane.id] : undefined;
      const navigateTo = (kind: "editor" | "chat" | "terminal") => {
        const target = frame.workbench.panes.find((candidate) => candidate.kind === kind);
        if (target) {
          send({ kind: "focus_pane", pane_id: target.id });
        } else if (kind === "editor") {
          send({ kind: "load_tree", path: "" });
        } else if (kind === "terminal") {
          send({ kind: "open_terminal", ...defaultTerminalSize });
        }
      };
      const navigationButton = (
        label: string,
        accessibleLabel: string,
        onClick: () => void,
        selected: boolean,
      ) => h("button", {
        class: { selected },
        type: "button",
        title: accessibleLabel,
        "aria-label": accessibleLabel,
        onClick,
      }, label);
      return h("div", {
        class: "agentide-vue-target",
        "data-agentide-renderer": "vue",
        "data-agentide-renderer-protocol": rendererProtocolFormat,
        onKeydown: (event: KeyboardEvent) => saveFocusedDraft(event, frame),
      }, [
        h("header", { class: "topbar" }, [
          h("a", { class: "brand", href: "/" }, [h("span", "A"), "AgentIDE ", h("em", "vue")]),
          h("div", { class: "session" }, [
            h("strong", frame.session.objective),
            h("small", `${frame.session.status} · event ${frame.session.cursor} · ${frame.session.id.slice(-8)}`),
          ]),
          button("Command ", () => undefined, "command", [h("kbd", "⌘ K")]),
        ]),
        h("main", { class: "shell" }, [
          h("aside", { class: "rail" }, [
            h("nav", { "aria-label": "Session views" }, [
              navigationButton("◫", "Workspace explorer", () => navigateTo("editor"), pane?.kind === "editor"),
              navigationButton("±", "Workspace changes", () => send({ kind: "show_diff" }), pane?.kind === "diff"),
              navigationButton("◎", "Agent chat", () => navigateTo("chat"), pane?.kind === "chat"),
              navigationButton("›_", "Terminal", () => navigateTo("terminal"), pane?.kind === "terminal"),
            ]),
            h("button", { class: "settings", type: "button", title: "Bindings", "aria-label": "Bindings" }, "⌘"),
          ]),
          h("aside", { class: "explorer" }, [
            h("div", { class: "section-title" }, [
              h("span", "WORKSPACE"),
              h(
                "button",
                {
                  type: "button",
                  "aria-label": "Refresh workspace",
                  onClick: () => send({ kind: "load_tree", path: "" }),
                },
                "↻",
              ),
            ]),
            frame.workbench.tree
              ? h("div", [
                  ...frame.workbench.tree.entries.map((entry) =>
                    button(
                      entry.name,
                      () => entry.kind === "directory"
                        ? send({ kind: "load_tree", path: entry.path })
                        : send({ kind: "open_file", path: entry.path }),
                      "file",
                      [h("span", entry.kind === "directory" ? "▸" : "◫")],
                    ),
                  ),
                  frame.workbench.tree.next_cursor
                    ? button("Load more…", () => send({
                        kind: "load_tree",
                        path: frame.workbench.tree?.root ?? "",
                        cursor: frame.workbench.tree?.next_cursor,
                      }), "file")
                    : undefined,
                ])
              : h("p", { class: "empty" }, [
                  button("Load workspace", () => send({ kind: "load_tree", path: "" })),
                ]),
            h("div", { class: "section-title" }, [h("span", "OPEN FILES")]),
            frame.workbench.open_files.length
              ? h(
                  "div",
                  frame.workbench.open_files.map((path) =>
                    button(path, () => send({ kind: "open_file", path }), "file", [h("span", "◫")]),
                  ),
                )
              : h("p", { class: "empty" }, "No files open."),
            h("div", { class: "section-title" }, [h("span", "APPROVALS")]),
            frame.pending_approvals.length
              ? h(
                  "div",
                  frame.pending_approvals.map((plan) =>
                    h("article", { class: "approval" }, [
                      h("small", "exact plan"),
                      h("strong", plan.intent),
                      h("code", `${plan.digest.slice(0, 16)}…`),
                      button("Approve exact plan", () =>
                        send({ kind: "approve", plan_digest: plan.digest }),
                      ),
                      button(
                        "Deny",
                        () => send({ kind: "deny", plan_digest: plan.digest }),
                        "deny",
                      ),
                    ]),
                  ),
                )
              : h("p", { class: "empty good" }, "No effects waiting for authority."),
          ]),
          h("section", { class: "workbench" }, [
            h(
              "div",
              { class: "tabs" },
              frame.workbench.panes.map((item) =>
                h(
                  "div",
                  {
                    class: ["tab", { active: item.id === frame.workbench.focused_pane }],
                  },
                  [
                    h(
                      "button",
                      {
                        class: "tab-focus",
                        type: "button",
                        "aria-label": `Focus ${item.title}`,
                        onClick: () => send({ kind: "focus_pane", pane_id: item.id }),
                      },
                      [h("span", glyph(item.kind)), item.title],
                    ),
                    h(
                      "button",
                      {
                        class: "tab-close",
                        type: "button",
                        "aria-label": `Close ${item.title}`,
                        onClick: () => send({ kind: "close_pane", pane_id: item.id }),
                      },
                      "×",
                    ),
                  ],
                ),
              ),
            ),
            h(
              "div",
              { class: "canvas" },
              pane
                ? [
                    h("header", { class: "pane-title" }, [
                      h("div", [
                        h("span", { class: "eyebrow" }, pane.kind),
                        h("h2", [
                          pane.title,
                          projection?.kind === "editor" && projection.document.dirty
                            ? h("span", { class: "pane-dirty", "aria-label": "Unsaved changes" }, " ●")
                            : undefined,
                        ]),
                      ]),
                      h("span", pane.line ? `line ${pane.line}` : "workspace view"),
                    ]),
                    projectionNode(projection),
                  ]
                : [
                    h("section", { class: "welcome" }, [
                      h("div", { class: "mark" }, "A"),
                      h("p", { class: "eyebrow" }, "Agent-native workbench"),
                      h("h2", "The session is the interface."),
                      h("p", "Files, diffs, processes, approvals, agents, and evidence are projections of one replayable event stream."),
                    ]),
                  ],
            ),
            h("div", { class: "notice", role: "status" }, frame.notice ?? ""),
          ]),
          h("aside", { class: "context" }, [
            h("div", { class: "section-title" }, [
              h("span", "SESSION TIMELINE"),
              h(
                "button",
                {
                  type: "button",
                  "aria-label": "Refresh session timeline",
                  onClick: () => send({ kind: "refresh" }),
                },
                "↻",
              ),
            ]),
            h(
              "ol",
              frame.activity.slice(-12).reverse().map((event) =>
                h("li", [
                  h("span", String(event.sequence)),
                  h("div", [
                    h("strong", event.intent ?? event.kind),
                    h("small", `${event.kind} · ${new Date(event.at).toLocaleTimeString()}`),
                  ]),
                ]),
              ),
            ),
            h("div", { class: "section-title" }, [h("span", "CONTEXT")]),
            frame.context_pins.length
              ? h(
                  "div",
                  frame.context_pins.map((pin) =>
                    h("article", { class: "context-pin" }, [
                      h("div", [h("strong", pin.label), h("small", pin.source)]),
                      h(
                        "button",
                        {
                          type: "button",
                          "aria-label": `Remove ${pin.label} from context`,
                          onClick: () => send({ kind: "remove_context_pin", pin_id: pin.id }),
                        },
                        "×",
                      ),
                    ]),
                  ),
                )
              : h("p", { class: "empty" }, "No pinned context."),
            h("div", { class: "section-title" }, [h("span", "CAPABILITIES")]),
            frame.grants.length
              ? h(
                  "div",
                  frame.grants.map((grant) =>
                    h("p", { class: "grant", "data-state": grant.state }, grant.capability),
                  ),
                )
              : h("p", { class: "empty" }, "No active capabilities."),
          ]),
        ]),
        h("footer", [
          h("span", [h("b", "●"), " Substrate boundary"]),
          h("span", "Semantic actions · exact approvals · durable replay"),
          h("span", rendererFrameFormat),
        ]),
      ]);
    };
  },
});

/** Builds a Vue target over the same optional Monaco and Ghostty leaves as Vanilla DOM. */
export function createVueRenderer(adapters: RendererAdapters = {}): RendererTarget {
  return {
    manifest: {
      format: rendererProtocolFormat,
      id: "vue",
      framework: "vue-3",
      frame_format: rendererFrameFormat,
      event_format: rendererEventFormat,
      action_format: rendererActionFormat,
    },
    mount(container: HTMLElement, options: RendererOptions): RendererHandle {
      const frame = shallowRef(options.frame);
      const terminals = new Map<string, TerminalAdapterHandle>();
      const terminalSequences = new Map<string, number>();
      const registerTerminal: TerminalRegistration = (terminalId, handle) => {
        if (handle) {
          terminals.set(terminalId, handle);
          terminalSequences.set(terminalId, 0);
        } else {
          terminals.delete(terminalId);
          terminalSequences.delete(terminalId);
        }
      };
      installTheme(container);
      const Root = defineComponent(() => () =>
        h(AgentIdeVueSurface, {
          frame: frame.value,
          dispatch: options.dispatch,
          adapters,
          registerTerminal,
        }),
      );
      const app = createApp(Root);
      app.mount(container);
      return {
        update(next) {
          frame.value = next;
        },
        deliver(event) {
          if (event.kind === "notice") {
            frame.value = { ...frame.value, notice: event.message };
          } else if (event.kind === "terminal_output") {
            const previous = terminalSequences.get(event.terminal_id) ?? 0;
            if (event.sequence === previous + 1) {
              terminalSequences.set(event.terminal_id, event.sequence);
              terminals.get(event.terminal_id)?.write(event.bytes);
            }
          }
        },
        destroy() {
          app.unmount();
          terminals.clear();
          terminalSequences.clear();
          uninstallTheme(container);
        },
      };
    },
  };
}

export const vueRenderer = createVueRenderer();

export { AgentIdeVueSurface };
