use tauri_plugin_updater::UpdaterExt;

#[derive(serde::Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub body: Option<String>,
}

#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    
    match updater.check().await {
        Ok(Some(update)) => {
            Ok(Some(UpdateInfo {
                version: update.version.clone(),
                body: update.body.clone(),
            }))
        }
        Ok(None) => {
            Ok(None) // 已是最新版本
        }
        Err(e) => {
            Err(format!("检查更新失败: {}", e))
        }
    }
}

#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<String, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    
    match updater.check().await {
        Ok(Some(update)) => {
            let mut downloaded = 0;
            
            update
                .download_and_install(
                    |chunk_length, content_length| {
                        downloaded += chunk_length;
                        if let Some(total) = content_length {
                            println!("下载进度: {}/{}", downloaded, total);
                        }
                    },
                    || {
                        println!("下载完成");
                    },
                )
                .await
                .map_err(|e| e.to_string())?;
            
            Ok("更新已下载，重启应用以完成安装".to_string())
        }
        Ok(None) => {
            Ok("当前已是最新版本".to_string())
        }
        Err(e) => {
            Err(format!("安装更新失败: {}", e))
        }
    }
}
