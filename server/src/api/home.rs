use crate::{jm::JmResult, AppState};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Deserializer, Serialize};

const PAGE_SIZE: usize = 20;

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
    slug: String,
    #[serde(rename = "type")]
    section_type: String,
    filter_value: String,
    list_mode: Option<HomeSectionMode>,
    rank_tag: String,
    items: Vec<FeedComic>,
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
    items: Vec<FeedComic>,
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
    items: Vec<FeedComic>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FeedComic {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) author: String,
    pub(crate) description: String,
    pub(crate) image: String,
    pub(crate) tags: Vec<String>,
    pub(crate) updated_at: Option<i64>,
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
    let sections = app
        .jm_request(|client, endpoint| Box::pin(client.get_home_feed(endpoint)))
        .await?;
    let img_host = app.img_host().await;
    let sections = sections
        .into_iter()
        .map(|section| {
            let list_mode = resolve_list_mode(
                &section.id,
                &section.title,
                &section.slug,
                &section.section_type,
                &section.filter_val,
            );
            let rank_tag = resolve_rank_tag(&section.id, &section.title);
            HomeFeedSection {
                id: section.id,
                title: section.title,
                slug: section.slug,
                section_type: section.section_type,
                filter_value: section.filter_val,
                list_mode,
                rank_tag,
                items: section
                    .content
                    .into_iter()
                    .take(8)
                    .map(|comic| FeedComic {
                        image: cover_url(img_host.as_deref(), &comic.id, &comic.image),
                        id: comic.id,
                        title: comic.name,
                        author: comic.author,
                        description: comic.description,
                        tags: comic.tags,
                        updated_at: None,
                    })
                    .collect(),
            }
        })
        .collect();
    Ok(Json(HomeFeedResult { sections }))
}

pub async fn get_home_section_list(
    State(app): State<AppState>,
    Query(query): Query<HomeSectionQuery>,
) -> JmResult<Json<HomeSectionListResult>> {
    let page = query.page.max(1);
    let payload = request_section_list(&app, &query, page).await?;
    let img_host = app.img_host().await;
    let title = if query.section_title.trim().is_empty() {
        default_title(query.mode).to_string()
    } else {
        query.section_title.trim().to_string()
    };

    Ok(Json(HomeSectionListResult {
        mode: query.mode,
        page,
        page_size: PAGE_SIZE as u32,
        total: payload.total,
        has_more: payload.has_more,
        title,
        items: payload
            .items
            .into_iter()
            .map(|item| map_feed_comic(item, img_host.as_deref()))
            .collect(),
    }))
}

pub async fn get_week_filters(State(app): State<AppState>) -> JmResult<Json<WeekFiltersResult>> {
    let payload: WeekPayload = app
        .jm_request(|client, endpoint| Box::pin(client.get(endpoint, "week", &[])))
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

    Ok(Json(WeekFiltersResult {
        default_category_id: categories.first().map(|item| item.id.clone()),
        default_type_id: types.first().map(|item| item.id.clone()),
        categories,
        types,
    }))
}

