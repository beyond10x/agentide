import { surfaceProfile, surfaceProfileFormat } from "../generated/surface-profile";
import type { Pane, RendererFrame } from "./protocol";

const themeVariables: Record<string, string> = {
  background: "--bg",
  panel: "--panel",
  raised: "--raised",
  line: "--line",
  muted: "--muted",
  text: "--text",
  accent: "--cyan",
  warning: "--amber",
  danger: "--red",
};

export function installTheme(root: HTMLElement): void {
  for (const [role, value] of Object.entries(surfaceProfile.theme.truecolor)) {
    const variable = themeVariables[role];
    if (variable) root.style.setProperty(variable, value);
  }
  root.dataset.surfaceProfile = surfaceProfileFormat;
}

export function focusedPane(frame: RendererFrame): Pane | undefined {
  return frame.workbench.panes.find((pane) => pane.id === frame.workbench.focused_pane);
}

export function glyph(kind: string): string {
  return (
    {
      editor: "◫",
      diff: "±",
      terminal: ">_",
      timeline: "◷",
      agents: "◎",
      approvals: "◇",
      evidence: "✓",
    } as Record<string, string>
  )[kind.toLowerCase()] ?? "□";
}

export function paneObservation(frame: RendererFrame, pane: Pane): string {
  if (frame.observation !== undefined) return JSON.stringify(frame.observation, null, 2);
  if (pane.path) return `Open ${pane.path} to resolve its current saved contents.`;
  return "This virtual pane is durable session state.";
}
