import { surfaceProfile, surfaceProfileFormat } from "../generated/surface-profile";
import type { Pane, PaneProjection, RendererFrame } from "./protocol";

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

/** Initial dimensions requested before a terminal leaf can report its measured size. */
export const defaultTerminalSize = { columns: 120, rows: 30 } as const;

export function installTheme(root: HTMLElement): void {
  root.classList.add("agentide-root");
  for (const [role, value] of Object.entries(surfaceProfile.theme.truecolor)) {
    const variable = themeVariables[role];
    if (variable) root.style.setProperty(variable, value);
  }
  root.dataset.surfaceProfile = surfaceProfileFormat;
}

export function uninstallTheme(root: HTMLElement): void {
  root.classList.remove("agentide-root");
  delete root.dataset.surfaceProfile;
  for (const variable of Object.values(themeVariables)) root.style.removeProperty(variable);
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
      chat: "◉",
      timeline: "◷",
      agents: "◎",
      approvals: "◇",
      evidence: "✓",
    } as Record<string, string>
  )[kind.toLowerCase()] ?? "□";
}

export function paneProjection(frame: RendererFrame, pane: Pane): PaneProjection | undefined {
  return frame.workbench.projections[pane.id];
}
