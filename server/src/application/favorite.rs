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
}

#[derive(Clone)]
pub struct FavoriteService {
    db: SqlitePool,
}

impl FavoriteService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn list(
        &self,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<FavoriteItem>, i64)> {
        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM favorites")
            .fetch_one(&self.db)
            .await?;
        let offset = i64::from(page - 1) * i64::from(page_size);
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, i64)>(
            "SELECT comic_id, title, author, description, image, tags, favorited_at \
             FROM favorites ORDER BY favorited_at DESC, comic_id DESC LIMIT ? OFFSET ?",
        )
        .bind(i64::from(page_size))
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let items = rows
            .into_iter()
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
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok((items, total))
    }

    pub async fn contains(&self, comic_id: &str) -> anyhow::Result<bool> {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT EXISTS(SELECT 1 FROM favorites WHERE comic_id = ?)",
        )
        .bind(comic_id)
        .fetch_one(&self.db)
        .await?;
        Ok(exists != 0)
    }

    pub async fn upsert(
        &self,
        comic_id: &str,
        input: FavoriteInput,
    ) -> anyhow::Result<FavoriteItem> {
        let favorited_at = chrono::Utc::now().timestamp_millis();
        let FavoriteInput {
            title,
            author,
            description,
            image,
            tags,
        } = input;
        let serialized_tags = serde_json::to_string(&tags)?;
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
        .bind(&title)
        .bind(&author)
        .bind(&description)
        .bind(&image)
        .bind(serialized_tags)
        .bind(favorited_at)
        .execute(&self.db)
        .await?;
        Ok(FavoriteItem {
            comic_id: comic_id.to_string(),
            title,
            author,
            description,
            image,
            tags,
            favorited_at,
        })
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
            .upsert("1", test_input("One"))
            .await
            .expect("insert first favorite");
        service
            .upsert("2", test_input("Two"))
            .await
            .expect("insert second favorite");

        let (items, total) = service.list(1, 1).await.expect("list first page");
        assert_eq!(total, 2);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].comic_id, "2");
        let (items, total) = service.list(2, 1).await.expect("list second page");
        assert_eq!(total, 2);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].comic_id, "1");
        assert!(service.contains("1").await.expect("find favorite"));
        assert!(!service.contains("3").await.expect("miss favorite"));

        service.remove("2").await.expect("remove favorite");
        let (items, total) = service.list(1, 20).await.expect("list after remove");
        assert_eq!(total, 1);
        assert_eq!(items.len(), 1);

        service.clear().await.expect("clear favorites");
        let (items, total) = service.list(1, 20).await.expect("list after clear");
        assert_eq!(total, 0);
        assert!(items.is_empty());
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

    fn test_input(title: &str) -> FavoriteInput {
        FavoriteInput {
            title: title.into(),
            author: "Author".into(),
            description: "Description".into(),
            image: "cover.jpg".into(),
            tags: vec!["Tag".into()],
        }
    }
}
