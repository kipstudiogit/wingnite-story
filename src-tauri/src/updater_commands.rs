use crate::error::AppError;
use crate::models::UpdateStatusPayload;

#[tauri::command]
pub async fn check_application_update(
    app: tauri::AppHandle,
) -> Result<UpdateStatusPayload, AppError> {
    #[cfg(desktop)]
    {
        use tauri_plugin_updater::UpdaterExt;
        if let Ok(updater) = app.updater() {
            match updater.check().await {
                Ok(Some(update)) => {
                    return Ok(UpdateStatusPayload {
                        available: true,
                        current_version: update.current_version.clone(),
                        version: Some(update.version.clone()),
                        date: update.date.map(|d| d.to_string()),
                        body: update.body.clone(),
                    });
                }
                Ok(None) => {
                    return Ok(UpdateStatusPayload {
                        available: false,
                        current_version: app.package_info().version.to_string(),
                        version: None,
                        date: None,
                        body: None,
                    });
                }
                Err(e) => {
                    return Err(AppError::Streaming(format!("Update check failed: {}", e)));
                }
            }
        }
    }

    Ok(UpdateStatusPayload {
        available: false,
        current_version: app.package_info().version.to_string(),
        version: None,
        date: None,
        body: None,
    })
}

#[tauri::command]
pub async fn install_application_update(
    app: tauri::AppHandle,
) -> Result<bool, AppError> {
    #[cfg(desktop)]
    {
        use tauri_plugin_updater::UpdaterExt;
        if let Ok(updater) = app.updater() {
            if let Ok(Some(update)) = updater.check().await {
                update
                    .download_and_install(|_chunk_len, _content_len| {}, || {})
                    .await
                    .map_err(|e| AppError::Streaming(format!("Install error: {}", e)))?;

                app.restart();
            }
        }
    }
    Ok(true)
}