use crate::db::EnterpriseDbPool;
use crate::error::AppError;
use crate::lore_guardian::sync_story_embedding;
use crate::models::EnterpriseStoryItem;
use crate::vault::VaultService;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn compile_book_edition(
    state: State<'_, EnterpriseDbPool>,
    project_id: Option<String>,
) -> Result<String, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id.unwrap_or_else(|| "default_project".to_string());

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

        let mut html = format!(
            "<!DOCTYPE html><html lang=\"ru\"><head><meta charset=\"UTF-8\"><title>{}</title><style>body{{font-family:'Literata',Georgia,serif;max-width:760px;margin:40px auto;line-height:1.9;color:#1a1a1a;padding:20px;}}h1,h2{{font-family:'Playfair Display',serif;}}h1{{text-align:center;font-size:3em;margin-top:2em;}}h2{{page-break-before:always;font-size:2em;border-bottom:1px solid #ddd;padding-bottom:10px;margin-top:2em;}}.toc{{background:#f9f9f9;padding:25px;border-radius:10px;margin-bottom:50px;}}p{{text-indent:1.8em;margin-bottom:1em;text-align:justify;}}</style></head><body><h1>{}</h1><div class=\"toc\"><h2>Оглавление</h2><ul>",
            project_title, project_title
        );

        for s in &stories {
            html.push_str(&format!("<li>[{}] {}</li>", s.category, s.title));
        }
        html.push_str("</ul></div>");
        for s in &stories {
            html.push_str(&format!("<h2>{}</h2><p><em>Акт: {}</em></p>", s.title, s.category));
            for paragraph in s.content.lines() {
                let trimmed = paragraph.trim();
                if !trimmed.is_empty() {
                    html.push_str(&format!("<p>{}</p>", trimmed));
                }
            }
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
) -> Result<String, AppError> {
    let db = state.reader.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db.lock();
        let target_pid = project_id.unwrap_or_else(|| "default_project".to_string());

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

        let mut doc = format!("# {}\n*Экспорт произведения K. I. P. Studio*\n\n---\n\n## Оглавление\n", project_title);
        for s in &stories {
            doc.push_str(&format!("* [{}] {}\n", s.category, s.title));
        }
        doc.push_str("\n---\n\n");
        for s in &stories {
            doc.push_str(&format!("## {}\n\n*Категория: {}*\n\n", s.title, s.category));
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
        let target_pid = project_id.unwrap_or_else(|| "default_project".to_string());

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
            doc.push_str("\n");
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
) -> Result<Vec<EnterpriseStoryItem>, AppError> {
    let db = state.writer.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = db.lock();
        let tx = conn.transaction()?;

        let mut imported_stories = Vec::new();
        let lines: Vec<&str> = raw_text.lines().collect();

        let mut chapters: Vec<(String, String)> = Vec::new();
        let mut current_title = default_title.clone();
        let mut current_body = String::new();

        let chapter_regex_prefixes = ["# ", "глава ", "акт ", "часть ", "chapter "];

        for line in lines {
            let trimmed = line.trim();
            let is_chapter_header = chapter_regex_prefixes.iter().any(|prefix| {
                trimmed.to_lowercase().starts_with(prefix)
            });

            if is_chapter_header && !current_body.trim().is_empty() {
                chapters.push((current_title.clone(), current_body.trim().to_string()));
                current_title = trimmed.replace('#', "").trim().to_string();
                current_body.clear();
            } else if is_chapter_header && current_body.trim().is_empty() {
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
            chapters.push((default_title, raw_text.clone()));
        }

        let now = chrono::Utc::now().timestamp();

        for (idx, (chap_title, chap_content)) in chapters.into_iter().enumerate() {
            let story_id = uuid::Uuid::new_v4().to_string();
            let hash = VaultService::calculate_hash(&chap_content);
            let category = if chap_title.to_lowercase().contains("акт") {
                chap_title.clone()
            } else {
                format!("Акт {}", (idx / 4) + 1)
            };

            tx.execute(
                "INSERT INTO stories (id, project_id, title, content, category, revision, sort_order, content_hash, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8)",
                params![story_id, project_id, chap_title, chap_content, category, idx as i64, hash, now + idx as i64],
            )?;

            sync_story_embedding(&tx, &story_id, &chap_content)?;

            imported_stories.push(EnterpriseStoryItem {
                id: story_id,
                project_id: project_id.clone(),
                title: chap_title,
                content: chap_content,
                category,
                revision: 1,
                sort_order: idx as i64,
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