use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct ReadingHistoryItem {
    pub comic_id: String,
    pub title: String,
    pub author: String,
    pub image: String,
    pub chapter_id: String,
    pub chapter_title: String,
    pub page_index: i64,
    pub page_count: i64,
    pub last_read_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingHistoryInput {
    pub title: String,
    pub author: String,
    pub image: String,
    pub chapter_id: String,
    pub chapter_title: String,
    pub page_index: i64,
    pub page_count: i64,
    #[serde(default)]
    pub last_read_at: Option<i64>,
}

#[derive(Clone)]
pub struct ReadingHistoryService {
    db: SqlitePool,
}

impl ReadingHistoryService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn list(
        &self,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ReadingHistoryItem>, i64)> {
        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM reading_history")
            .fetch_one(&self.db)
            .await?;
        let offset = i64::from(page - 1) * i64::from(page_size);
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                i64,
                i64,
                i64,
            ),
        >(
            "SELECT comic_id, title, author, image, chapter_id, chapter_title, \
             page_index, page_count, last_read_at \
             FROM reading_history ORDER BY last_read_at DESC, comic_id DESC LIMIT ? OFFSET ?",
        )
        .bind(i64::from(page_size))
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let items = rows
            .into_iter()
            .map(
                |(
                    comic_id,
                    title,
                    author,
                    image,
                    chapter_id,
                    chapter_title,
                    page_index,
                    page_count,
                    last_read_at,
                )| ReadingHistoryItem {
                    comic_id,
                    title,
                    author,
                    image,
                    chapter_id,
                    chapter_title,
                    page_index,
                    page_count,
                    last_read_at,
                },
            )
            .collect();
        Ok((items, total))
    }

    pub async fn get(&self, comic_id: &str) -> anyhow::Result<Option<ReadingHistoryItem>> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                i64,
                i64,
                i64,
            ),
        >(
            "SELECT comic_id, title, author, image, chapter_id, chapter_title, \
             page_index, page_count, last_read_at \
             FROM reading_history WHERE comic_id = ?",
        )
        .bind(comic_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(
            |(
                comic_id,
                title,
                author,
                image,
                chapter_id,
                chapter_title,
                page_index,
                page_count,
                last_read_at,
            )| ReadingHistoryItem {
                comic_id,
                title,
                author,
                image,
                chapter_id,
                chapter_title,
                page_index,
                page_count,
                last_read_at,
            },
        ))
    }

    pub async fn upsert(&self, comic_id: &str, input: ReadingHistoryInput) -> anyhow::Result<()> {
        let last_read_at = input
            .last_read_at
            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
        let mut transaction = self.db.begin().await?;
        persist_item(&mut transaction, comic_id, input, last_read_at).await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn remove_many(&self, comic_ids: &[String]) -> anyhow::Result<()> {
        let mut transaction = self.db.begin().await?;
        for comic_id in comic_ids {
            sqlx::query("DELETE FROM reading_history WHERE comic_id = ?")
                .bind(comic_id)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn clear(&self) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM reading_history")
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

async fn persist_item(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    comic_id: &str,
    input: ReadingHistoryInput,
    last_read_at: i64,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO reading_history \
         (comic_id, title, author, image, chapter_id, chapter_title, \
          page_index, page_count, last_read_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(comic_id) DO UPDATE SET \
         title = excluded.title, author = excluded.author, image = excluded.image, \
         chapter_id = excluded.chapter_id, chapter_title = excluded.chapter_title, \
         page_index = excluded.page_index, page_count = excluded.page_count, \
         last_read_at = excluded.last_read_at \
         WHERE excluded.last_read_at >= reading_history.last_read_at",
    )
    .bind(comic_id)
    .bind(input.title)
    .bind(input.author)
    .bind(input.image)
    .bind(input.chapter_id)
    .bind(input.chapter_title)
    .bind(input.page_index)
    .bind(input.page_count)
    .bind(last_read_at)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ReadingHistoryInput, ReadingHistoryService};
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn newer_progress_wins_and_items_are_ordered_by_last_read_time() {
        let service = test_service().await;
        service
            .upsert("1", test_input("chapter-old", 1, 10))
            .await
            .expect("insert old progress");
        service
            .upsert("2", test_input("chapter-two", 2, 20))
            .await
            .expect("insert second progress");
        service
            .upsert("1", test_input("chapter-new", 3, 30))
            .await
            .expect("update progress");

        let (items, total) = service.list(1, 1).await.expect("list first page");
        assert_eq!(total, 2);
        assert_eq!(items[0].comic_id, "1");
        assert_eq!(items[0].chapter_id, "chapter-new");
        assert_eq!(items[0].page_index, 3);
        let (items, total) = service.list(2, 1).await.expect("list second page");
        assert_eq!(total, 2);
        assert_eq!(items[0].comic_id, "2");
        let item = service
            .get("1")
            .await
            .expect("get history")
            .expect("history item");
        assert_eq!(item.chapter_id, "chapter-new");
        assert_eq!(item.page_index, 3);
        assert!(service.get("3").await.expect("missing history").is_none());
    }

    async fn test_service() -> ReadingHistoryService {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect in-memory sqlite");
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .expect("run migrations");
        ReadingHistoryService::new(db)
    }

    fn test_input(chapter_id: &str, page_index: i64, last_read_at: i64) -> ReadingHistoryInput {
        ReadingHistoryInput {
            title: "Title".into(),
            author: "Author".into(),
            image: "cover.jpg".into(),
            chapter_id: chapter_id.into(),
            chapter_title: "Chapter".into(),
            page_index,
            page_count: 10,
            last_read_at: Some(last_read_at),
        }
    }
}
