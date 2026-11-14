#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures_util::{stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallResult {
    version: String,
    libraries_downloaded: usize,
    assets_downloaded: usize,
    game_directory: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JavaInfo {
    available: bool,
    version: Option<String>,
    executable: Option<String>,
    message: String,
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

fn game_root(app: &AppHandle) -> Result<PathBuf, String> {
    let settings = read_settings(app)?;
    Ok(PathBuf::from(settings.game_directory))
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

async fn download_file(
    client: &reqwest::Client,
    url: &str,
    destination: &Path,
    expected_sha1: Option<&str>,
) -> Result<(), String> {
    if destination.exists() {
        if let Some(expected) = expected_sha1 {
            let existing = fs::read(destination)
                .map_err(|error| format!("Could not verify cached file: {error}"))?;
            let mut hasher = Sha1::new();
            hasher.update(&existing);
            if hex::encode(hasher.finalize()).eq_ignore_ascii_case(expected) {
                return Ok(());
            }
        } else {
            return Ok(());
        }
    }

    let response = client.get(url).send().await
        .map_err(|error| format!("Download request failed: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("Download failed with HTTP {}: {}", response.status(), url));
    }
    let bytes = response.bytes().await
        .map_err(|error| format!("Could not read download: {error}"))?;

    if let Some(expected) = expected_sha1 {
        let mut hasher = Sha1::new();
        hasher.update(&bytes);
        let actual = hex::encode(hasher.finalize());
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(format!("SHA-1 verification failed for {}", destination.display()));
        }
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create download directory: {error}"))?;
    }
    let temporary = destination.with_extension("crescent-download");
    fs::write(&temporary, &bytes)
        .map_err(|error| format!("Could not write download: {error}"))?;
    fs::rename(&temporary, destination)
        .map_err(|error| format!("Could not finalize download: {error}"))?;
    Ok(())
}

#[tauri::command]
async fn get_versions() -> Result<serde_json::Value, String> {
    let response = reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .await
        .map_err(|error| format!("Could not reach Minecraft version service: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("Minecraft version service returned HTTP {}", response.status()));
    }
    let manifest: serde_json::Value = response.json().await
        .map_err(|error| format!("Could not parse Minecraft version manifest: {error}"))?;
    let releases: Vec<String> = manifest["versions"].as_array()
        .ok_or("Minecraft version manifest has no versions list")?
        .iter()
        .filter(|item| item["type"].as_str() == Some("release"))
        .take(80)
        .filter_map(|item| item["id"].as_str().map(str::to_owned))
        .collect();
    Ok(serde_json::json!({ "latest": manifest["latest"], "releases": releases }))
}

#[tauri::command]
fn list_instances(app: AppHandle) -> Result<Vec<Instance>, String> {
    let root = instances_dir(&app)?;
    fs::create_dir_all(&root).map_err(|error| format!("Could not create instances directory: {error}"))?;
    let mut instances = Vec::new();
    let entries = fs::read_dir(root).map_err(|error| format!("Could not list instances: {error}"))?;
    for entry in entries.flatten() {
        if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) { continue; }
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
    if name.is_empty() || name.chars().count() > 32
        || !name.chars().all(|ch| ch.is_alphanumeric() || " _.-".contains(ch)) {
        return Err("Choose an instance name using letters, numbers, spaces, dots, dashes, or underscores (up to 32 characters).".into());
    }
    let parts: Vec<&str> = version.split('.').collect();
    if !(2..=3).contains(&parts.len())
        || parts.iter().any(|part| part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit())) {
        return Err("Choose a valid Minecraft release version.".into());
    }

    let id = Uuid::new_v4().to_string();
    let folder = instances_dir(&app)?.join(&id);
    fs::create_dir_all(folder.join("mods")).map_err(|error| format!("Could not create mods directory: {error}"))?;
    fs::create_dir_all(folder.join("config")).map_err(|error| format!("Could not create config directory: {error}"))?;
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
        fs::create_dir_all(parent).map_err(|error| format!("Could not create data directory: {error}"))?;
    }
    fs::write(path, contents).map_err(|error| format!("Could not save file: {error}"))
}

fn chrono_free_timestamp() -> String {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_secs().to_string()
}

#[tauri::command]
async fn install_version(app: AppHandle, instance_id: String) -> Result<InstallResult, String> {
    let instance = list_instances(app.clone())?.into_iter()
        .find(|item| item.id == instance_id)
        .ok_or("Instance was not found")?;
    let client = reqwest::Client::builder().user_agent("CrescentClient/0.2.0").build()
        .map_err(|error| format!("Could not initialize downloader: {error}"))?;

    let manifest_response = client.get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .send().await.map_err(|error| format!("Could not fetch version manifest: {error}"))?;
    if !manifest_response.status().is_success() {
        return Err(format!("Version manifest returned HTTP {}", manifest_response.status()));
    }
    let manifest: serde_json::Value = manifest_response.json().await
        .map_err(|error| format!("Invalid version manifest: {error}"))?;
    let version_entry = manifest["versions"].as_array()
        .and_then(|versions| versions.iter().find(|v| v["id"].as_str() == Some(instance.version.as_str())))
        .ok_or("That Minecraft version was not found in Mojang's release manifest")?;
    let version_url = version_entry["url"].as_str().ok_or("Version metadata URL is missing")?;

    let metadata_response = client.get(version_url).send().await
        .map_err(|error| format!("Could not fetch version metadata: {error}"))?;
    if !metadata_response.status().is_success() {
        return Err(format!("Version metadata returned HTTP {}", metadata_response.status()));
    }
    let metadata_bytes = metadata_response.bytes().await
        .map_err(|error| format!("Could not read version metadata: {error}"))?;
    let metadata: serde_json::Value = serde_json::from_slice(&metadata_bytes)
        .map_err(|error| format!("Invalid version metadata: {error}"))?;

    let root = game_root(&app)?;
    let version_dir = root.join("versions").join(&instance.version);
    fs::create_dir_all(&version_dir).map_err(|error| format!("Could not create version folder: {error}"))?;
    fs::write(version_dir.join(format!("{}.json", instance.version)), &metadata_bytes)
        .map_err(|error| format!("Could not save version metadata: {error}"))?;

    let client_download = &metadata["downloads"]["client"];
    let client_url = client_download["url"].as_str().ok_or("Minecraft client download URL is missing")?;
    let client_hash = client_download["sha1"].as_str();
    let client_path = version_dir.join(format!("{}.jar", instance.version));
    download_file(&client, client_url, &client_path, client_hash).await?;

    let mut library_tasks: Vec<(String, PathBuf, Option<String>)> = Vec::new();
    if let Some(libraries) = metadata["libraries"].as_array() {
        for library in libraries {
            if let Some(artifact) = library["downloads"]["artifact"].as_object() {
                if let (Some(url), Some(path)) = (artifact.get("url").and_then(|v| v.as_str()), artifact.get("path").and_then(|v| v.as_str())) {
                    library_tasks.push((
                        url.to_owned(),
                        root.join("libraries").join(path),
                        artifact.get("sha1").and_then(|v| v.as_str()).map(str::to_owned)
                    ));
                }
            }
        }
    }
    let libraries_downloaded = library_tasks.len();
    stream::iter(library_tasks)
        .map(|(url, path, hash)| {
            let client = client.clone();
            async move { download_file(&client, &url, &path, hash.as_deref()).await }
        })
        .buffer_unordered(10)
        .try_collect::<Vec<_>>().await?;

    let mut assets_downloaded = 0usize;
    if let Some(asset_index) = metadata["assetIndex"].as_object() {
        let index_id = asset_index.get("id").and_then(|v| v.as_str()).ok_or("Asset index ID is missing")?;
        let index_url = asset_index.get("url").and_then(|v| v.as_str()).ok_or("Asset index URL is missing")?;
        let index_path = root.join("assets").join("indexes").join(format!("{index_id}.json"));
        download_file(&client, index_url, &index_path, asset_index.get("sha1").and_then(|v| v.as_str())).await?;
        let index_bytes = fs::read(&index_path).map_err(|error| format!("Could not read asset index: {error}"))?;
        let index_json: serde_json::Value = serde_json::from_slice(&index_bytes)
            .map_err(|error| format!("Could not parse asset index: {error}"))?;
        let mut assets: Vec<(String, PathBuf, Option<String>)> = Vec::new();
        if let Some(objects) = index_json["objects"].as_object() {
            for object in objects.values() {
                if let (Some(hash), Some(size)) = (object["hash"].as_str(), object["size"].as_u64()) {
                    if size == 0 { continue; }
                    let prefix = &hash[0..2];
                    let url = format!("https://resources.download.minecraft.net/{prefix}/{hash}");
                    let path = root.join("assets").join("objects").join(prefix).join(hash);
                    assets.push((url, path, Some(hash.to_owned())));
                }
            }
        }
        assets_downloaded = assets.len();
        stream::iter(assets)
            .map(|(url, path, hash)| {
                let client = client.clone();
                async move { download_file(&client, &url, &path, hash.as_deref()).await }
            })
            .buffer_unordered(16)
            .try_collect::<Vec<_>>().await?;
    }

    Ok(InstallResult {
        version: instance.version,
        libraries_downloaded,
        assets_downloaded,
        game_directory: root.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
fn get_java_info() -> JavaInfo {
    let candidates = {
        let mut paths = vec![PathBuf::from("java")];
        if let Ok(home) = std::env::var("JAVA_HOME") {
            paths.insert(0, PathBuf::from(home).join("bin").join(if cfg!(windows) { "java.exe" } else { "java" }));
        }
        paths
    };
    for candidate in candidates {
        if let Ok(output) = Command::new(&candidate).arg("-version").output() {
            if output.status.success() || !output.stderr.is_empty() {
                let text = String::from_utf8_lossy(&output.stderr).to_string()
                    + &String::from_utf8_lossy(&output.stdout);
                let version = text.lines().find(|line| line.contains("version"))
                    .unwrap_or("Java version detected").trim().to_owned();
                return JavaInfo {
                    available: true,
                    version: Some(version),
                    executable: Some(candidate.to_string_lossy().into_owned()),
                    message: "Java is available. The required major version still depends on the Minecraft release.".into(),
                };
            }
        }
    }
    JavaInfo {
        available: false,
        version: None,
        executable: None,
        message: "Java was not found. Install a compatible Java runtime and add it to PATH or JAVA_HOME.".into(),
    }
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
            install_version,
            get_java_info,
            get_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running Crescent Client");
}
