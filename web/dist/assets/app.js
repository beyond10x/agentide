import { surfaceProfile, surfaceProfileFormat } from "./generated/surface-profile.js";
const byId = (id) => {
    const element = document.getElementById(id);
    if (!element)
        throw new Error(`missing #${id}`);
    return element;
};
const state = {
    events: [],
    palette: false,
};
const themeVariables = {
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
for (const [role, value] of Object.entries(surfaceProfile.theme.truecolor)) {
    const variable = themeVariables[role];
    if (variable)
        document.documentElement.style.setProperty(variable, value);
}
document.documentElement.dataset.surfaceProfile = surfaceProfileFormat;
async function api(path, init) {
    const response = await fetch(path, {
        headers: { "content-type": "application/json" },
        ...init,
    });
    const value = await response.json();
    if (!response.ok)
        throw new Error(value.message ?? `request failed (${response.status})`);
    return value;
}
async function refresh() {
    [state.snapshot, state.events] = await Promise.all([
        api("/api/snapshot"),
        api("/api/events"),
    ]);
    render();
}
async function call(intent, input = {}) {
    const result = await api(`/api/intents/${intent}/call`, {
        method: "POST",
        body: JSON.stringify({ input }),
    });
    await refresh();
    return result;
}
function render() {
    const snapshot = state.snapshot;
    if (!snapshot)
        return;
    byId("objective").textContent = snapshot.objective;
    byId("session-meta").textContent = `${snapshot.status} · event ${snapshot.cursor} · ${snapshot.session_id.slice(-8)}`;
    byId("files").innerHTML = snapshot.workbench.open_files.length
        ? snapshot.workbench.open_files.map((path) => `<button class="file" data-path="${escape(path)}"><span>◫</span>${escape(path)}</button>`).join("")
        : `<p class="empty">No files open. Press <kbd>O</kbd>.</p>`;
    byId("tabs").innerHTML = snapshot.workbench.panes.map((pane) => {
        const selected = pane.id === snapshot.workbench.focused_pane;
        return `<button class="tab ${selected ? "active" : ""}" data-pane="${escape(pane.id)}"><span>${glyph(pane.kind)}</span>${escape(pane.title)}<i data-close="${escape(pane.id)}">×</i></button>`;
    }).join("");
    const focused = snapshot.workbench.panes.find((pane) => pane.id === snapshot.workbench.focused_pane);
    byId("canvas").innerHTML = focused ? paneView(focused, state.observation ?? snapshot.last_result) : welcome();
    byId("approvals").innerHTML = snapshot.pending_approvals.length
        ? snapshot.pending_approvals.map((plan) => `<article class="approval"><small>exact plan</small><strong>${escape(plan.intent)}</strong><code>${plan.digest.slice(0, 16)}…</code><button data-approve="${plan.digest}">Approve exact plan</button></article>`).join("")
        : `<p class="empty good">No effects waiting for authority.</p>`;
    byId("timeline").innerHTML = state.events.slice(-12).reverse().map((event) => `<li><span>${event.sequence}</span><div><strong>${escape(event.intent ?? event.kind)}</strong><small>${escape(event.kind)} · ${new Date(event.at).toLocaleTimeString()}</small></div></li>`).join("");
    const palette = byId("palette");
    if (state.palette && !palette.open)
        palette.showModal();
    if (!state.palette && palette.open)
        palette.close();
}
function paneView(pane, result) {
    const location = pane.line ? `line ${pane.line}${pane.column ? `:${pane.column}` : ""}` : "workspace view";
    return `<header class="pane-title"><div><span class="eyebrow">${escape(pane.kind)}</span><h2>${escape(pane.title)}</h2></div><span>${escape(location)}</span></header><pre>${escape(result ? JSON.stringify(result, null, 2) : "This virtual pane is durable session state.\nInvoke code_read to attach source observations.")}</pre>`;
}
function welcome() {
    return `<section class="welcome"><div class="mark">A</div><p class="eyebrow">Agent-native workbench</p><h2>The session is the interface.</h2><p>Files, diffs, processes, approvals, agents, and evidence are projections of one replayable event stream.</p><div class="actions"><button data-command="open">Open file <kbd>O</kbd></button><button data-command="diff">Show changes <kbd>D</kbd></button></div></section>`;
}
function glyph(kind) {
    return { editor: "◫", diff: "±", terminal: ">_", timeline: "◷", agents: "◎", approvals: "◇", evidence: "✓" }[kind.toLowerCase()] ?? "□";
}
function escape(value) {
    return value.replace(/[&<>'"]/g, (character) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "'": "&#39;", '"': "&quot;" })[character] ?? character);
}
function openPalette() {
    state.palette = true;
    render();
    requestAnimationFrame(() => byId("command-input").focus());
}
async function runCommand(command) {
    state.palette = false;
    if (command === "palette") {
        state.palette = true;
        render();
        requestAnimationFrame(() => byId("command-input").focus());
        return;
    }
    else if (command === "open") {
        const path = window.prompt("Workspace-relative file path");
        if (path) {
            await call("file_open", { path });
            state.observation = await call("code_read", { path });
        }
    }
    else if (command === "diff") {
        await call("diff_show");
        state.observation = await call("code_changes");
    }
    else if (command === "refresh") {
        await refresh();
    }
    render();
}
document.addEventListener("click", async (event) => {
    const target = event.target;
    const command = target.closest("[data-command]")?.dataset.command;
    const pane = target.closest("[data-pane]")?.dataset.pane;
    const file = target.closest("[data-path]")?.dataset.path;
    const close = target.closest("[data-close]")?.dataset.close;
    const approve = target.closest("[data-approve]")?.dataset.approve;
    try {
        if (command)
            await runCommand(command);
        else if (close)
            await call("pane_close", { pane_id: close });
        else if (pane) {
            const path = state.snapshot?.workbench.panes.find((item) => item.id === pane)?.path;
            await call("pane_focus", { pane_id: pane });
            if (path)
                state.observation = await call("code_read", { path });
            render();
        }
        else if (file) {
            await call("file_open", { path: file });
            state.observation = await call("code_read", { path: file });
            render();
        }
        else if (approve) {
            await api(`/api/approvals/${approve}`, { method: "POST", body: "{}" });
            await refresh();
            byId("notice").textContent = "Exact plan approved. The requesting agent can now retry the same intent input.";
        }
    }
    catch (error) {
        byId("notice").textContent = String(error);
    }
});
document.addEventListener("keydown", async (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        openPalette();
        return;
    }
    if (event.key === "Escape") {
        state.palette = false;
        render();
        return;
    }
    if (event.target instanceof HTMLInputElement)
        return;
    const command = { o: "open", d: "diff", r: "refresh" }[event.key.toLowerCase()];
    if (command)
        await runCommand(command);
});
byId("command-input").addEventListener("keydown", async (event) => {
    if (event.key !== "Enter")
        return;
    const value = event.currentTarget.value.toLowerCase();
    await runCommand(value.includes("diff") ? "diff" : value.includes("refresh") ? "refresh" : "open");
});
refresh().catch((error) => { byId("notice").textContent = String(error); });
byId("surface-profile").textContent = surfaceProfileFormat;
window.setInterval(() => refresh().catch(() => undefined), 2_000);
