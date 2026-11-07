use directories::ProjectDirs;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::str::FromStr;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn init(custom_path: Option<PathBuf>) -> anyhow::Result<Self> {
        let db_path = match custom_path {
            Some(p) => p,
            None => {
                if let Some(proj_dirs) = ProjectDirs::from("com", "speedflow", "SpeedFlow") {
                    let data_dir = proj_dirs.data_local_dir();
                    tokio::fs::create_dir_all(data_dir).await?;
                    data_dir.join("speedflow.db")
                } else {
                    PathBuf::from("speedflow.db")
                }
            }
        };

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let schema = include_str!("../../migrations/001_initial_schema.sql");
        sqlx::query(schema).execute(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
