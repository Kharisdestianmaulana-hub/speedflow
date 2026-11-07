use sqlx::SqlitePool;
use crate::storage::models::{AppSettings, DownloadRow, SegmentRow};

#[derive(Clone)]
pub struct DownloadRepository {
    pool: SqlitePool,
}

impl DownloadRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert_download(&self, row: &DownloadRow) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO downloads (
                id, url, filename, destination_path, total_bytes, downloaded_bytes,
                status, supports_range, etag, connections, error_message, checksum, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&row.id)
        .bind(&row.url)
        .bind(&row.filename)
        .bind(&row.destination_path)
        .bind(row.total_bytes)
        .bind(row.downloaded_bytes)
        .bind(&row.status)
        .bind(row.supports_range)
        .bind(&row.etag)
        .bind(row.connections)
        .bind(&row.error_message)
        .bind(&row.checksum)
        .bind(&row.created_at)
        .bind(&row.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_status(&self, id: &str, status: &str, updated_at: &str) -> anyhow::Result<()> {
        sqlx::query("UPDATE downloads SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(updated_at)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_url(&self, id: &str, url: &str, updated_at: &str) -> anyhow::Result<()> {
        sqlx::query("UPDATE downloads SET url = ?, updated_at = ? WHERE id = ?")
            .bind(url)
            .bind(updated_at)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_progress(&self, id: &str, downloaded_bytes: i64, updated_at: &str) -> anyhow::Result<()> {
        sqlx::query("UPDATE downloads SET downloaded_bytes = ?, updated_at = ? WHERE id = ?")
            .bind(downloaded_bytes)
            .bind(updated_at)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_completed(
        &self,
        id: &str,
        downloaded_bytes: i64,
        checksum: Option<&str>,
        updated_at: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE downloads SET status = 'completed', downloaded_bytes = ?, checksum = ?, updated_at = ? WHERE id = ?"
        )
        .bind(downloaded_bytes)
        .bind(checksum)
        .bind(updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_download(&self, id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM downloads WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn clear_history(&self) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM downloads WHERE status IN ('completed', 'failed', 'cancelled')")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_all_downloads(&self) -> anyhow::Result<Vec<DownloadRow>> {
        let rows = sqlx::query_as::<_, DownloadRow>("SELECT * FROM downloads ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn get_history(&self) -> anyhow::Result<Vec<DownloadRow>> {
        let rows = sqlx::query_as::<_, DownloadRow>(
            "SELECT * FROM downloads WHERE status IN ('completed', 'failed') ORDER BY updated_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn save_dynamic_segments(
        &self,
        task_id: &str,
        segments: &[(u32, u64, u64)],
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM segments WHERE task_id = ?")
            .bind(task_id)
            .execute(&mut *tx)
            .await?;

        for &(idx, start_b, end_b) in segments {
            sqlx::query(
                "INSERT INTO segments (task_id, segment_index, start_byte, end_byte, downloaded_bytes, is_completed) VALUES (?, ?, ?, ?, 0, 0)",
            )
            .bind(task_id)
            .bind(idx as i32)
            .bind(start_b as i64)
            .bind(end_b as i64)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn load_dynamic_segments(&self, task_id: &str) -> anyhow::Result<Vec<(u32, u64, u64)>> {
        let rows = sqlx::query_as::<_, SegmentRow>(
            "SELECT * FROM segments WHERE task_id = ? ORDER BY start_byte ASC",
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.segment_index as u32, r.start_byte as u64, r.end_byte as u64))
            .collect())
    }

    pub async fn save_settings(&self, settings: &AppSettings) -> anyhow::Result<()> {
        let json = serde_json::to_string(settings)?;
        sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('app_settings', ?)")
            .bind(json)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn load_settings(&self) -> Option<AppSettings> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT value FROM settings WHERE key = 'app_settings'")
                .fetch_optional(&self.pool)
                .await
                .ok()
                .flatten();

        row.and_then(|(val,)| serde_json::from_str::<AppSettings>(&val).ok())
    }
}
