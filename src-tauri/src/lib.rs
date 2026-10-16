use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Internal concurrency error: {0}")]
    Concurrency(String),
    #[error("File system error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Entity not found: {0}")]
    NotFound(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotItem {
    pub id: String,
    pub story_id: String,
    pub title: String,
    pub content: String,
    pub word_count: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtsSearchResult {
    pub id: String,
    pub title: String,
    pub category: String,
    pub snippet: String,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextMetricsResult {
    pub word_count: usize,
    pub char_count: usize,
    pub reading_time_minutes: f64,
    pub dialogue_ratio: f64,
    pub sentence_cadence: f64,
    pub vocabulary_richness: f64,
    pub repetition_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEventItem {
    pub id: String,
    pub story_id: Option<String>,
    pub title: String,
    pub description: String,
    pub act: String,
    pub character_name: String,
    pub temporal_order: i64,
    pub intensity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRelationItem {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub description: String,
}

pub struct DbState(pub Arc<Mutex<Connection>>);

fn get_db_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    fs::create_dir_all(&dir)?;
    Ok(dir.join("wingnite_story.db"))
}

#[tauri::command]
async fn get_stories(state: State<'_, DbState>) -> Result<Vec<StoryItem>, AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, title, content, category, updated_at FROM stories ORDER BY updated_at DESC",
        )?;
        let items = stmt
            .query_map([], |row| {
                Ok(StoryItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    category: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<StoryItem>, _>>()?;
        Ok(items)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn save_story(
    state: State<'_, DbState>,
    mut story: StoryItem,
) -> Result<StoryItem, AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        story.updated_at = chrono::Utc::now().timestamp();
        if story.id.is_empty() {
            story.id = uuid::Uuid::new_v4().to_string();
        }

        conn.execute(
            "INSERT INTO stories (id, title, content, category, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                category = excluded.category,
                updated_at = excluded.updated_at",
            params![
                story.id,
                story.title,
                story.content,
                story.category,
                story.updated_at
            ],
        )?;

        Ok(story)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn delete_story(state: State<'_, DbState>, id: String) -> Result<(), AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        conn.execute("DELETE FROM stories WHERE id = ?1", params![id])?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn create_snapshot(
    state: State<'_, DbState>,
    story_id: String,
    title: String,
    content: String,
) -> Result<SnapshotItem, AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        let id = uuid::Uuid::new_v4().to_string();
        let word_count = content.split_whitespace().count() as i64;
        let created_at = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO snapshots (id, story_id, title, content, word_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, story_id, title, content, word_count, created_at],
        )?;

        Ok(SnapshotItem {
            id,
            story_id,
            title,
            content,
            word_count,
            created_at,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn get_snapshots(
    state: State<'_, DbState>,
    story_id: String,
) -> Result<Vec<SnapshotItem>, AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, story_id, title, content, word_count, created_at 
             FROM snapshots 
             WHERE story_id = ?1 
             ORDER BY created_at DESC",
        )?;

        let items = stmt
            .query_map(params![story_id], |row| {
                Ok(SnapshotItem {
                    id: row.get(0)?,
                    story_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    word_count: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<SnapshotItem>, _>>()?;

        Ok(items)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn get_timeline_events(
    state: State<'_, DbState>,
) -> Result<Vec<TimelineEventItem>, AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, story_id, title, description, act, character_name, temporal_order, intensity 
             FROM timeline_events 
             ORDER BY temporal_order ASC",
        )?;

        let items = stmt
            .query_map([], |row| {
                Ok(TimelineEventItem {
                    id: row.get(0)?,
                    story_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    act: row.get(4)?,
                    character_name: row.get(5)?,
                    temporal_order: row.get(6)?,
                    intensity: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<TimelineEventItem>, _>>()?;

        Ok(items)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn save_timeline_event(
    state: State<'_, DbState>,
    mut event: TimelineEventItem,
) -> Result<TimelineEventItem, AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        if event.id.is_empty() {
            event.id = uuid::Uuid::new_v4().to_string();
        }

        conn.execute(
            "INSERT INTO timeline_events (id, story_id, title, description, act, character_name, temporal_order, intensity)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                story_id = excluded.story_id,
                title = excluded.title,
                description = excluded.description,
                act = excluded.act,
                character_name = excluded.character_name,
                temporal_order = excluded.temporal_order,
                intensity = excluded.intensity",
            params![
                event.id,
                event.story_id,
                event.title,
                event.description,
                event.act,
                event.character_name,
                event.temporal_order,
                event.intensity
            ],
        )?;

        Ok(event)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn delete_timeline_event(state: State<'_, DbState>, id: String) -> Result<(), AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        conn.execute("DELETE FROM timeline_events WHERE id = ?1", params![id])?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn get_entity_relations(
    state: State<'_, DbState>,
) -> Result<Vec<EntityRelationItem>, AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, source_id, target_id, relation_type, description 
             FROM entity_relations",
        )?;

        let items = stmt
            .query_map([], |row| {
                Ok(EntityRelationItem {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    target_id: row.get(2)?,
                    relation_type: row.get(3)?,
                    description: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<EntityRelationItem>, _>>()?;

        Ok(items)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn save_entity_relation(
    state: State<'_, DbState>,
    mut relation: EntityRelationItem,
) -> Result<EntityRelationItem, AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        if relation.id.is_empty() {
            relation.id = uuid::Uuid::new_v4().to_string();
        }

        conn.execute(
            "INSERT INTO entity_relations (id, source_id, target_id, relation_type, description)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                source_id = excluded.source_id,
                target_id = excluded.target_id,
                relation_type = excluded.relation_type,
                description = excluded.description",
            params![
                relation.id,
                relation.source_id,
                relation.target_id,
                relation.relation_type,
                relation.description
            ],
        )?;

        Ok(relation)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn delete_entity_relation(state: State<'_, DbState>, id: String) -> Result<(), AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        conn.execute("DELETE FROM entity_relations WHERE id = ?1", params![id])?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn search_stories_fts(
    state: State<'_, DbState>,
    query: String,
) -> Result<Vec<FtsSearchResult>, AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;

        let cleaned = query
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>();
        let terms: Vec<String> = cleaned
            .split_whitespace()
            .map(|w| format!("\"{}\"*", w))
            .collect();

        if terms.is_empty() {
            return Ok(Vec::new());
        }

        let fts_query = terms.join(" ");
        let mut stmt = conn.prepare_cached(
            "SELECT id, title, category, snippet(stories_fts, 2, '<b>', '</b>', '...', 14), bm25(stories_fts)
             FROM stories_fts
             WHERE stories_fts MATCH ?1
             ORDER BY bm25(stories_fts)
             LIMIT 40",
        )?;

        let items = stmt
            .query_map(params![fts_query], |row| {
                Ok(FtsSearchResult {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    category: row.get(2)?,
                    snippet: row.get(3)?,
                    rank: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<FtsSearchResult>, _>>()?;

        Ok(items)
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn analyze_text_metrics(text: String) -> Result<TextMetricsResult, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let char_count = text.chars().count();
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();
        let reading_time_minutes = (word_count as f64 / 200.0).max(0.1);

        let mut in_quote = false;
        let mut dialogue_chars = 0usize;
        for c in text.chars() {
            if c == '«' || c == '"' {
                in_quote = true;
            } else if c == '»' || (in_quote && c == '"') {
                in_quote = false;
            } else if in_quote {
                dialogue_chars += 1;
            }
        }
        let dialogue_ratio = if char_count > 0 {
            ((dialogue_chars as f64 / char_count as f64) * 100.0).min(100.0)
        } else {
            0.0
        };

        let sentences: Vec<&str> = text
            .split(|c| c == '.' || c == '!' || c == '?' || c == '…')
            .filter(|s| !s.trim().is_empty())
            .collect();

        let sentence_cadence = if !sentences.is_empty() {
            let lengths: Vec<f64> = sentences
                .iter()
                .map(|s| s.split_whitespace().count() as f64)
                .collect();
            let mean: f64 = lengths.iter().sum::<f64>() / lengths.len() as f64;
            let variance: f64 = lengths
                .iter()
                .map(|l| (l - mean).powi(2))
                .sum::<f64>()
                / lengths.len() as f64;
            variance.sqrt().min(40.0)
        } else {
            0.0
        };

        let mut word_freq: HashMap<String, usize> = HashMap::new();
        let mut unique_words = HashSet::new();
        for w in &words {
            let lower = w
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if lower.chars().count() > 3 {
                unique_words.insert(lower.clone());
                *word_freq.entry(lower).or_insert(0) += 1;
            }
        }

        let vocabulary_richness = if word_count > 0 {
            ((unique_words.len() as f64 / word_count as f64) * 100.0).min(100.0)
        } else {
            0.0
        };

        let mut freq_vec: Vec<(String, usize)> = word_freq.into_iter().collect();
        freq_vec.sort_by(|a, b| b.1.cmp(&a.1));
        let repetition_warnings = freq_vec
            .into_iter()
            .filter(|(_, count)| *count >= 4)
            .take(6)
            .map(|(word, count)| format!("{}: {} раз", word, count))
            .collect();

        Ok(TextMetricsResult {
            word_count,
            char_count,
            reading_time_minutes,
            dialogue_ratio,
            sentence_cadence,
            vocabulary_richness,
            repetition_warnings,
        })
    })
    .await
    .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
async fn compile_book_edition(state: State<'_, DbState>) -> Result<String, AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, title, content, category, updated_at FROM stories ORDER BY updated_at ASC",
        )?;
        let stories = stmt
            .query_map([], |row| {
                Ok(StoryItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    category: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<StoryItem>, _>>()?;

        let mut html = String::from("<!DOCTYPE html><html lang=\"ru\"><head><meta charset=\"UTF-8\"><title>Wingnite Story Edition</title><style>body{font-family:'Literata',Georgia,serif;max-width:760px;margin:40px auto;line-height:1.9;color:#1a1a1a;padding:20px;}h1,h2{font-family:'Playfair Display',serif;}h1{text-align:center;font-size:3em;margin-top:2em;}h2{page-break-before:always;font-size:2em;border-bottom:1px solid #ddd;padding-bottom:10px;margin-top:2em;}.toc{background:#f9f9f9;padding:25px;border-radius:10px;margin-bottom:50px;}p{text-indent:1.8em;margin-bottom:1em;text-align:justify;}</style></head><body><h1>Wingnite Story</h1><div class=\"toc\"><h2>Оглавление</h2><ul>");
        for s in &stories {
            html.push_str(&format!("<li>[{}] {}</li>", s.category, s.title));
        }
        html.push_str("</ul></div>");
        for s in &stories {
            html.push_str(&format!("<h2>{}</h2><p><em>Акт: {}</em></p>", s.title, s.category));
            for paragraph in s.content.lines() {
                if !paragraph.trim().is_empty() {
                    html.push_str(&format!("<p>{}</p>", paragraph));
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
async fn generate_twist() -> Result<String, AppError> {
    use rand::seq::IndexedRandom;
    let mut rng = rand::rng();

    let catalysts = [
        "Тайный советник ордена",
        "Преданный оруженосец",
        "Беглый картограф империи",
        "Торговец забытыми артефактами",
        "Странник из зеркального измерения",
        "Истинный наследник трона",
    ];

    let twists = [
        "невольно взламывает защитный барьер цитадели",
        "находит доказательства того, что война окончилась век назад",
        "теряет способность лгать перед лицом инквизиции",
        "обнаруживает в древнем фолианте портрет собственного предка",
        "похищает ключ от крипты забвения ради спасения врага",
        "снимает маску, обнажая метку павшего демиурга",
    ];

    let climaxes = [
        "и теперь обратный отсчет необратим.",
        "что ставит под удар всю экспедицию.",
        "пока на границе собираются тени прошлого.",
        "хотя цена этой правды уже оплачена чужой кровью.",
        "вынуждая стороны заключить шаткое перемирие.",
    ];

    let c = catalysts.choose(&mut rng).unwrap_or(&"Герой");
    let t = twists.choose(&mut rng).unwrap_or(&"совершает ошибку");
    let cl = climaxes.choose(&mut rng).unwrap_or(&"навсегда.");

    Ok(format!("💡 ПОВОРОТ: {} {}, {}", c, t, cl))
}

#[tauri::command]
async fn export_manuscript(state: State<'_, DbState>) -> Result<String, AppError> {
    let db = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = db
            .lock()
            .map_err(|e| AppError::Concurrency(e.to_string()))?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, title, content, category, updated_at FROM stories ORDER BY updated_at ASC",
        )?;
        let stories = stmt
            .query_map([], |row| {
                Ok(StoryItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    category: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<StoryItem>, _>>()?;

        let mut doc =
            String::from("# Wingnite Story\n*by K. I. P. Studio*\n\n---\n\n## Оглавление\n");
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = get_db_path(app.handle())?;
            let conn = Connection::open(db_path)?;

            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA foreign_keys = ON;

                 CREATE TABLE IF NOT EXISTS stories (
                     id TEXT PRIMARY KEY,
                     title TEXT NOT NULL,
                     content TEXT NOT NULL,
                     category TEXT NOT NULL,
                     updated_at INTEGER NOT NULL
                 );

                 CREATE TABLE IF NOT EXISTS snapshots (
                     id TEXT PRIMARY KEY,
                     story_id TEXT NOT NULL,
                     title TEXT NOT NULL,
                     content TEXT NOT NULL,
                     word_count INTEGER NOT NULL,
                     created_at INTEGER NOT NULL,
                     FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
                 );

                 CREATE TABLE IF NOT EXISTS timeline_events (
                     id TEXT PRIMARY KEY,
                     story_id TEXT,
                     title TEXT NOT NULL,
                     description TEXT NOT NULL,
                     act TEXT NOT NULL,
                     character_name TEXT NOT NULL,
                     temporal_order INTEGER NOT NULL,
                     intensity INTEGER NOT NULL,
                     FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE SET NULL
                 );

                 CREATE TABLE IF NOT EXISTS entity_relations (
                     id TEXT PRIMARY KEY,
                     source_id TEXT NOT NULL,
                     target_id TEXT NOT NULL,
                     relation_type TEXT NOT NULL,
                     description TEXT NOT NULL,
                     FOREIGN KEY (source_id) REFERENCES stories(id) ON DELETE CASCADE,
                     FOREIGN KEY (target_id) REFERENCES stories(id) ON DELETE CASCADE
                 );

                 CREATE INDEX IF NOT EXISTS idx_snapshots_story_id ON snapshots(story_id);
                 CREATE INDEX IF NOT EXISTS idx_stories_updated_at ON stories(updated_at DESC);
                 CREATE INDEX IF NOT EXISTS idx_timeline_temporal ON timeline_events(temporal_order ASC);
                 CREATE INDEX IF NOT EXISTS idx_relations_source ON entity_relations(source_id);
                 CREATE INDEX IF NOT EXISTS idx_relations_target ON entity_relations(target_id);

                 CREATE VIRTUAL TABLE IF NOT EXISTS stories_fts USING fts5(
                     id UNINDEXED,
                     title,
                     content,
                     category,
                     tokenize='unicode61'
                 );

                 CREATE TRIGGER IF NOT EXISTS stories_ai AFTER INSERT ON stories BEGIN
                     INSERT INTO stories_fts(id, title, content, category)
                     VALUES (new.id, new.title, new.content, new.category);
                 END;

                 CREATE TRIGGER IF NOT EXISTS stories_ad AFTER DELETE ON stories BEGIN
                     DELETE FROM stories_fts WHERE id = old.id;
                 END;

                 CREATE TRIGGER IF NOT EXISTS stories_au AFTER UPDATE ON stories BEGIN
                     DELETE FROM stories_fts WHERE id = old.id;
                     INSERT INTO stories_fts(id, title, content, category)
                     VALUES (new.id, new.title, new.content, new.category);
                 END;

                 INSERT INTO stories_fts(id, title, content, category)
                 SELECT id, title, content, category FROM stories
                 WHERE id NOT IN (SELECT id FROM stories_fts);",
            )?;

            app.manage(DbState(Arc::new(Mutex::new(conn))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_stories,
            save_story,
            delete_story,
            create_snapshot,
            get_snapshots,
            get_timeline_events,
            save_timeline_event,
            delete_timeline_event,
            get_entity_relations,
            save_entity_relation,
            delete_entity_relation,
            search_stories_fts,
            analyze_text_metrics,
            compile_book_edition,
            generate_twist,
            export_manuscript
        ])
        .run(tauri::generate_context!())
        .expect("error while running Wingnite Story application");
}