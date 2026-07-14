use crate::{
    api::comic_summary::ComicSummaryResponse,
    application::{
        HomeFeed, HomeSectionList, HomeSectionMode as ApplicationHomeSectionMode,
        HomeSectionRequest, WeekFilters, WeekItems,
    },
    jm::JmResult,
    AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeFeedResult {
    sections: Vec<HomeFeedSection>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeFeedSection {
    id: String,
    title: String,
    filter_value: String,
    list_mode: Option<HomeSectionMode>,
    rank_tag: String,
    items: Vec<ComicSummaryResponse>,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HomeSectionMode {
    Promote,
    Weekly,
    Latest,
    Ranking,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeSectionListResult {
    mode: HomeSectionMode,
    page: u32,
    page_size: u32,
    total: u32,
    has_more: bool,
    title: String,
    items: Vec<ComicSummaryResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekFiltersResult {
    categories: Vec<WeekCategory>,
    types: Vec<WeekType>,
    default_category_id: Option<String>,
    default_type_id: Option<String>,
}

#[derive(Serialize)]
pub struct WeekCategory {
    id: String,
    time: String,
    title: String,
    label: String,
}

#[derive(Serialize)]
pub struct WeekType {
    id: String,
    title: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekItemsResult {
    page: u32,
    total: u32,
    items: Vec<ComicSummaryResponse>,
}

#[derive(Deserialize)]
pub struct HomeSectionQuery {
    mode: HomeSectionMode,
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default, rename = "sectionId")]
    section_id: String,
    #[serde(default, rename = "sectionTitle")]
    section_title: String,
    #[serde(default, rename = "filterValue")]
    filter_value: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    week: String,
    #[serde(default)]
    order: String,
}

#[derive(Deserialize)]
pub struct WeekItemsQuery {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(rename = "categoryId")]
    category_id: String,
    #[serde(rename = "typeId")]
    type_id: String,
}

pub async fn get_home_feed(State(app): State<AppState>) -> JmResult<Json<HomeFeedResult>> {
    app.comics
        .get_home_feed()
        .await
        .map(HomeFeedResult::from)
        .map(Json)
}

pub async fn get_home_section_list(
    State(app): State<AppState>,
    Query(query): Query<HomeSectionQuery>,
) -> JmResult<Json<HomeSectionListResult>> {
    app.comics
        .get_home_section_list(HomeSectionRequest {
            mode: query.mode.into(),
            page: query.page,
            section_id: query.section_id,
            section_title: query.section_title,
            filter_value: query.filter_value,
            category: query.category,
            week: query.week,
            order: query.order,
        })
        .await
        .map(HomeSectionListResult::from)
        .map(Json)
}

pub async fn get_week_filters(State(app): State<AppState>) -> JmResult<Json<WeekFiltersResult>> {
    app.comics
        .get_week_filters()
        .await
        .map(WeekFiltersResult::from)
        .map(Json)
}

pub async fn get_week_items(
    State(app): State<AppState>,
    Query(query): Query<WeekItemsQuery>,
) -> JmResult<Json<WeekItemsResult>> {
    app.comics
        .get_week_items(query.page, query.category_id, query.type_id)
        .await
        .map(WeekItemsResult::from)
        .map(Json)
}

impl From<HomeFeed> for HomeFeedResult {
    fn from(feed: HomeFeed) -> Self {
        Self {
            sections: feed
                .sections
                .into_iter()
                .map(|section| HomeFeedSection {
                    id: section.id,
                    title: section.title,
                    filter_value: section.filter_value,
                    list_mode: section.list_mode.map(HomeSectionMode::from),
                    rank_tag: section.rank_tag,
                    items: section.items.into_iter().map(Into::into).collect(),
                })
                .collect(),
        }
    }
}

impl From<HomeSectionList> for HomeSectionListResult {
    fn from(list: HomeSectionList) -> Self {
        Self {
            mode: list.mode.into(),
            page: list.page,
            page_size: list.page_size,
            total: list.total,
            has_more: list.has_more,
            title: list.title,
            items: list.items.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<WeekFilters> for WeekFiltersResult {
    fn from(filters: WeekFilters) -> Self {
        Self {
            categories: filters
                .categories
                .into_iter()
                .map(|category| WeekCategory {
                    id: category.id,
                    time: category.time,
                    title: category.title,
                    label: category.label,
                })
                .collect(),
            types: filters
                .types
                .into_iter()
                .map(|item| WeekType {
                    id: item.id,
                    title: item.title,
                })
                .collect(),
            default_category_id: filters.default_category_id,
            default_type_id: filters.default_type_id,
        }
    }
}

impl From<WeekItems> for WeekItemsResult {
    fn from(items: WeekItems) -> Self {
        Self {
            page: items.page,
            total: items.total,
            items: items.items.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<HomeSectionMode> for ApplicationHomeSectionMode {
    fn from(mode: HomeSectionMode) -> Self {
        match mode {
            HomeSectionMode::Promote => Self::Promote,
            HomeSectionMode::Weekly => Self::Weekly,
            HomeSectionMode::Latest => Self::Latest,
            HomeSectionMode::Ranking => Self::Ranking,
        }
    }
}

impl From<ApplicationHomeSectionMode> for HomeSectionMode {
    fn from(mode: ApplicationHomeSectionMode) -> Self {
        match mode {
            ApplicationHomeSectionMode::Promote => Self::Promote,
            ApplicationHomeSectionMode::Weekly => Self::Weekly,
            ApplicationHomeSectionMode::Latest => Self::Latest,
            ApplicationHomeSectionMode::Ranking => Self::Ranking,
        }
    }
}

fn default_page() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use crate::api::media::cover_url;

    #[test]
    fn proxies_numeric_cover_ids() {
        assert_eq!(cover_url("1444583", "fallback"), "/api/covers/1444583");
    }

    #[test]
    fn preserves_fallback_for_invalid_cover_ids() {
        assert_eq!(cover_url("", "fallback"), "fallback");
        assert_eq!(cover_url("not-numeric", "fallback"), "fallback");
    }
}
