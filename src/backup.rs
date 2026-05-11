use chrono::Local;
use rfd::FileDialog;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::PathBuf;

fn default_collection_file() -> Result<PathBuf, String> {
    let mut dir =
        dirs::config_dir().ok_or_else(|| "Could not find your config directory.".to_string())?;

    dir.push("flosskeeper");

    fs::create_dir_all(&dir)
        .map_err(|e| format!("Could not create FlossKeeper config folder: {e}"))?;

    dir.push("flosskeeper_collection.tsv");

    Ok(dir)
}

fn stash_path_config_file() -> Result<PathBuf, String> {
    let mut path = default_collection_file()?;
    path.set_file_name("stash_path.txt");
    Ok(path)
}

pub fn collection_file() -> Result<PathBuf, String> {
    let config_path = stash_path_config_file()?;

    if let Ok(raw) = fs::read_to_string(&config_path) {
        let trimmed = raw.trim();

        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }

    default_collection_file()
}

fn parse_tsv_rows(raw: &str) -> Vec<Value> {
    let mut lines = raw.lines();

    let Some(header_line) = lines.next() else {
        return Vec::new();
    };

    let headers: Vec<String> = header_line
        .split('\t')
        .map(|s| s.trim().to_string())
        .collect();

    let mut rows = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }

        let values: Vec<&str> = line.split('\t').collect();
        let mut obj = Map::new();

        for (idx, header) in headers.iter().enumerate() {
            let value = values.get(idx).copied().unwrap_or("").to_string();
            obj.insert(header.clone(), Value::String(value));
        }

        rows.push(Value::Object(obj));
    }

    rows
}

pub fn export_json_backup() -> Result<String, String> {
    let collection_path = collection_file()?;

    if !collection_path.exists() {
        return Err(format!(
            "Collection file not found:\n{}",
            collection_path.display()
        ));
    }

    let raw_tsv = fs::read_to_string(&collection_path)
        .map_err(|e| format!("Could not read collection file: {e}"))?;

    let rows = parse_tsv_rows(&raw_tsv);

    let default_name = format!(
        "flosskeeper_backup_{}.json",
        Local::now().format("%Y-%m-%d_%H-%M-%S")
    );

    let Some(save_path) = FileDialog::new()
        .set_title("Export FlossKeeper JSON Backup")
        .set_file_name(&default_name)
        .add_filter("JSON backup", &["json"])
        .save_file()
    else {
        return Ok("Export cancelled.".to_string());
    };

    let backup = json!({
        "app": "FlossKeeper",
        "backup_version": 1,
        "created_at": Local::now().to_rfc3339(),
        "collection_file": collection_path.to_string_lossy(),
        "format": "raw_tsv_plus_parsed_rows",
        "raw_tsv": raw_tsv,
        "rows": rows
    });

    let pretty = serde_json::to_string_pretty(&backup)
        .map_err(|e| format!("Could not create JSON backup: {e}"))?;

    fs::write(&save_path, pretty).map_err(|e| format!("Could not write backup file: {e}"))?;

    Ok(format!(
        "JSON backup saved:\n{}\n\nRows exported: {}",
        save_path.display(),
        backup["rows"].as_array().map(|r| r.len()).unwrap_or(0)
    ))
}

pub fn restore_json_backup() -> Result<String, String> {
    let collection_path = collection_file()?;

    let Some(open_path) = FileDialog::new()
        .set_title("Restore FlossKeeper JSON Backup")
        .add_filter("JSON backup", &["json"])
        .pick_file()
    else {
        return Ok("Restore cancelled.".to_string());
    };

    let text =
        fs::read_to_string(&open_path).map_err(|e| format!("Could not read backup file: {e}"))?;

    let data: Value =
        serde_json::from_str(&text).map_err(|e| format!("Backup is not valid JSON: {e}"))?;

    if data.get("app").and_then(|v| v.as_str()) != Some("FlossKeeper") {
        return Err("This does not look like a FlossKeeper backup.".to_string());
    }

    let Some(raw_tsv) = data.get("raw_tsv").and_then(|v| v.as_str()) else {
        return Err("Backup is missing raw_tsv data.".to_string());
    };

    let mut safety_copy_message = String::new();

    if collection_path.exists() {
        let stamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let safety_path = collection_path.with_file_name(format!(
            "flosskeeper_collection_before_restore_{}.tsv",
            stamp
        ));

        fs::copy(&collection_path, &safety_path)
            .map_err(|e| format!("Could not make safety copy before restore: {e}"))?;

        safety_copy_message = format!("\n\nSafety copy made:\n{}", safety_path.display());
    }

    fs::write(&collection_path, raw_tsv)
        .map_err(|e| format!("Could not restore collection file: {e}"))?;

    Ok(format!(
        "Restore complete from:\n{}{}",
        open_path.display(),
        safety_copy_message
    ))
}

pub fn export_plain_tsv_copy() -> Result<String, String> {
    let collection_path = collection_file()?;

    if !collection_path.exists() {
        return Err(format!(
            "Collection file not found:\n{}",
            collection_path.display()
        ));
    }

    let raw_tsv = fs::read_to_string(&collection_path)
        .map_err(|e| format!("Could not read collection file: {e}"))?;

    let default_name = format!(
        "flosskeeper_collection_export_{}.tsv",
        Local::now().format("%Y-%m-%d_%H-%M-%S")
    );

    let Some(save_path) = FileDialog::new()
        .set_title("Export Plain TSV Copy")
        .set_file_name(&default_name)
        .add_filter("TSV file", &["tsv"])
        .save_file()
    else {
        return Ok("TSV export cancelled.".to_string());
    };

    fs::write(&save_path, raw_tsv).map_err(|e| format!("Could not write TSV export: {e}"))?;

    Ok(format!("TSV copy saved:\n{}", save_path.display()))
}

pub fn export_text_file(
    title: &str,
    default_name: &str,
    filter_name: &str,
    extensions: &[&str],
    contents: &str,
) -> Result<String, String> {
    let Some(save_path) = FileDialog::new()
        .set_title(title)
        .set_file_name(default_name)
        .add_filter(filter_name, extensions)
        .save_file()
    else {
        return Ok("Export cancelled.".to_string());
    };

    fs::write(&save_path, contents).map_err(|e| format!("Could not write export file: {e}"))?;

    Ok(format!("Export saved:\n{}", save_path.display()))
}
