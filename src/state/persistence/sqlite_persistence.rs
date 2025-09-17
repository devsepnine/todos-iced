use sqlx::{sqlite::SqliteConnectOptions, SqlitePool, Row};
use std::str::FromStr;
use uuid::Uuid;
use crate::task::Task;
use super::{SavedState, Filter, LoadError, SaveError};

pub struct SqlitePersistence {
    pool: SqlitePool,
}

impl SqlitePersistence {
    pub async fn new() -> Result<Self, String> {
        let db_path = Self::db_path();
        
        // Create directory if it doesn't exist
        if let Some(dir) = db_path.parent() {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))
            .map_err(|e| format!("Failed to parse SQLite connection string: {}", e))?
            .create_if_missing(true);
            
        let pool = SqlitePool::connect_with(options)
            .await
            .map_err(|e| format!("Failed to connect to database: {}", e))?;

        let persistence = Self { pool };
        persistence.init_tables().await?;
        
        Ok(persistence)
    }

    fn db_path() -> std::path::PathBuf {
        let mut path = if let Some(project_dirs) = directories::ProjectDirs::from("rs", "Iced", "Todos") {
            project_dirs.data_dir().into()
        } else {
            std::env::current_dir().unwrap_or_default()
        };
        
        path.push("todos.db");
        path
    }

    async fn init_tables(&self) -> Result<(), String> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS app_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                input_value TEXT NOT NULL DEFAULT '',
                filter INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create app_state table: {}", e))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT FALSE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create tasks table: {}", e))?;

        // Initialize default app state if not exists
        sqlx::query(
            "INSERT OR IGNORE INTO app_state (id, input_value, filter) VALUES (1, '', 0)"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to initialize app state: {}", e))?;

        Ok(())
    }

    pub async fn load(&self) -> Result<SavedState, LoadError> {
        // Load app state
        let app_state_row = sqlx::query("SELECT input_value, filter FROM app_state WHERE id = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|_| LoadError::File)?;

        let input_value: String = app_state_row.get("input_value");
        let filter_int: i64 = app_state_row.get("filter");
        let filter = match filter_int {
            1 => Filter::Active,
            2 => Filter::Completed,
            _ => Filter::All,
        };

        // Load tasks
        let task_rows = sqlx::query("SELECT id, description, completed FROM tasks ORDER BY created_at")
            .fetch_all(&self.pool)
            .await
            .map_err(|_| LoadError::File)?;

        let mut tasks = Vec::new();
        for row in task_rows {
            let id_str: String = row.get("id");
            let id = Uuid::parse_str(&id_str).map_err(|_| LoadError::Format)?;
            let description: String = row.get("description");
            let completed: bool = row.get("completed");

            tasks.push(Task::from_parts(id, description, completed));
        }

        Ok(SavedState {
            input_value,
            filter,
            tasks,
        })
    }

    pub async fn save(&self, state: SavedState) -> Result<(), SaveError> {
        let mut tx = self.pool.begin().await.map_err(|_| SaveError::Write)?;

        // Save app state
        let filter_int = match state.filter {
            Filter::All => 0,
            Filter::Active => 1,
            Filter::Completed => 2,
        };

        sqlx::query("UPDATE app_state SET input_value = ?, filter = ? WHERE id = 1")
            .bind(&state.input_value)
            .bind(filter_int)
            .execute(&mut *tx)
            .await
            .map_err(|_| SaveError::Write)?;

        // Clear existing tasks
        sqlx::query("DELETE FROM tasks")
            .execute(&mut *tx)
            .await
            .map_err(|_| SaveError::Write)?;

        // Save tasks
        for task in &state.tasks {
            sqlx::query("INSERT INTO tasks (id, description, completed) VALUES (?, ?, ?)")
                .bind(task.id().to_string())
                .bind(task.description())
                .bind(task.completed())
                .execute(&mut *tx)
                .await
                .map_err(|_| SaveError::Write)?;
        }

        tx.commit().await.map_err(|_| SaveError::Write)?;
        
        Ok(())
    }
}