function escape(value: string): string {
  return value.replace(/[&<>"']/g, (character) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  })[character] ?? character);
}

function inline(value: string): string {
  return escape(value)
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(/\*([^*]+)\*/g, "<em>$1</em>")
    .replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2" rel="noreferrer noopener" target="_blank">$1</a>');
}

/** A dependency-free, streaming-safe Markdown subset. Every source byte is escaped first. */
export function renderMarkdown(markdown: string): string {
  const output: string[] = [];
  let code = false;
  let list = false;
  for (const line of markdown.replace(/\r\n?/g, "\n").split("\n")) {
    if (line.startsWith("```")) {
      if (list) { output.push("</ul>"); list = false; }
      output.push(code ? "</code></pre>" : "<pre><code>");
      code = !code;
      continue;
    }
    if (code) { output.push(`${escape(line)}\n`); continue; }
    const item = /^\s*[-*]\s+(.+)$/.exec(line);
    if (item) {
      if (!list) { output.push("<ul>"); list = true; }
      output.push(`<li>${inline(item[1] ?? "")}</li>`);
      continue;
    }
    if (list) { output.push("</ul>"); list = false; }
    const heading = /^(#{1,4})\s+(.+)$/.exec(line);
    if (heading) {
      output.push(`<h${heading[1]?.length}>${inline(heading[2] ?? "")}</h${heading[1]?.length}>`);
    } else if (line.startsWith("> ")) {
      output.push(`<blockquote>${inline(line.slice(2))}</blockquote>`);
    } else if (line.trim()) {
      output.push(`<p>${inline(line)}</p>`);
    }
  }
  if (list) output.push("</ul>");
  if (code) output.push("</code></pre>");
  return output.join("");
}
