use super::{
    current_china_weekday,
    payload::{CategoryPayload, ComicListPayload, PromotePayload, SectionPayload, WeeklyPayload},
    ComicService, HomeSectionMode, HomeSectionRequest, PAGE_SIZE, PROMOTE_PAGE_SIZE,
};
use crate::jm::JmResult;

impl ComicService {
    pub(super) async fn request_section_list(
        &self,
        request: &HomeSectionRequest,
        page: u32,
    ) -> JmResult<SectionPayload> {
        let start = page.saturating_sub(1) as usize * PAGE_SIZE;
        match request.mode {
            HomeSectionMode::Promote => self.promote_section(request, page).await,
            HomeSectionMode::Weekly => self.weekly_section(request, start).await,
            HomeSectionMode::Latest => self.latest_section(start).await,
            HomeSectionMode::Ranking => self.ranking_section(request, start).await,
        }
    }

    async fn promote_section(
        &self,
        request: &HomeSectionRequest,
        page: u32,
    ) -> JmResult<SectionPayload> {
        let id = request
            .section_id
            .parse::<u32>()
            .ok()
            .or_else(|| request.filter_value.parse::<u32>().ok())
            .unwrap_or_default();
        let source_page = page.saturating_sub(1);
        let payload: PromotePayload = self
            .with_failover(move |client, endpoint| {
                Box::pin(async move {
                    client
                        .get(
                            endpoint,
                            "promote_list",
                            &[("id", id.to_string()), ("page", source_page.to_string())],
                        )
                        .await
                })
            })
            .await?;
        let count = payload.list.len();
        let loaded_count = source_page as usize * PROMOTE_PAGE_SIZE + count;

        Ok(SectionPayload {
            page_size: PROMOTE_PAGE_SIZE as u32,
            total: payload.total,
            has_more: count >= PROMOTE_PAGE_SIZE
                && (payload.total == 0 || loaded_count < payload.total as usize),
            items: payload.list,
        })
    }

    async fn weekly_section(
        &self,
        request: &HomeSectionRequest,
        start: usize,
    ) -> JmResult<SectionPayload> {
        let request_page = (start / 40) as u32 + 1;
        let offset = start % 40;
        let date = request
            .week
            .parse::<u32>()
            .unwrap_or_else(|_| current_china_weekday());
        let category = if request.category.is_empty() {
            "all".to_string()
        } else {
            request.category.clone()
        };
        let payload: WeeklyPayload = self
            .with_failover(move |client, endpoint| {
                let category = category.clone();
                Box::pin(async move {
                    client
                        .get(
                            endpoint,
                            "serialization",
                            &[
                                ("date", date.to_string()),
                                ("type", category),
                                ("page", request_page.to_string()),
                            ],
                        )
                        .await
                })
            })
            .await?;
        let count = payload.list.len();

        Ok(SectionPayload {
            page_size: PAGE_SIZE as u32,
            total: 0,
            has_more: count > offset + PAGE_SIZE || count >= 40,
            items: payload
                .list
                .into_iter()
                .skip(offset)
                .take(PAGE_SIZE)
                .collect(),
        })
    }

    async fn latest_section(&self, start: usize) -> JmResult<SectionPayload> {
        let request_page = (start / 80) as u32;
        let offset = start % 80;
        let items: Vec<ComicListPayload> = self
            .with_failover(move |client, endpoint| {
                Box::pin(async move {
                    client
                        .get(endpoint, "latest", &[("page", request_page.to_string())])
                        .await
                })
            })
            .await?;
        let count = items.len();

        Ok(SectionPayload {
            page_size: PAGE_SIZE as u32,
            total: 0,
            has_more: count > offset + PAGE_SIZE || count >= 80,
            items: items.into_iter().skip(offset).take(PAGE_SIZE).collect(),
        })
    }

    async fn ranking_section(
        &self,
        request: &HomeSectionRequest,
        start: usize,
    ) -> JmResult<SectionPayload> {
        let request_page = (start / 80) as u32;
        let offset = start % 80;
        let category = if request.category.is_empty() {
            "latest".to_string()
        } else {
            request.category.clone()
        };
        let order = if request.order.is_empty() {
            "new".to_string()
        } else {
            request.order.clone()
        };
        let payload: CategoryPayload = self
            .with_failover(move |client, endpoint| {
                let category = category.clone();
                let order = order.clone();
                Box::pin(async move {
                    client
                        .get(
                            endpoint,
                            "categories/filter",
                            &[
                                ("page", request_page.to_string()),
                                ("c", category),
                                ("o", order),
                            ],
                        )
                        .await
                })
            })
            .await?;
        let count = payload.content.len();

        Ok(SectionPayload {
            page_size: PAGE_SIZE as u32,
            total: payload.total,
            has_more: count > offset + PAGE_SIZE
                || (payload.total > 0 && start + PAGE_SIZE < payload.total as usize),
            items: payload
                .content
                .into_iter()
                .skip(offset)
                .take(PAGE_SIZE)
                .collect(),
        })
    }
}
