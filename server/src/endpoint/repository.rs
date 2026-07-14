use super::EndpointMode;

const MODE_KEY: &str = "endpoint_mode";
const SELECTED_KEY: &str = "endpoint_selected";

#[derive(Clone)]
pub(super) struct EndpointRepository {
    db: sqlx::SqlitePool,
}

impl EndpointRepository {
    pub(super) fn new(db: sqlx::SqlitePool) -> Self {
        Self { db }
    }

    pub(super) async fn load_mode(&self) -> anyhow::Result<EndpointMode> {
        Ok(self
            .load(MODE_KEY)
            .await?
            .and_then(|value| serde_json::from_str(&value).ok())
            .unwrap_or_default())
    }

    pub(super) async fn load_selected(&self) -> anyhow::Result<Option<String>> {
        self.load(SELECTED_KEY).await
    }

    pub(super) async fn save_selection(
        &self,
        mode: EndpointMode,
        selected: Option<&str>,
    ) -> anyhow::Result<()> {
        self.save(MODE_KEY, &serde_json::to_string(&mode)?).await?;
        match selected {
            Some(endpoint) => self.save(SELECTED_KEY, endpoint).await,
            None => self.delete(SELECTED_KEY).await,
        }
    }

    async fn load(&self, key: &str) -> anyhow::Result<Option<String>> {
        Ok(
            sqlx::query_scalar::<_, String>("SELECT value FROM app_settings WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.db)
                .await?,
        )
    }

    async fn save(&self, key: &str, value: &str) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO app_settings (key, value, updated_at) VALUES (?, ?, ?) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(value)
        .bind(chrono::Utc::now().timestamp())
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM app_settings WHERE key = ?")
            .bind(key)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}
