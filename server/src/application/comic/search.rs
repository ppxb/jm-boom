use super::ComicService;
use crate::{domain::comic::ComicSummary, jm::JmResult};

const SEARCH_PAGE_SIZE: u32 = 80;

pub(crate) struct ComicSearchRequest {
    pub(crate) keyword: String,
    pub(crate) page: u32,
    pub(crate) sort_by: u8,
}

pub(crate) struct ComicSearch {
    pub(crate) page: u32,
    pub(crate) pages: u32,
    pub(crate) total: u32,
    pub(crate) has_reached_max: bool,
    pub(crate) items: Vec<ComicSummary>,
}

impl ComicService {
    pub async fn search_comics(&self, request: ComicSearchRequest) -> JmResult<ComicSearch> {
        let keyword = request.keyword.trim().to_string();
        let page = request.page.max(1);
        if keyword.is_empty() {
            return Ok(search_result(page, 0, Vec::new()));
        }

        if page == 1 && is_comic_id(&keyword) {
            if let Ok(detail) = self.get_comic_detail(keyword.clone()).await {
                return Ok(search_result(
                    page,
                    1,
                    vec![ComicSummary::from_detail(detail)],
                ));
            }
        }

        let order = match request.sort_by {
            2 => "mv",
            3 => "mp",
            4 => "tf",
            _ => "mr",
        }
        .to_string();
        let result = self
            .with_failover(move |client, endpoint| {
                let keyword = keyword.clone();
                let order = order.clone();
                Box::pin(async move { client.search(endpoint, &keyword, page, &order).await })
            })
            .await?;

        if result.content.is_empty() {
            if let Some(redirect_id) = result.redirect_aid.filter(|id| is_comic_id(id)) {
                let detail = self.get_comic_detail(redirect_id).await?;
                return Ok(search_result(
                    page,
                    1,
                    vec![ComicSummary::from_detail(detail)],
                ));
            }
        }

        let total = result.total;
        let items = result
            .content
            .into_iter()
            .map(|comic| {
                ComicSummary::new(
                    comic.id,
                    comic.name,
                    comic.author,
                    comic.description,
                    comic.image,
                    comic.tags,
                )
            })
            .collect();
        Ok(search_result(page, total, items))
    }
}

fn search_result(page: u32, total: u32, items: Vec<ComicSummary>) -> ComicSearch {
    let pages = total.div_ceil(SEARCH_PAGE_SIZE);
    ComicSearch {
        page,
        pages,
        total,
        has_reached_max: page >= pages,
        items,
    }
}

fn is_comic_id(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|character| character.is_ascii_digit())
}
