use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct FavoriteItem {
    pub comic_id: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub image: String,
    pub tags: Vec<String>,
    pub favorited_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteInput {
    pub title: String,
    pub author: String,
    pub description: String,
    pub image: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub favorited_at: Option<i64>,
}

#[derive(Clone)]
pub struct FavoriteService {
    db: SqlitePool,
}

impl FavoriteService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn list(&self) -> anyhow::Result<Vec<FavoriteItem>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, i64)>(
            "SELECT comic_id, title, author, description, image, tags, favorited_at \
             FROM favorites ORDER BY favorited_at DESC",
        )
        .fetch_all(&self.db)
        .await?;

        rows.into_iter()
            .map(
                |(comic_id, title, author, description, image, tags, favorited_at)| {
                    Ok(FavoriteItem {
                        comic_id,
                        title,
                        author,
                        description,
                        image,
                        tags: serde_json::from_str(&tags)?,
                        favorited_at,
                    })
                },
            )
            .collect()
    }

    pub async fn upsert(&self, comic_id: &str, input: FavoriteInput) -> anyhow::Result<()> {
        let favorited_at = input
            .favorited_at
            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
        sqlx::query(
            "INSERT INTO favorites \
             (comic_id, title, author, description, image, tags, favorited_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(comic_id) DO UPDATE SET \
             title = excluded.title, author = excluded.author, \
             description = excluded.description, image = excluded.image, \
             tags = excluded.tags, favorited_at = excluded.favorited_at",
        )
        .bind(comic_id)
        .bind(input.title)
        .bind(input.author)
        .bind(input.description)
        .bind(input.image)
        .bind(serde_json::to_string(&input.tags)?)
        .bind(favorited_at)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn import(&self, items: Vec<(String, FavoriteInput)>) -> anyhow::Result<()> {
        let mut transaction = self.db.begin().await?;

        for (comic_id, input) in items {
            let favorited_at = input
                .favorited_at
                .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
            sqlx::query(
                "INSERT INTO favorites \
                 (comic_id, title, author, description, image, tags, favorited_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?) \
                 ON CONFLICT(comic_id) DO NOTHING",
            )
            .bind(comic_id)
            .bind(input.title)
            .bind(input.author)
            .bind(input.description)
            .bind(input.image)
            .bind(serde_json::to_string(&input.tags)?)
            .bind(favorited_at)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        Ok(())
    }

    pub async fn remove(&self, comic_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM favorites WHERE comic_id = ?")
            .bind(comic_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn clear(&self) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM favorites")
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{FavoriteInput, FavoriteService};
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn persists_orders_and_removes_instance_favorites() {
        let service = test_service().await;
        service
            .upsert("1", test_input("One", 10))
            .await
            .expect("insert first favorite");
        service
            .upsert("2", test_input("Two", 20))
            .await
            .expect("insert second favorite");

        let items = service.list().await.expect("list favorites");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].comic_id, "2");
        assert_eq!(items[1].comic_id, "1");

        service.remove("2").await.expect("remove favorite");
        assert_eq!(service.list().await.expect("list after remove").len(), 1);

        service.clear().await.expect("clear favorites");
        assert!(service.list().await.expect("list after clear").is_empty());
    }

    #[tokio::test]
    async fn import_does_not_overwrite_existing_server_favorites() {
        let service = test_service().await;
        service
            .upsert("1", test_input("Server", 20))
            .await
            .expect("insert server favorite");
        service
            .import(vec![("1".into(), test_input("Browser", 10))])
            .await
            .expect("import browser favorite");

        let items = service.list().await.expect("list favorites");
        assert_eq!(items[0].title, "Server");
        assert_eq!(items[0].favorited_at, 20);
    }

    async fn test_service() -> FavoriteService {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect in-memory sqlite");
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .expect("run migrations");
        FavoriteService::new(db)
    }

    fn test_input(title: &str, favorited_at: i64) -> FavoriteInput {
        FavoriteInput {
            title: title.into(),
            author: "Author".into(),
            description: "Description".into(),
            image: "cover.jpg".into(),
            tags: vec!["Tag".into()],
            favorited_at: Some(favorited_at),
        }
    }
}
