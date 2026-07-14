mod payload;
mod section;

use super::ComicService;
use crate::{domain::comic::ComicSummary, jm::JmResult};
use payload::{ComicListPayload, WeekComicsPayload, WeekPayload};

pub(super) const PAGE_SIZE: usize = 20;
pub(super) const PROMOTE_PAGE_SIZE: usize = 27;

pub(crate) struct HomeFeed {
    pub(crate) sections: Vec<HomeFeedSection>,
}

pub(crate) struct HomeFeedSection {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) filter_value: String,
    pub(crate) list_mode: Option<HomeSectionMode>,
    pub(crate) rank_tag: String,
    pub(crate) items: Vec<ComicSummary>,
}

#[derive(Clone, Copy)]
pub(crate) enum HomeSectionMode {
    Promote,
    Weekly,
    Latest,
    Ranking,
}

pub(crate) struct HomeSectionRequest {
    pub(crate) mode: HomeSectionMode,
    pub(crate) page: u32,
    pub(crate) section_id: String,
    pub(crate) section_title: String,
    pub(crate) filter_value: String,
    pub(crate) category: String,
    pub(crate) week: String,
    pub(crate) order: String,
}

pub(crate) struct HomeSectionList {
    pub(crate) mode: HomeSectionMode,
    pub(crate) page: u32,
    pub(crate) page_size: u32,
    pub(crate) total: u32,
    pub(crate) has_more: bool,
    pub(crate) title: String,
    pub(crate) items: Vec<ComicSummary>,
}

pub(crate) struct WeekFilters {
    pub(crate) categories: Vec<WeekCategory>,
    pub(crate) types: Vec<WeekType>,
    pub(crate) default_category_id: Option<String>,
    pub(crate) default_type_id: Option<String>,
}

pub(crate) struct WeekCategory {
    pub(crate) id: String,
    pub(crate) time: String,
    pub(crate) title: String,
    pub(crate) label: String,
}

pub(crate) struct WeekType {
    pub(crate) id: String,
    pub(crate) title: String,
}

pub(crate) struct WeekItems {
    pub(crate) page: u32,
    pub(crate) total: u32,
    pub(crate) items: Vec<ComicSummary>,
}

impl ComicService {
    pub async fn get_home_feed(&self) -> JmResult<HomeFeed> {
        let sections = self
            .with_failover(|client, endpoint| Box::pin(client.get_home_feed(endpoint)))
            .await?
            .into_iter()
            .map(|section| HomeFeedSection {
                list_mode: resolve_list_mode(
                    &section.id,
                    &section.title,
                    &section.slug,
                    &section.section_type,
                    &section.filter_val,
                ),
                rank_tag: resolve_rank_tag(&section.id, &section.title),
                id: section.id,
                title: section.title,
                filter_value: section.filter_val,
                items: section
                    .content
                    .into_iter()
                    .take(8)
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
                    .collect(),
            })
            .collect();

        Ok(HomeFeed { sections })
    }

    pub async fn get_home_section_list(
        &self,
        request: HomeSectionRequest,
    ) -> JmResult<HomeSectionList> {
        let page = request.page.max(1);
        let payload = self.request_section_list(&request, page).await?;
        let title = if request.section_title.trim().is_empty() {
            default_title(request.mode).to_string()
        } else {
            request.section_title.trim().to_string()
        };

        Ok(HomeSectionList {
            mode: request.mode,
            page,
            page_size: payload.page_size,
            total: payload.total,
            has_more: payload.has_more,
            title,
            items: payload.items.into_iter().map(map_feed_comic).collect(),
        })
    }

    pub async fn get_week_filters(&self) -> JmResult<WeekFilters> {
        let payload: WeekPayload = self
            .with_failover(|client, endpoint| Box::pin(client.get(endpoint, "week", &[])))
            .await?;
        let categories = payload
            .categories
            .into_iter()
            .map(|item| WeekCategory {
                label: if item.time.is_empty() {
                    item.title.clone()
                } else {
                    format!("{} {}", item.time, item.title)
                },
                id: item.id,
                time: item.time,
                title: item.title,
            })
            .collect::<Vec<_>>();
        let types = payload
            .types
            .into_iter()
            .map(|item| WeekType {
                id: item.id,
                title: item.title,
            })
            .collect::<Vec<_>>();

        Ok(WeekFilters {
            default_category_id: categories.first().map(|item| item.id.clone()),
            default_type_id: types.first().map(|item| item.id.clone()),
            categories,
            types,
        })
    }

    pub async fn get_week_items(
        &self,
        page: u32,
        category_id: String,
        type_id: String,
    ) -> JmResult<WeekItems> {
        let page = page.max(1);
        let payload: WeekComicsPayload = self
            .with_failover(move |client, endpoint| {
                let category_id = category_id.clone();
                let type_id = type_id.clone();
                Box::pin(async move {
                    client
                        .get(
                            endpoint,
                            "week/filter",
                            &[
                                ("page", page.to_string()),
                                ("id", category_id),
                                ("type", type_id),
                            ],
                        )
                        .await
                })
            })
            .await?;

        Ok(WeekItems {
            page,
            total: payload.total,
            items: payload.list.into_iter().map(map_feed_comic).collect(),
        })
    }
}

fn map_feed_comic(item: ComicListPayload) -> ComicSummary {
    let mut tags = Vec::new();
    for title in [item.category, item.category_sub]
        .into_iter()
        .flatten()
        .map(|item| item.title)
        .filter(|title| !title.is_empty())
    {
        if !tags.contains(&title) {
            tags.push(title);
        }
    }

    ComicSummary::new(
        item.id,
        item.name,
        item.author,
        item.description,
        item.image,
        tags,
    )
}

fn resolve_list_mode(
    id: &str,
    title: &str,
    slug: &str,
    section_type: &str,
    filter: &str,
) -> Option<HomeSectionMode> {
    let lower = format!("{id} {slug} {section_type} {filter}").to_lowercase();
    if title.contains("推荐") || title.contains("推薦") || id == "30" {
        Some(HomeSectionMode::Promote)
    } else if id == "26" || title.ends_with("连载更新") || title.ends_with("連載更新") {
        Some(HomeSectionMode::Weekly)
    } else if matches!(id, "998" | "999" | "1000") {
        Some(HomeSectionMode::Ranking)
    } else if title.contains("最新") || lower.contains("latest") {
        Some(HomeSectionMode::Latest)
    } else {
        None
    }
}

fn resolve_rank_tag(id: &str, title: &str) -> String {
    if id == "998" || title.contains("汉化组") || title.contains("漢化組") {
        "禁漫汉化组".to_string()
    } else if id == "999" || title.contains("韩漫") || title.contains("韓漫") {
        "hanManTypeMap".to_string()
    } else if id == "1000" || title.contains("其他") {
        "qiTaLeiTypeMap".to_string()
    } else {
        String::new()
    }
}

pub(super) fn default_title(mode: HomeSectionMode) -> &'static str {
    match mode {
        HomeSectionMode::Promote => "推荐",
        HomeSectionMode::Weekly => "每周连载更新",
        HomeSectionMode::Latest => "最新",
        HomeSectionMode::Ranking => "分类更新",
    }
}

pub(super) fn current_china_weekday() -> u32 {
    let seconds = chrono::Utc::now().timestamp().saturating_add(8 * 60 * 60) as u64;
    match ((seconds / 86_400) + 4) % 7 {
        0 => 7,
        value => value as u32,
    }
}
