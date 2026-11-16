// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::AppError;
use crate::models::TextMetricsResult;
use crate::services::text_analyzer::analyze_text;
use crate::services::twist_engine::generate_localized_twist;

#[tauri::command]
pub async fn analyze_text_metrics(
    text: String,
    locale: Option<String>,
) -> Result<TextMetricsResult, AppError> {
    let lang = locale.unwrap_or_else(|| "en".to_string());
    tauri::async_runtime::spawn_blocking(move || Ok(analyze_text(&text, &lang)))
        .await
        .map_err(|e| AppError::Concurrency(e.to_string()))?
}

#[tauri::command]
pub async fn generate_twist(locale: Option<String>) -> Result<String, AppError> {
    let lang = locale.unwrap_or_else(|| "en".to_string());
    Ok(generate_localized_twist(&lang))
}