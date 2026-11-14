#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures_util::{stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    io::copy,
    process::Stdio,
};
use tauri::{AppHandle, Manager};
use uuid::Uuid;
use zip::ZipArchive;

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


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LaunchInput {
    instance_id: String,
    player_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchResult {
    process_id: u32,
    player_name: String,
    version: String,
    authentication: String,
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
    if destination.exists() {
        fs::remove_file(destination)
            .map_err(|error| format!("Could not replace invalid cached file: {error}"))?;
    }
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
    let mut native_jars: Vec<PathBuf> = Vec::new();
    if let Some(libraries) = metadata["libraries"].as_array() {
        for library in libraries {
            if !library_allowed(library) { continue; }
            if let Some(artifact) = library["downloads"]["artifact"].as_object() {
                if let (Some(url), Some(path)) = (artifact.get("url").and_then(|v| v.as_str()), artifact.get("path").and_then(|v| v.as_str())) {
                    library_tasks.push((
                        url.to_owned(),
                        root.join("libraries").join(path),
                        artifact.get("sha1").and_then(|v| v.as_str()).map(str::to_owned)
                    ));
                }
            }
            if let Some(classifiers) = library["downloads"]["classifiers"].as_object() {
                if let Some(key) = native_classifier_key(library) {
                    if let Some(native) = classifiers.get(&key) {
                        if let (Some(url), Some(path)) = (native["url"].as_str(), native["path"].as_str()) {
                            let native_path = root.join("libraries").join(path);
                            native_jars.push(native_path.clone());
                            library_tasks.push((
                                url.to_owned(),
                                native_path,
                                native["sha1"].as_str().map(str::to_owned)
                            ));
                        }
                    }
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

    let natives_dir = version_dir.join(format!("{}-natives", instance.version));
    fs::create_dir_all(&natives_dir).map_err(|error| format!("Could not create natives folder: {error}"))?;
    for native_jar in &native_jars {
        extract_natives(native_jar, &natives_dir)?;
    }

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


fn current_os_name() -> &'static str {
    if cfg!(target_os = "windows") { "windows" }
    else if cfg!(target_os = "macos") { "osx" }
    else { "linux" }
}

fn library_allowed(library: &serde_json::Value) -> bool {
    let Some(rules) = library["rules"].as_array() else { return true; };
    let mut allowed = false;
    for rule in rules {
        let os_matches = rule["os"]["name"].as_str()
            .map(|name| name == current_os_name()).unwrap_or(true);
        if os_matches {
            allowed = rule["action"].as_str() == Some("allow");
        }
    }
    allowed
}

fn native_classifier_key(library: &serde_json::Value) -> Option<String> {
    let template = library["natives"][current_os_name()].as_str()?;
    let arch = if cfg!(target_pointer_width = "64") { "64" } else { "32" };
    Some(template.replace("${arch}", arch))
}

fn extract_natives(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let file = fs::File::open(archive_path)
        .map_err(|error| format!("Could not open native library: {error}"))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| format!("Could not read native archive: {error}"))?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)
            .map_err(|error| format!("Could not read native archive entry: {error}"))?;
        let Some(relative) = entry.enclosed_name().map(|path| path.to_path_buf()) else { continue; };
        if relative.components().any(|part| part.as_os_str() == "META-INF") { continue; }
        let output = destination.join(relative);
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("Could not create native folder: {error}"))?;
        }
        let mut target = fs::File::create(output)
            .map_err(|error| format!("Could not extract native library: {error}"))?;
        copy(&mut entry, &mut target)
            .map_err(|error| format!("Could not extract native library: {error}"))?;
    }
    Ok(())
}

fn offline_uuid(player_name: &str) -> String {
    let digest = md5::compute(format!("OfflinePlayer:{player_name}"));
    let mut bytes = digest.0;
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes).to_string()
}

fn expand_argument(value: &str, replacements: &[(String, String)]) -> String {
    replacements.iter().fold(value.to_owned(), |result, (key, replacement)| {
        result.replace(&format!("${{{key}}}"), replacement)
    })
}

fn argument_values(value: &serde_json::Value) -> Vec<String> {
    if let Some(text) = value.as_str() { return vec![text.to_owned()]; }
    if let Some(values) = value.as_array() {
        return values.iter().filter_map(|item| item.as_str().map(str::to_owned)).collect();
    }
    Vec::new()
}

fn game_argument_allowed(argument: &serde_json::Value) -> bool {
    let Some(rules) = argument["rules"].as_array() else { return true; };
    let mut allowed = false;
    for rule in rules {
        let os_matches = rule["os"]["name"].as_str()
            .map(|name| name == current_os_name()).unwrap_or(true);
        if os_matches { allowed = rule["action"].as_str() == Some("allow"); }
    }
    allowed
}

fn find_java() -> Option<PathBuf> {
    let candidate = if let Ok(home) = std::env::var("JAVA_HOME") {
        PathBuf::from(home).join("bin").join(if cfg!(windows) { "java.exe" } else { "java" })
    } else {
        PathBuf::from(if cfg!(windows) { "java.exe" } else { "java" })
    };
    if Command::new(&candidate).arg("-version").output().is_ok() { Some(candidate) } else { None }
}


