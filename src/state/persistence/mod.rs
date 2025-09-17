use super::Filter;
use crate::task::Task;
use serde::{Deserialize, Serialize};

pub mod sqlite_persistence;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    pub input_value: String,
    pub filter: Filter,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone)]
pub enum LoadError {
    File,
    Format,
}

#[derive(Debug, Clone)]
pub enum SaveError {
    Write,
}

#[cfg(not(target_arch = "wasm32"))]
impl SavedState {
    pub fn path() -> std::path::PathBuf {
        let mut path =
            if let Some(project_dirs) = directories::ProjectDirs::from("rs", "Iced", "Todos") {
                project_dirs.data_dir().into()
            } else {
                std::env::current_dir().unwrap_or_default()
            };

        path.push("todos.json");
        path
    }

    pub async fn load() -> Result<SavedState, LoadError> {
        let persistence = sqlite_persistence::SqlitePersistence::new()
            .await
            .map_err(|_| LoadError::File)?;
        persistence.load().await
    }

    pub async fn save(self) -> Result<(), SaveError> {
        let persistence = sqlite_persistence::SqlitePersistence::new()
            .await
            .map_err(|_| SaveError::Write)?;
        persistence.save(self).await
    }
}

#[cfg(target_arch = "wasm32")]
impl SavedState {
    fn storage() -> Option<web_sys::Storage> {
        let window = web_sys::window()?;
        window.local_storage().ok()?
    }

    pub async fn load() -> Result<SavedState, LoadError> {
        let storage = Self::storage().ok_or(LoadError::File)?;

        let contents = storage
            .get_item("state")
            .map_err(|_| LoadError::File)?
            .ok_or(LoadError::File)?;

        serde_json::from_str(&contents).map_err(|_| LoadError::Format)
    }

    pub async fn save(self) -> Result<(), SaveError> {
        let storage = Self::storage().ok_or(SaveError::Write)?;

        let json = serde_json::to_string_pretty(&self).map_err(|_| SaveError::Format)?;

        storage
            .set_item("state", &json)
            .map_err(|_| SaveError::Write)?;

        wasmtimer::tokio::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}
