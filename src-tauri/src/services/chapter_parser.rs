// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

pub fn escape_html(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn replace_delimited(input: &str, open: &str, close: &str, tag_open: &str, tag_close: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut cursor = 0usize;

    while let Some(rel_start) = input[cursor..].find(open) {
        let start = cursor + rel_start;
        let content_start = start + open.len();
        if let Some(rel_end) = input[content_start..].find(close) {
            let end = content_start + rel_end;
            result.push_str(&input[cursor..start]);
            result.push_str(tag_open);
            result.push_str(&input[content_start..end]);
            result.push_str(tag_close);
            cursor = end + close.len();
        } else {
            break;
        }
    }
    result.push_str(&input[cursor..]);
    result
}

fn render_inline_escaped(raw: &str) -> String {
    let escaped = escape_html(raw);
    let step1 = replace_delimited(&escaped, "`", "`", "<code>", "</code>");
    let step2 = replace_delimited(&step1, "**", "**", "<strong>", "</strong>");
    let step3 = replace_delimited(&step2, "~~", "~~", "<del>", "</del>");
    let step4 = replace_delimited(&step3, "*", "*", "<em>", "</em>");
    let step5 = replace_delimited(&step4, "==", "==", "<mark>", "</mark>");
    replace_delimited(&step5, "[[", "]]", "<span class=\"wiki-ref\">", "</span>")
}

pub fn render_markdown_to_html(raw: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut in_list = false;

    for line in raw.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            if in_code_block {
                html.push_str("</code></pre>");
                in_code_block = false;
            } else {
                html.push_str("<pre><code>");
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&escape_html(line));
            html.push('\n');
            continue;
        }

        if trimmed.is_empty() {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            continue;
        }

        if trimmed == "---" || trimmed == "***" {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str("<hr/>");
            continue;
        }

        if let Some(item) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            if !in_list {
                html.push_str("<ul>");
                in_list = true;
            }
            html.push_str(&format!("<li>{}</li>", render_inline_escaped(item)));
            continue;
        }

        if in_list {
            html.push_str("</ul>");
            in_list = false;
        }

        if let Some(h3) = trimmed.strip_prefix("### ") {
            html.push_str(&format!("<h4>{}</h4>", render_inline_escaped(h3)));
        } else if let Some(h2) = trimmed.strip_prefix("## ") {
            html.push_str(&format!("<h3>{}</h3>", render_inline_escaped(h2)));
        } else if let Some(h1) = trimmed.strip_prefix("# ") {
            html.push_str(&format!("<h3>{}</h3>", render_inline_escaped(h1)));
        } else if let Some(quote) = trimmed.strip_prefix("> ") {
            html.push_str(&format!("<blockquote>{}</blockquote>", render_inline_escaped(quote)));
        } else {
            html.push_str(&format!("<p>{}</p>", render_inline_escaped(trimmed)));
        }
    }

    if in_list {
        html.push_str("</ul>");
    }
    if in_code_block {
        html.push_str("</code></pre>");
    }

    html
}

pub fn localized_export_labels(locale: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    match locale {
        "ru" => ("Оглавление", "Акт", "Категория", "Экспорт рукописи K. I. P. Studio"),
        "de" => ("Inhaltsverzeichnis", "Akt", "Kategorie", "Manuskript-Export von K. I. P. Studio"),
        "es" => ("Tabla de contenidos", "Acto", "Categoría", "Exportación de manuscrito de K. I. P. Studio"),
        _ => ("Table of Contents", "Act", "Category", "Manuscript Export by K. I. P. Studio"),
    }
}

pub fn split_plain_text_chapters(
    raw_text: &str,
    default_title: &str,
    locale: &str,
) -> Vec<(String, String, String)> {
    let lines: Vec<&str> = raw_text.lines().collect();
    let mut chapters: Vec<(String, String)> = Vec::new();
    let mut current_title = default_title.to_string();
    let mut current_body = String::new();

    let prefixes = [
        "# ",
        "chapter ",
        "act ",
        "part ",
        "глава ",
        "акт ",
        "часть ",
        "kapitel ",
        "akt ",
        "teil ",
        "capítulo ",
        "acto ",
        "parte ",
    ];

    for line in lines {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        let is_header = prefixes.iter().any(|prefix| lower.starts_with(prefix));

        if is_header && !current_body.trim().is_empty() {
            chapters.push((current_title.clone(), current_body.trim().to_string()));
            current_title = trimmed.replace('#', "").trim().to_string();
            current_body.clear();
        } else if is_header && current_body.trim().is_empty() {
            current_title = trimmed.replace('#', "").trim().to_string();
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }

    if !current_body.trim().is_empty() {
        chapters.push((current_title, current_body.trim().to_string()));
    }

    if chapters.is_empty() {
        chapters.push((default_title.to_string(), raw_text.to_string()));
    }

    let (_, act_word, _, _) = localized_export_labels(locale);

    chapters
        .into_iter()
        .enumerate()
        .map(|(idx, (chap_title, chap_content))| {
            let lower = chap_title.to_lowercase();
            let category = if lower.contains("act")
                || lower.contains("акт")
                || lower.contains("akt")
                || lower.contains("acto")
            {
                chap_title.clone()
            } else {
                format!("{} {}", act_word, (idx / 4) + 1)
            };
            (chap_title, chap_content, category)
        })
        .collect()
}