#[tauri::command]
fn launch_instance(app: AppHandle, input: LaunchInput) -> Result<LaunchResult, String> {
    let player_name = input.player_name.trim();
    if player_name.is_empty() || player_name.chars().count() > 16
        || !player_name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
        return Err("Enter a Minecraft username using 1–16 letters, numbers, or underscores.".into());
    }
    let java = find_java().ok_or("Java was not found. Install the Java version required by this Minecraft release.")?;
    let instance = list_instances(app.clone())?.into_iter()
        .find(|item| item.id == input.instance_id).ok_or("Instance was not found")?;
    let root = game_root(&app)?;
    let version_dir = root.join("versions").join(&instance.version);
    let metadata_path = version_dir.join(format!("{}.json", instance.version));
    let client_jar = version_dir.join(format!("{}.jar", instance.version));
    if !metadata_path.exists() || !client_jar.exists() {
        return Err("This instance is not installed yet. Install its game files first.".into());
    }
    let metadata: serde_json::Value = serde_json::from_slice(
        &fs::read(&metadata_path).map_err(|error| format!("Could not read version metadata: {error}"))?
    ).map_err(|error| format!("Invalid version metadata: {error}"))?;

    let instance_dir = instances_dir(&app)?.join(&instance.id).join("game");
    fs::create_dir_all(&instance_dir).map_err(|error| format!("Could not create instance game folder: {error}"))?;
    let assets_dir = root.join("assets");
    let asset_index = metadata["assetIndex"]["id"].as_str().unwrap_or(&instance.version);
    let natives_dir = version_dir.join(format!("{}-natives", instance.version));

    let mut classpath: Vec<String> = Vec::new();
    if let Some(libraries) = metadata["libraries"].as_array() {
        for library in libraries {
            if !library_allowed(library) { continue; }
            if let Some(path) = library["downloads"]["artifact"]["path"].as_str() {
                let full_path = root.join("libraries").join(path);
                if full_path.exists() { classpath.push(full_path.to_string_lossy().into_owned()); }
            }
        }
    }
    classpath.push(client_jar.to_string_lossy().into_owned());
    let separator = if cfg!(windows) { ";" } else { ":" };
    let classpath = classpath.join(separator);
    let uuid = offline_uuid(player_name);
    let settings = read_settings(&app)?;
    let replacements = vec![
        ("auth_player_name".to_owned(), player_name.to_owned()),
        ("version_name".to_owned(), instance.version.clone()),
        ("game_directory".to_owned(), instance_dir.to_string_lossy().into_owned()),
        ("assets_root".to_owned(), assets_dir.to_string_lossy().into_owned()),
        ("assets_index_name".to_owned(), asset_index.to_owned()),
        ("auth_uuid".to_owned(), uuid),
        ("auth_access_token".to_owned(), "0".to_owned()),
        ("user_type".to_owned(), "legacy".to_owned()),
        ("version_type".to_owned(), "release".to_owned()),
        ("natives_directory".to_owned(), natives_dir.to_string_lossy().into_owned()),
        ("launcher_name".to_owned(), "Crescent".to_owned()),
        ("launcher_version".to_owned(), "0.2.0".to_owned()),
        ("classpath".to_owned(), classpath.clone()),
        ("classpath_separator".to_owned(), separator.to_owned()),
        ("library_directory".to_owned(), root.join("libraries").to_string_lossy().into_owned()),
    ];

    let mut args: Vec<String> = vec![
        "-Xms512M".into(),
        format!("-Xmx{}G", settings.ram_gb),
        format!("-Djava.library.path={}", natives_dir.to_string_lossy()),
    ];
    if let Some(jvm_args) = metadata["arguments"]["jvm"].as_array() {
        for argument in jvm_args {
            if argument.is_object() {
                if game_argument_allowed(argument) {
                    args.extend(argument_values(&argument["value"]));
                }
            } else {
                args.extend(argument_values(argument));
            }
        }
    }
    args = args.into_iter().map(|arg| expand_argument(&arg, &replacements)).collect();

    let mut game_args: Vec<String> = Vec::new();
    if let Some(arguments) = metadata["arguments"]["game"].as_array() {
        for argument in arguments {
            if argument.is_object() {
                if game_argument_allowed(argument) {
                    game_args.extend(argument_values(&argument["value"]));
                }
            } else {
                game_args.extend(argument_values(argument));
            }
        }
    } else if let Some(legacy) = metadata["minecraftArguments"].as_str() {
        game_args.extend(legacy.split_whitespace().map(str::to_owned));
    } else {
        return Err("This Minecraft version uses an unsupported launch argument format.".into());
    }
    game_args = game_args.into_iter().map(|arg| expand_argument(&arg, &replacements)).collect();

    let has_classpath = args.iter().any(|arg| arg == "-cp" || arg == "-classpath" || arg == "--class-path");
    if !has_classpath {
        args.push("-cp".into());
        args.push(classpath);
    }
    let main_class = metadata["mainClass"].as_str().ok_or("Minecraft main class is missing")?;
    args.push(main_class.into());
    args.extend(game_args);

    fs::create_dir_all(instance_dir.join("logs")).map_err(|error| format!("Could not create logs folder: {error}"))?;
    let log_file = fs::OpenOptions::new().create(true).append(true)
        .open(instance_dir.join("logs").join("latest-crescent.log"))
        .map_err(|error| format!("Could not open game log: {error}"))?;
    let error_log = log_file.try_clone().map_err(|error| format!("Could not open error log: {error}"))?;
    let child = Command::new(&java)
        .args(&args)
        .current_dir(&instance_dir)
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(error_log))
        .spawn()
        .map_err(|error| format!("Could not start Minecraft with Java: {error}"))?;

    Ok(LaunchResult {
        process_id: child.id(),
        player_name: player_name.to_owned(),
        version: instance.version,
        authentication: "Offline profile: only compatible with servers configured to allow offline-mode accounts.".into(),
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
            launch_instance,
            get_java_info,
            get_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running Crescent Client");
}
