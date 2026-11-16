// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::commands::vault::ensure_project_unlocked;
use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::models::EnterpriseStoryItem;
use crate::services::chapter_parser::{
    escape_html, localized_export_labels, render_markdown_to_html, split_plain_text_chapters,
};
use crate::services::crypto_vault::VaultService;
use crate::services::lore_engine::sync_story_embedding;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn compile_book_edition(
    state: State<'_, EnterpriseDbPool>,
    project_id: Option<String>,
    locale: Option<String>,
) -> Result<String, AppError> {
    let db = state.reader.clone();
    let lang = locale.unwrap_or_else(|| "en".to_string());

    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| "default_project".to_string());

        let mut proj_stmt = conn.prepare_cached("SELECT title FROM projects WHERE id = ?1")?;
        let project_title: String = proj_stmt
            .query_row(params![target_pid], |r| r.get(0))
            .unwrap_or_else(|_| "Wingnite Story Edition".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT id, project_id, title, content, category, revision, sort_order, content_hash, updated_at
             FROM stories
             WHERE project_id = ?1
             ORDER BY sort_order ASC, updated_at ASC",
        )?;
        let stories = stmt
            .query_map(params![target_pid], |row| {
                Ok(EnterpriseStoryItem {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    category: row.get(4)?,
                    revision: row.get(5)?,
                    sort_order: row.get(6)?,
                    content_hash: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<EnterpriseStoryItem>, _>>()?;

        let (toc_heading, act_label, _, _) = localized_export_labels(&lang);
        let safe_project_title = escape_html(&project_title);
        let safe_lang = escape_html(&lang);

        let mut html = format!(
            "<!DOCTYPE html><html lang=\"{}\"><head><meta charset=\"UTF-8\"><title>{}</title><style>body{{font-family:'Literata',Georgia,serif;max-width:760px;margin:40px auto;line-height:1.9;color:#1a1a1a;padding:20px;}}h1,h2,h3,h4{{font-family:'Playfair Display',serif;}}h1{{text-align:center;font-size:3em;margin-top:2em;}}h2{{page-break-before:always;font-size:2em;border-bottom:1px solid #ddd;padding-bottom:10px;margin-top:2em;}}.toc{{background:#f9f9f9;padding:25px;border-radius:10px;margin-bottom:50px;}}p{{text-indent:1.8em;margin-bottom:1em;text-align:justify;}}blockquote{{border-left:3px solid #6366f1;padding:10px 18px;margin:1.5em 0;font-style:italic;background:#f5f7ff;}}pre{{background:#f3f4f6;padding:14px;border-radius:8px;overflow-x:auto;}}code{{font-family:monospace;background:#eef2ff;padding:2px 5px;border-radius:4px;}}.wiki-ref{{color:#4338ca;font-weight:700;}}hr{{border:none;border-top:1px solid #ddd;margin:2.5em 0;}}</style></head><body><h1>{}</h1><div class=\"toc\"><h2>{}</h2><ul>",
            safe_lang, safe_project_title, safe_project_title, escape_html(toc_heading)
        );

        for s in &stories {
            html.push_str(&format!(
                "<li>[{}] {}</li>",
                escape_html(&s.category),
                escape_html(&s.title)
            ));
        }
        html.push_str("</ul></div>");
        for s in &stories {
            html.push_str(&format!(
                "<h2>{}</h2><p><em>{}: {}</em></p>",
                escape_html(&s.title),
                escape_html(act_label),
                escape_html(&s.category)
            ));
            html.push_str(&render_markdown_to_html(&s.content));
        }
        html.push_str("</body></html>");
        Ok(html)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn export_manuscript(
    state: State<'_, EnterpriseDbPool>,
    project_id: Option<String>,
    locale: Option<String>,
) -> Result<String, AppError> {
    let db = state.reader.clone();
    let lang = locale.unwrap_or_else(|| "en".to_string());

    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| "default_project".to_string());

        let mut proj_stmt = conn.prepare_cached("SELECT title FROM projects WHERE id = ?1")?;
        let project_title: String = proj_stmt
            .query_row(params![target_pid], |r| r.get(0))
            .unwrap_or_else(|_| "Wingnite Story".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT id, project_id, title, content, category, revision, sort_order, content_hash, updated_at
             FROM stories
             WHERE project_id = ?1
             ORDER BY sort_order ASC, updated_at ASC",
        )?;
        let stories = stmt
            .query_map(params![target_pid], |row| {
                Ok(EnterpriseStoryItem {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    category: row.get(4)?,
                    revision: row.get(5)?,
                    sort_order: row.get(6)?,
                    content_hash: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<EnterpriseStoryItem>, _>>()?;

        let (toc_heading, _, cat_label, export_sub) = localized_export_labels(&lang);
        let mut doc = format!(
            "# {}\n*{}*\n\n---\n\n## {}\n",
            project_title, export_sub, toc_heading
        );
        for s in &stories {
            doc.push_str(&format!("* [{}] {}\n", s.category, s.title));
        }
        doc.push_str("\n---\n\n");
        for s in &stories {
            doc.push_str(&format!("## {}\n\n*{}: {}*\n\n", s.title, cat_label, s.category));
            doc.push_str(&s.content);
            doc.push_str("\n\n---\n\n");
        }
        Ok(doc)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn export_manuscript_txt(
    state: State<'_, EnterpriseDbPool>,
    project_id: Option<String>,
) -> Result<String, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| "default_project".to_string());

        let mut proj_stmt = conn.prepare_cached("SELECT title FROM projects WHERE id = ?1")?;
        let project_title: String = proj_stmt
            .query_row(params![target_pid], |r| r.get(0))
            .unwrap_or_else(|_| "Wingnite Story".to_string());

        let mut stmt = conn.prepare_cached(
            "SELECT id, project_id, title, content, category, revision, sort_order, content_hash, updated_at
             FROM stories
             WHERE project_id = ?1
             ORDER BY sort_order ASC, updated_at ASC",
        )?;
        let stories = stmt
            .query_map(params![target_pid], |row| {
                Ok(EnterpriseStoryItem {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    category: row.get(4)?,
                    revision: row.get(5)?,
                    sort_order: row.get(6)?,
                    content_hash: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<EnterpriseStoryItem>, _>>()?;

        let mut doc = format!("{}\n====================\n\n", project_title);
        for s in &stories {
            doc.push_str(&format!("\n\n--- {} [{}] ---\n\n", s.title, s.category));
            doc.push_str(&s.content);
            doc.push('\n');
        }
        Ok(doc)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn import_plain_text_chapters(
    state: State<'_, EnterpriseDbPool>,
    project_id: String,
    raw_text: String,
    default_title: String,
    locale: Option<String>,
) -> Result<Vec<EnterpriseStoryItem>, AppError> {
    let db = state.writer.clone();
    let lang = locale.unwrap_or_else(|| "en".to_string());

    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        let target_pid = if project_id.trim().is_empty() {
            "default_project".to_string()
        } else {
            project_id
        };

        ensure_project_unlocked(&tx, &target_pid)?;

        let base_order: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM stories WHERE project_id = ?1",
                params![target_pid],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let chapters = split_plain_text_chapters(&raw_text, &default_title, &lang);
        let mut imported_stories = Vec::with_capacity(chapters.len());
        let now = chrono::Utc::now().timestamp();

        for (idx, (chap_title, chap_content, category)) in chapters.into_iter().enumerate() {
            let story_id = uuid::Uuid::new_v4().to_string();
            let hash = VaultService::calculate_hash(&chap_content);
            let order = base_order + idx as i64;

            tx.execute(
                "INSERT INTO stories (id, project_id, title, content, category, revision, sort_order, content_hash, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8)",
                params![
                    story_id,
                    target_pid,
                    chap_title,
                    chap_content,
                    category,
                    order,
                    hash,
                    now + idx as i64
                ],
            )?;

            sync_story_embedding(&tx, &story_id, &chap_content)?;

            imported_stories.push(EnterpriseStoryItem {
                id: story_id,
                project_id: target_pid.clone(),
                title: chap_title,
                content: chap_content,
                category,
                revision: 1,
                sort_order: order,
                content_hash: hash,
                updated_at: now + idx as i64,
            });
        }

        tx.commit()?;
        Ok(imported_stories)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}