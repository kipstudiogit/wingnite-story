// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { marked } from "marked";

marked.setOptions({
  gfm: true,
  breaks: true,
});

function escapeHtml(raw: string): string {
  return raw
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function renderInlineExtensions(html: string): string {
  const protectedBlocks: string[] = [];
  let masked = html.replace(/<(pre|code)[\s\S]*?<\/\1>/gi, (match) => {
    const token = `@@WINGNITE_CODE_${protectedBlocks.length}@@`;
    protectedBlocks.push(match);
    return token;
  });

  masked = masked.replace(/==([^=\n]+)==/g, `<mark class="md-highlight">$1</mark>`);
  masked = masked.replace(/\|\|([^|\n]+)\|\|/g, `<span class="md-spoiler">$1</span>`);
  masked = masked.replace(/(^|[^~])~([^~\s][^~\n]*)~(?!~)/g, `$1<sub>$2</sub>`);
  masked = masked.replace(/\^([^\^\s][^\^\n]*)\^/g, `<sup>$1</sup>`);
  masked = masked.replace(/\[\[kbd:([^\]]+)\]\]/g, `<kbd class="md-kbd">$1</kbd>`);
  masked = masked.replace(/\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g, (_, target: string, alias?: string) => {
    const cleanTarget = escapeHtml(target.trim());
    const label = escapeHtml((alias ?? target).trim());
    return `<span class="md-wikilink" data-entity="${cleanTarget}"><span class="wiki-tag-symbol">◈</span> ${label}</span>`;
  });
  masked = masked.replace(/(^|[^\w/@])@([a-zA-Zа-яА-ЯёЁäöüÄÖÜßñÑáéíóúÁÉÍÓÚ0-9_-]+)/g, `$1<span class="md-mention">@$2</span>`);
  masked = masked.replace(/<p>(—\s*[\s\S]*?)<\/p>/g, `<p class="md-dialogue-line">$1</p>`);

  return masked.replace(/@@WINGNITE_CODE_(\d+)@@/g, (_, idx: string) => protectedBlocks[Number(idx)] ?? "");
}

function extractCallouts(source: string): { text: string; callouts: string[] } {
  const callouts: string[] = [];
  const lines = source.split("\n");
  const out: string[] = [];
  let i = 0;

  while (i < lines.length) {
    const headerMatch = lines[i].match(/^>\s*\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION|DANGER|QUOTE)\][ \t]*(.*)$/i);
    if (headerMatch) {
      const calloutType = headerMatch[1].toLowerCase();
      const calloutTitle = escapeHtml(headerMatch[2].trim() || headerMatch[1].toUpperCase());
      const bodyLines: string[] = [];
      i++;
      while (i < lines.length && /^>\s?/.test(lines[i])) {
        bodyLines.push(lines[i].replace(/^>\s?/, ""));
        i++;
      }
      const bodyRaw = bodyLines.join("\n").trim();
      const parsedBody = renderInlineExtensions(marked.parse(bodyRaw) as string);
      const token = `\n\n@@WINGNITE_CALLOUT_${callouts.length}@@\n\n`;
      callouts.push(
        `<div class="md-callout md-callout-${calloutType}"><div class="callout-head"><span class="callout-badge">${calloutTitle}</span></div><div class="callout-body">${parsedBody}</div></div>`
      );
      out.push(token);
      continue;
    }
    out.push(lines[i]);
    i++;
  }

  return { text: out.join("\n"), callouts };
}

export function renderMarkdown(rawText: string): string {
  if (!rawText) return "";

  const { text: withoutCallouts, callouts } = extractCallouts(rawText);

  let taskCounter = 0;
  const withTasks = withoutCallouts.replace(/^-\s*\[([ xX])\]\s*(.*)$/gm, (_, check: string, label: string) => {
    const isChecked = check.toLowerCase() === "x";
    const currentIdx = taskCounter++;
    const renderedLabel = renderInlineExtensions(marked.parseInline(label) as string);
    return `<div class="md-task-item" data-task-index="${currentIdx}"><input type="checkbox" ${isChecked ? "checked" : ""} class="task-check" data-task-index="${currentIdx}" /><span>${renderedLabel}</span></div>`;
  });

  const parsed = marked.parse(withTasks) as string;
  const enhanced = renderInlineExtensions(parsed);

  return enhanced.replace(/(?:<p>)?@@WINGNITE_CALLOUT_(\d+)@@(?:<\/p>)?/g, (_, idx: string) => {
    return callouts[Number(idx)] ?? "";
  });
}

export function toggleMarkdownTask(content: string, taskIndex: number): string {
  let counter = 0;
  return content.replace(/^-\s*\[([ xX])\]/gm, (match, check) => {
    if (counter++ === taskIndex) {
      return check.toLowerCase() === "x" ? "- [ ]" : "- [x]";
    }
    return match;
  });
}