pub async fn get_week_items(
    State(app): State<AppState>,
    Query(query): Query<WeekItemsQuery>,
) -> JmResult<Json<WeekItemsResult>> {
    let page = query.page.max(1);
    let category_id = query.category_id.clone();
    let type_id = query.type_id.clone();
    let payload: WeekComicsPayload = app
        .jm_request(move |client, endpoint| {
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
    let img_host = app.img_host().await;

    Ok(Json(WeekItemsResult {
        page,
        total: payload.total,
        items: payload
            .list
            .into_iter()
            .map(|item| map_feed_comic(item, img_host.as_deref()))
            .collect(),
    }))
}

async fn request_section_list(
    app: &AppState,
    query: &HomeSectionQuery,
    page: u32,
) -> JmResult<SectionPayload> {
    let start = page.saturating_sub(1) as usize * PAGE_SIZE;
    match query.mode {
        HomeSectionMode::Promote => {
            let id = query
                .section_id
                .parse::<u32>()
                .ok()
                .or_else(|| query.filter_value.parse::<u32>().ok())
                .unwrap_or_default();
            let source_page = (start / 27) as u32;
            let offset = start % 27;
            let payload: PromotePayload = app
                .jm_request(move |client, endpoint| {
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
            Ok(SectionPayload {
                total: payload.total,
                has_more: count > offset + PAGE_SIZE
                    || (payload.total > 0 && start + PAGE_SIZE < payload.total as usize),
                items: payload
                    .list
                    .into_iter()
                    .skip(offset)
                    .take(PAGE_SIZE)
                    .collect(),
            })
        }
        HomeSectionMode::Weekly => {
            let request_page = (start / 40) as u32 + 1;
            let offset = start % 40;
            let date = query
                .week
                .parse::<u32>()
                .unwrap_or_else(|_| current_china_weekday());
            let category = if query.category.is_empty() {
                "all".to_string()
            } else {
                query.category.clone()
            };
            let payload: WeeklyPayload = app
                .jm_request(move |client, endpoint| {
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
        HomeSectionMode::Latest => {
            let request_page = (start / 80) as u32;
            let offset = start % 80;
            let items: Vec<ComicListPayload> = app
                .jm_request(move |client, endpoint| {
                    Box::pin(async move {
                        client
                            .get(endpoint, "latest", &[("page", request_page.to_string())])
                            .await
                    })
                })
                .await?;
            let count = items.len();
            Ok(SectionPayload {
                total: 0,
                has_more: count > offset + PAGE_SIZE || count >= 80,
                items: items.into_iter().skip(offset).take(PAGE_SIZE).collect(),
            })
        }
        HomeSectionMode::Ranking => {
            let request_page = (start / 80) as u32;
            let offset = start % 80;
            let category = if query.category.is_empty() {
                "latest".to_string()
            } else {
                query.category.clone()
            };
            let order = if query.order.is_empty() {
                "new".to_string()
            } else {
                query.order.clone()
            };
            let payload: CategoryPayload = app
                .jm_request(move |client, endpoint| {
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
}

struct SectionPayload {
    total: u32,
    has_more: bool,
    items: Vec<ComicListPayload>,
}

#[derive(Deserialize)]
struct PromotePayload {
    #[serde(default, deserialize_with = "u32_from_value")]
    total: u32,
    #[serde(default)]
    list: Vec<ComicListPayload>,
}

#[derive(Deserialize)]
struct WeeklyPayload {
    #[serde(default)]
    list: Vec<ComicListPayload>,
}

#[derive(Deserialize)]
struct CategoryPayload {
    #[serde(default, deserialize_with = "u32_from_value")]
    total: u32,
    #[serde(default)]
    content: Vec<ComicListPayload>,
}

#[derive(Deserialize)]
struct WeekPayload {
    #[serde(default)]
    categories: Vec<WeekCategoryPayload>,
    #[serde(default, rename = "type")]
    types: Vec<WeekTypePayload>,
}

#[derive(Deserialize)]
struct WeekCategoryPayload {
    #[serde(default, deserialize_with = "string_from_value")]
    id: String,
    #[serde(default, deserialize_with = "string_from_value")]
    time: String,
    #[serde(default, deserialize_with = "string_from_value")]
    title: String,
}

#[derive(Deserialize)]
struct WeekTypePayload {
    #[serde(default, deserialize_with = "string_from_value")]
    id: String,
    #[serde(default, deserialize_with = "string_from_value")]
    title: String,
}

#[derive(Deserialize)]
struct WeekComicsPayload {
    #[serde(default, deserialize_with = "u32_from_value")]
    total: u32,
    #[serde(default)]
    list: Vec<ComicListPayload>,
}

#[derive(Deserialize)]
struct ComicListPayload {
    #[serde(default, deserialize_with = "string_from_value")]
    id: String,
    #[serde(default, deserialize_with = "string_from_value")]
    author: String,
    #[serde(default, deserialize_with = "string_from_value")]
    description: String,
    #[serde(default, deserialize_with = "string_from_value")]
    name: String,
    #[serde(default, deserialize_with = "string_from_value")]
    image: String,
    #[serde(default)]
    category: Option<CategoryTag>,
    #[serde(default, rename = "category_sub")]
    category_sub: Option<CategoryTag>,
    #[serde(
        default,
        deserialize_with = "optional_i64_from_value",
        rename = "update_at"
    )]
    updated_at: Option<i64>,
}

#[derive(Deserialize)]
struct CategoryTag {
    #[serde(default, deserialize_with = "string_from_value")]
    title: String,
}

fn map_feed_comic(item: ComicListPayload, img_host: Option<&str>) -> FeedComic {
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
    FeedComic {
        image: cover_url(img_host, &item.id, &item.image),
        id: item.id,
        title: item.name,
        author: item.author,
        description: item.description,
        tags,
        updated_at: item.updated_at,
    }
}

pub(crate) fn cover_url(img_host: Option<&str>, comic_id: &str, fallback: &str) -> String {
    img_host
        .filter(|host| !host.is_empty() && !comic_id.is_empty())
        .map(|host| {
            format!(
                "{}/media/albums/{}_3x4.jpg",
                host.trim_end_matches('/'),
                comic_id
            )
        })
        .unwrap_or_else(|| fallback.to_string())
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

fn default_title(mode: HomeSectionMode) -> &'static str {
    match mode {
        HomeSectionMode::Promote => "推荐",
        HomeSectionMode::Weekly => "每周连载更新",
        HomeSectionMode::Latest => "最新",
        HomeSectionMode::Ranking => "分类更新",
    }
}

fn default_page() -> u32 {
    1
}

fn current_china_weekday() -> u32 {
    let seconds = chrono::Utc::now().timestamp().saturating_add(8 * 60 * 60) as u64;
    match ((seconds / 86_400) + 4) % 7 {
        0 => 7,
        value => value as u32,
    }
}

fn string_from_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(value) => value,
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        value => value.to_string(),
    })
}

fn u32_from_value<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(value
        .as_u64()
        .map(|value| value as u32)
        .or_else(|| value.as_str()?.parse().ok())
        .unwrap_or_default())
}

fn optional_i64_from_value<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(value.as_i64().or_else(|| value.as_str()?.parse().ok()))
}
