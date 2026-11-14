#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Instance {
    id: String,
    name: String,
    version: String,
    loader: String,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Settings {
    ram_gb: u8,
    game_directory: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateInstanceInput {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsPatch {
    ram_gb: u8,
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| format!("Could not locate Crescent data directory: {error}"))
}

fn instances_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("instances"))
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("settings.json"))
}

fn read_settings(app: &AppHandle) -> Result<Settings, String> {
    let path = settings_path(app)?;
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map_err(|error| format!("Could not parse settings: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let game_dir = app_data_dir(app)?.join("game");
            Ok(Settings {
                ram_gb: 4,
                game_directory: game_dir.to_string_lossy().into_owned(),
            })
        }
        Err(error) => Err(format!("Could not read settings: {error}")),
    }
}

#[tauri::command]
async fn get_versions() -> Result<serde_json::Value, String> {
    let response = reqwest::get(
        "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
    )
    .await
    .map_err(|error| format!("Could not reach Minecraft version service: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Minecraft version service returned HTTP {}",
            response.status()
        ));
    }

    let manifest: serde_json::Value = response
        .json()
        .await
        .map_err(|error| format!("Could not parse Minecraft version manifest: {error}"))?;

    let releases: Vec<String> = manifest["versions"]
        .as_array()
        .ok_or("Minecraft version manifest has no versions list")?
        .iter()
        .filter(|item| item["type"].as_str() == Some("release"))
        .take(40)
        .filter_map(|item| item["id"].as_str().map(str::to_owned))
        .collect();

    Ok(serde_json::json!({
        "latest": manifest["latest"],
        "releases": releases
    }))
}

#[tauri::command]
fn list_instances(app: AppHandle) -> Result<Vec<Instance>, String> {
    let root = instances_dir(&app)?;
    fs::create_dir_all(&root)
        .map_err(|error| format!("Could not create instances directory: {error}"))?;

    let mut instances = Vec::new();
    let entries = fs::read_dir(root)
        .map_err(|error| format!("Could not list instances: {error}"))?;

    for entry in entries.flatten() {
        if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            continue;
        }
        let file = entry.path().join("instance.json");
        if let Ok(contents) = fs::read_to_string(file) {
            if let Ok(instance) = serde_json::from_str::<Instance>(&contents) {
                instances.push(instance);
            }
        }
    }

    instances.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(instances)
}

#[tauri::command]
fn create_instance(app: AppHandle, input: CreateInstanceInput) -> Result<Instance, String> {
    let name = input.name.trim();
    let version = input.version.trim();

    if name.is_empty()
        || name.chars().count() > 32
        || !name.chars().all(|ch| ch.is_alphanumeric() || " _.-".contains(ch))
    {
        return Err("Choose an instance name using letters, numbers, spaces, dots, dashes, or underscores (up to 32 characters).".into());
    }

    let version_parts: Vec<&str> = version.split('.').collect();
    if !(2..=3).contains(&version_parts.len())
        || version_parts.iter().any(|part| part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit()))
    {
        return Err("Choose a valid Minecraft release version.".into());
    }

    let id = Uuid::new_v4().to_string();
    let folder = instances_dir(&app)?.join(&id);
    fs::create_dir_all(folder.join("mods"))
        .map_err(|error| format!("Could not create mods directory: {error}"))?;
    fs::create_dir_all(folder.join("config"))
        .map_err(|error| format!("Could not create config directory: {error}"))?;

    let instance = Instance {
        id,
        name: name.to_owned(),
        version: version.to_owned(),
        loader: "vanilla".into(),
        created_at: chrono_free_timestamp(),
    };

    let metadata = serde_json::to_string_pretty(&instance)
        .map_err(|error| format!("Could not serialize instance: {error}"))?;
    write_file(&folder.join("instance.json"), &metadata)?;
    Ok(instance)
}

fn write_file(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create data directory: {error}"))?;
    }
    fs::write(path, contents).map_err(|error| format!("Could not save file: {error}"))
}

fn chrono_free_timestamp() -> String {
    // A sortable timestamp without an additional time dependency.
    // UUIDs still guarantee unique instance folder names.
    format!("{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Result<Settings, String> {
    read_settings(&app)
}

#[tauri::command]
fn save_settings(app: AppHandle, patch: SettingsPatch) -> Result<Settings, String> {
    if !(2..=8).contains(&patch.ram_gb) {
        return Err("RAM allocation must be between 2 and 8 GB.".into());
    }

    let mut settings = read_settings(&app)?;
    settings.ram_gb = patch.ram_gb;
    let serialized = serde_json::to_string_pretty(&settings)
        .map_err(|error| format!("Could not serialize settings: {error}"))?;
    write_file(&settings_path(&app)?, &serialized)?;
    Ok(settings)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_versions,
            list_instances,
            create_instance,
            get_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running Crescent Client");
}
