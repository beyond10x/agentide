import {
  createApp,
  defineComponent,
  h,
  shallowRef,
  type CSSProperties,
  type PropType,
  type VNode,
} from "vue";
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
  type RendererOptions,
  type RendererTarget,
} from "./protocol";
import { focusedPane, glyph, installTheme, paneObservation } from "./shared";

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
          class: "agentide-vue-workbench",
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

const AgentIdeVueSurface = defineComponent({
  name: "AgentIdeVueSurface",
  props: {
    frame: { type: Object as PropType<RendererFrame>, required: true },
    dispatch: {
      type: Function as PropType<(action: RendererAction) => void>,
      required: true,
    },
  },
  setup(props) {
    const send = (value: RendererActionInput) => props.dispatch(action(value));
    const button = (
      label: string,
      onClick: () => void,
      className?: string,
      children: VNode[] = [],
    ) => h("button", { class: className, type: "button", onClick }, [label, ...children]);
    return () => {
      const frame = props.frame;
      const pane = focusedPane(frame);
      return h("div", { class: "agentide-vue-target" }, [
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
              button("◫", () => undefined, "selected"),
              button("±", () => send({ kind: "show_diff" })),
              button("›_", () => undefined),
              button("◎", () => undefined),
              button("✓", () => undefined),
            ]),
            button("⌘", () => undefined, "settings"),
          ]),
          h("aside", { class: "explorer" }, [
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
                  "button",
                  {
                    class: ["tab", { active: item.id === frame.workbench.focused_pane }],
                    type: "button",
                    onClick: () => send({ kind: "focus_pane", pane_id: item.id }),
                  },
                  [
                    h("span", glyph(item.kind)),
                    item.title,
                    h(
                      "i",
                      { onClick: (event: Event) => { event.stopPropagation(); send({ kind: "close_pane", pane_id: item.id }); } },
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
                      h("div", [h("span", { class: "eyebrow" }, pane.kind), h("h2", pane.title)]),
                      h("span", pane.line ? `line ${pane.line}` : "workspace view"),
                    ]),
                    h("pre", paneObservation(frame, pane)),
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
              button("↻", () => send({ kind: "refresh" })),
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

export const vueRenderer: RendererTarget = {
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
    installTheme(container);
    const Root = defineComponent(() => () =>
      h(AgentIdeVueSurface, { frame: frame.value, dispatch: options.dispatch }),
    );
    const app = createApp(Root);
    app.mount(container);
    return {
      update(next) {
        frame.value = next;
      },
      deliver(event) {
        if (event.kind === "notice") frame.value = { ...frame.value, notice: event.message };
      },
      destroy() {
        app.unmount();
      },
    };
  },
};

export { AgentIdeVueSurface };
