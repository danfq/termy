use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{LazyLock, Mutex},
};

use super::DEFAULT_CONFIG;

static CONFIG_CHANGE_SUBSCRIBERS: LazyLock<Mutex<Vec<flume::Sender<()>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

pub(crate) fn notify_config_changed() {
    let Ok(mut subscribers) = CONFIG_CHANGE_SUBSCRIBERS.lock() else {
        return;
    };
    subscribers.retain(|tx| tx.send(()).is_ok());
}

pub fn subscribe_config_changes() -> flume::Receiver<()> {
    let (tx, rx) = flume::unbounded();
    if let Ok(mut subscribers) = CONFIG_CHANGE_SUBSCRIBERS.lock() {
        subscribers.push(tx);
    }
    rx
}

pub fn ensure_config_file() -> Option<PathBuf> {
    let path = termy_config_core::config_path()?;
    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, DEFAULT_CONFIG);
    }
    Some(path)
}

pub fn open_config_file() {
    let Some(path) = ensure_config_file() else {
        return;
    };

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(&path).status();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(&path).status();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("cmd")
            .args(["/C", "start", "", path.to_string_lossy().as_ref()])
            .status();
    }
}
