import { marked } from "marked";

marked.setOptions({
  gfm: true,
  breaks: true,
});

export function renderMarkdown(rawText: string): string {
  if (!rawText) return "";

  let processed = rawText;

  processed = processed.replace(
    /^>\s*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION|DANGER|QUOTE)\][ \t]*(.*)$/gim,
    (_, type: string, header: string) => {
      const lower = type.toLowerCase();
      const title = header.trim() || type.toUpperCase();
      return `<div class="md-callout md-callout-${lower}"><div class="callout-head"><span class="callout-badge">${title}</span></div><div class="callout-body">`;
    }
  );

  processed = processed.replace(/==([^=\n]+)==/g, `<mark class="md-highlight">$1</mark>`);
  processed = processed.replace(/\|\|([^|\n]+)\|\|/g, `<span class="md-spoiler" title="Спойлер: нажать для просмотра">$1</span>`);
  processed = processed.replace(/~([^~\n]+)~/g, `<sub>$1</sub>`);
  processed = processed.replace(/\^([^\^\n]+)\^/g, `<sup>$1</sup>`);
  processed = processed.replace(/\[\[kbd:([^\]]+)\]\]/g, `<kbd class="md-kbd">$1</kbd>`);

  processed = processed.replace(/\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g, (_, target: string, alias?: string) => {
    const text = alias ? alias.trim() : target.trim();
    return `<span class="md-wikilink" data-entity="${target.trim()}"><span class="wiki-tag-symbol">◈</span> ${text}</span>`;
  });

  processed = processed.replace(/(^|[^\w])@([a-zA-Zа-яА-ЯёЁ0-9_-]+)/g, `$1<span class="md-mention">@$2</span>`);

  processed = processed.replace(/^-\s*\[([ xX])\]\s*(.*)$/gm, (_, check: string, label: string) => {
    const isChecked = check.toLowerCase() === "x";
    return `<div class="md-task-item"><input type="checkbox" disabled ${isChecked ? "checked" : ""} class="task-check" /><span>${label}</span></div>`;
  });

  processed = processed.replace(/(^|\n)(—\s*[^<\n]+)/g, `$1<div class="md-dialogue-line">$2</div>`);

  let parsed = marked.parse(processed) as string;

  parsed = parsed.replace(/<div class="md-callout/g, `</div></div><div class="md-callout`);
  if (parsed.startsWith("</div></div>")) {
    parsed = parsed.slice(12);
  }

  return parsed;
}