use crate::models::Filtro;
use crate::models::Partido;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

pub fn get_data_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();

    let current_dir = std::env::current_dir().unwrap_or_default();

    if exe_dir.join("partidos.json").exists() && exe_dir.join("equipos.json").exists() {
        return exe_dir;
    }

    current_dir
}

pub fn find_file_path(filename: &str) -> PathBuf {
    let data_dir = get_data_dir();
    let data_path = data_dir.join(filename);
    let current_path = PathBuf::from(filename);

    if data_path.exists() {
        data_path
    } else if current_path.exists() {
        current_path
    } else {
        data_path
    }
}

pub fn cargar_filtros() -> Vec<Filtro> {
    let final_path = find_file_path("equipos.json");

    if let Ok(data) = fs::read_to_string(&final_path) {
        if let Ok(config) = serde_json::from_str::<crate::models::EquipoConfig>(&data) {
            return config.filtros;
        }
    }

    vec![crate::models::Filtro {
        nombre: "Todos".to_string(),
        buscar: "".to_string(),
        categoria: "".to_string(),
    }]
}

pub fn cargar_partidos() -> Vec<Partido> {
    let final_path = find_file_path("partidos.json");

    if let Ok(data) = fs::read_to_string(&final_path) {
        if let Ok(partidos) = serde_json::from_str::<Vec<Partido>>(&data) {
            return partidos;
        }
    }
    vec![]
}

pub fn scrape_partidos() -> Result<Vec<Partido>, String> {
    let data_dir = get_data_dir();
    let scrape_script = data_dir.join("scrape-hockey.js");
    let current_script = PathBuf::from("scrape-hockey.js");

    let script_path = if scrape_script.exists() {
        scrape_script
    } else if current_script.exists() {
        current_script
    } else {
        return Err("No se encontró scrape-hockey.js".to_string());
    };

    let output = std::process::Command::new("node")
        .arg(script_path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error ejecutando node: {}", stderr));
    }

    let part_path = find_file_path("partidos.json");
    let data =
        fs::read_to_string(&part_path).map_err(|e| format!("Error leyendo archivo: {}", e))?;

    let partidos: Vec<Partido> =
        serde_json::from_str(&data).map_err(|e| format!("Error parseando JSON: {}", e))?;

    Ok(partidos)
}

pub fn download_partidos() -> Result<Vec<Partido>, String> {
    let url = "https://raw.githubusercontent.com/tortajet/fecapa-explorer/main/partidos.json";

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(url).send().map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let data = response.text().map_err(|e| e.to_string())?;
    let partidos: Vec<Partido> = serde_json::from_str(&data).map_err(|e| e.to_string())?;

    Ok(partidos)
}

pub fn is_android() -> bool {
    std::env::consts::OS == "android"
        || std::env::var("ANDROID_ROOT").is_ok()
        || std::env::var("TERMUX_VERSION").is_ok()
}
