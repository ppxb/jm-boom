use super::*;

pub async fn get_home_feed(endpoint: Option<String>) -> ApiResult<HomeFeedResult> {
    let endpoint = resolve_api_endpoint(endpoint)?;
    let client = build_http_client()?;
    let setting_auth = SettingAuth::current();
    let api_auth = ApiAuth::current();
    let img_host_future = request_remote_img_host(&client, &endpoint, &setting_auth);
    let payload_future = request_home_feed_payload(&client, &endpoint, &api_auth);
    let (img_host_result, payload_result) = tokio::join!(img_host_future, payload_future);
    let img_host = match img_host_result {
        Ok(img_host) => Some(img_host),
        Err(error) => {
            tracing::warn!(error = %error, "failed to load remote setting for home covers");
            None
        }
    };
    let sections = map_home_feed_sections(
        payload_result?,
        img_host.as_deref(),
        HOME_SECTION_PREVIEW_LIMIT,
    );

    Ok(HomeFeedResult { endpoint, sections })
}

#[allow(clippy::too_many_arguments)]
pub async fn get_home_section_list(
    mode: String,
    page: Option<u32>,
    section_id: Option<String>,
    section_title: Option<String>,
    _slug: Option<String>,
    _section_type: Option<String>,
    filter_value: Option<String>,
    category: Option<String>,
    week: Option<String>,
    order: Option<String>,
    endpoint: Option<String>,
) -> ApiResult<HomeSectionListResult> {
    let mode = parse_home_section_list_mode(&mode)?;
    let page = page.unwrap_or(1).max(1);
    let endpoint = resolve_api_endpoint(endpoint)?;
    let section_id = section_id.unwrap_or_default().trim().to_string();
    let section_title = section_title.unwrap_or_default().trim().to_string();
    let filter_value = filter_value.unwrap_or_default().trim().to_string();
    let category = category.unwrap_or_default().trim().to_string();
    let week = week.unwrap_or_default().trim().to_string();
    let order = order.unwrap_or_default().trim().to_string();
    let title = if section_title.is_empty() {
        default_home_section_list_title(mode)
    } else {
        section_title.clone()
    };

    let client = build_http_client()?;
    let setting_auth = SettingAuth::current();
    let api_auth = ApiAuth::current();
    let img_host_future = request_remote_img_host(&client, &endpoint, &setting_auth);
    let payload_future = request_home_section_list(
        &client,
        &endpoint,
        mode,
        page,
        &section_id,
        &filter_value,
        &category,
        &week,
        &order,
        &api_auth,
    );
    let (img_host_result, payload_result) = tokio::join!(img_host_future, payload_future);
    let img_host = match img_host_result {
        Ok(img_host) => Some(img_host),
        Err(error) => {
            tracing::warn!(
                error = %error,
                "failed to load remote setting for home section list covers"
            );
            None
        }
    };
    let payload = payload_result?;

    Ok(HomeSectionListResult {
        endpoint,
        mode,
        page,
        page_size: HOME_SECTION_LIST_PAGE_SIZE as u32,
        total: payload.total,
        has_more: payload.has_more,
        title,
        items: payload
            .items
            .into_iter()
            .map(|item| map_feed_comic(item, img_host.as_deref()))
            .collect(),
    })
}

pub async fn get_week_filters(endpoint: Option<String>) -> ApiResult<WeekFiltersResult> {
    let endpoint = resolve_api_endpoint(endpoint)?;
    let client = build_http_client()?;
    let auth = ApiAuth::current();
    let week = request_week_data(&client, &endpoint, &auth).await?;
    let categories = map_week_categories(week.categories);
    let types = map_week_types(week.types);

    Ok(WeekFiltersResult {
        endpoint,
        default_category_id: categories.first().map(|item| item.id.clone()),
        default_type_id: types.first().map(|item| item.id.clone()),
        categories,
        types,
    })
}

pub async fn get_week_items(
    page: Option<u32>,
    category_id: String,
    type_id: String,
    endpoint: Option<String>,
) -> ApiResult<WeekItemsResult> {
    let page = page.unwrap_or(1);
    let endpoint = resolve_api_endpoint(endpoint)?;
    let category_id = category_id.trim();
    let type_id = type_id.trim();

    if category_id.is_empty() || type_id.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Week items need both category_id and type_id",
        ));
    }

    let client = build_http_client()?;
    let setting_auth = SettingAuth::current();
    let api_auth = ApiAuth::current();
    let img_host_future = request_remote_img_host(&client, &endpoint, &setting_auth);
    let payload_future =
        request_week_comics(&client, &endpoint, page, category_id, type_id, &api_auth);
    let (img_host_result, payload_result) = tokio::join!(img_host_future, payload_future);
    let img_host = match img_host_result {
        Ok(img_host) => Some(img_host),
        Err(error) => {
            tracing::warn!(error = %error, "failed to load remote setting for weekly covers");
            None
        }
    };
    let payload = payload_result?;

    Ok(WeekItemsResult {
        endpoint,
        page,
        total: payload.total,
        items: payload
            .list
            .into_iter()
            .map(|item| map_feed_comic(item, img_host.as_deref()))
            .collect(),
    })
}

pub(crate) async fn request_home_feed_payload(
    client: &reqwest::Client,
    endpoint: &str,
    auth: &ApiAuth,
) -> ApiResult<Vec<HomeFeedSectionPayload>> {
    request_api_data(client, endpoint, "promote", &[], auth).await
}

pub(crate) fn map_home_feed_sections(
    payload: Vec<HomeFeedSectionPayload>,
    img_host: Option<&str>,
    preview_limit: usize,
) -> Vec<HomeFeedSection> {
    payload
        .into_iter()
        .filter(|section| !is_unsupported_home_section(&section.title))
        .map(|section| {
            let list_mode = resolve_home_section_list_mode(&section);
            let rank_tag = if matches!(list_mode, Some(HomeSectionListMode::Ranking)) {
                resolve_home_section_ranking_tag(&section)
            } else {
                String::new()
            };

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
                    .take(preview_limit)
                    .map(|item| map_feed_comic(item, img_host))
                    .collect(),
            }
        })
        .collect()
}

pub(crate) struct HomeSectionListPayload {
    pub(crate) total: u32,
    pub(crate) has_more: bool,
    pub(crate) items: Vec<ComicListItemPayload>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn request_home_section_list(
    client: &reqwest::Client,
    endpoint: &str,
    mode: HomeSectionListMode,
    page: u32,
    section_id: &str,
    filter_value: &str,
    category: &str,
    week: &str,
    order: &str,
    auth: &ApiAuth,
) -> ApiResult<HomeSectionListPayload> {
    match mode {
        HomeSectionListMode::Promote => {
            request_promote_list(client, endpoint, page, section_id, filter_value, auth).await
        }
        HomeSectionListMode::Weekly => {
            request_weekly_update_list(client, endpoint, page, week, category, auth).await
        }
        HomeSectionListMode::Latest => request_latest_list(client, endpoint, page, auth).await,
        HomeSectionListMode::Ranking => {
            request_category_filter_list(client, endpoint, page, category, order, auth).await
        }
    }
}

pub(crate) async fn request_promote_list(
    client: &reqwest::Client,
    endpoint: &str,
    page: u32,
    section_id: &str,
    filter_value: &str,
    auth: &ApiAuth,
) -> ApiResult<HomeSectionListPayload> {
    const SOURCE_PAGE_SIZE: usize = 27;

    let id = parse_u32_or_default(section_id)
        .or_else(|| parse_u32_or_default(filter_value))
        .unwrap_or_default();
    let start = local_list_start(page);
    let mut source_page = (start / SOURCE_PAGE_SIZE) as u32;
    let offset = start % SOURCE_PAGE_SIZE;
    let mut total = 0;
    let mut source_has_more = true;
    let mut buffer = Vec::new();

    while buffer.len() < offset + HOME_SECTION_LIST_PAGE_SIZE && source_has_more {
        let payload = request_promote_source_page(client, endpoint, id, source_page, auth).await?;
        total = payload.total;
        let count = payload.list.len();
        let loaded_count = source_page as usize * SOURCE_PAGE_SIZE + count;
        source_has_more = count >= SOURCE_PAGE_SIZE
            && (payload.total == 0 || loaded_count < payload.total as usize);
        buffer.extend(payload.list);
        source_page = source_page.saturating_add(1);
    }

    let available = buffer.len().saturating_sub(offset);
    let has_more = if total > 0 {
        (page as usize * HOME_SECTION_LIST_PAGE_SIZE) < total as usize
    } else {
        available > HOME_SECTION_LIST_PAGE_SIZE || source_has_more
    };
    let items = buffer
        .into_iter()
        .skip(offset)
        .take(HOME_SECTION_LIST_PAGE_SIZE)
        .collect();

    Ok(HomeSectionListPayload {
        total,
        has_more,
        items,
    })
}

pub(crate) async fn request_promote_source_page(
    client: &reqwest::Client,
    endpoint: &str,
    id: u32,
    page: u32,
    auth: &ApiAuth,
) -> ApiResult<PromoteListPayload> {
    request_api_data(
        client,
        endpoint,
        "promote_list",
        &[("id", id.to_string()), ("page", page.to_string())],
        auth,
    )
    .await
}

pub(crate) async fn request_weekly_update_list(
    client: &reqwest::Client,
    endpoint: &str,
    page: u32,
    week: &str,
    category: &str,
    auth: &ApiAuth,
) -> ApiResult<HomeSectionListPayload> {
    const SOURCE_PAGE_SIZE: usize = 40;

    let start = local_list_start(page);
    let request_page = (start / SOURCE_PAGE_SIZE) as u32 + 1;
    let offset = start % SOURCE_PAGE_SIZE;
    let date = parse_u32_or_default(week).unwrap_or_else(current_china_weekday);
    let category = if category.is_empty() { "all" } else { category };
    let value: serde_json::Value = request_api_data(
        client,
        endpoint,
        "serialization",
        &[
            ("date", date.to_string()),
            ("type", category.to_string()),
            ("page", request_page.to_string()),
        ],
        auth,
    )
    .await?;

    if value
        .get("error")
        .and_then(|error| error.as_str())
        .map(|error| error == "没有资料")
        .unwrap_or(false)
    {
        return Ok(HomeSectionListPayload {
            total: 0,
            has_more: false,
            items: Vec::new(),
        });
    }

    let payload: WeeklyUpdatePayload = serde_json::from_value(value).map_err(|error| {
        ApiError::new(
            ApiErrorKind::Payload,
            format!("{endpoint}/serialization: Invalid payload: {error}"),
        )
    })?;
    let source_count = payload.list.len();
    let has_more =
        source_count > offset + HOME_SECTION_LIST_PAGE_SIZE || source_count >= SOURCE_PAGE_SIZE;
    let items = payload
        .list
        .into_iter()
        .skip(offset)
        .take(HOME_SECTION_LIST_PAGE_SIZE)
        .collect();

    Ok(HomeSectionListPayload {
        total: 0,
        has_more,
        items,
    })
}

pub(crate) async fn request_latest_list(
    client: &reqwest::Client,
    endpoint: &str,
    page: u32,
    auth: &ApiAuth,
) -> ApiResult<HomeSectionListPayload> {
    const SOURCE_PAGE_SIZE: usize = 80;

    let start = local_list_start(page);
    let request_page = (start / SOURCE_PAGE_SIZE) as u32;
    let offset = start % SOURCE_PAGE_SIZE;
    let items: Vec<ComicListItemPayload> = request_api_data(
        client,
        endpoint,
        "latest",
        &[("page", request_page.to_string())],
        auth,
    )
    .await?;
    let source_count = items.len();
    let has_more =
        source_count > offset + HOME_SECTION_LIST_PAGE_SIZE || source_count >= SOURCE_PAGE_SIZE;
    let items = items
        .into_iter()
        .skip(offset)
        .take(HOME_SECTION_LIST_PAGE_SIZE)
        .collect();

    Ok(HomeSectionListPayload {
        total: 0,
        has_more,
        items,
    })
}

pub(crate) async fn request_category_filter_list(
    client: &reqwest::Client,
    endpoint: &str,
    page: u32,
    category: &str,
    order: &str,
    auth: &ApiAuth,
) -> ApiResult<HomeSectionListPayload> {
    const SOURCE_PAGE_SIZE: usize = 80;

    let start = local_list_start(page);
    let request_page = (start / SOURCE_PAGE_SIZE) as u32;
    let offset = start % SOURCE_PAGE_SIZE;
    let category = if category.is_empty() {
        "latest"
    } else {
        category
    };
    let order = if order.is_empty() { "new" } else { order };
    let payload: CategoryFilterPayload = request_api_data(
        client,
        endpoint,
        "categories/filter",
        &[
            ("page", request_page.to_string()),
            ("c", category.to_string()),
            ("o", order.to_string()),
        ],
        auth,
    )
    .await?;
    let source_count = payload.content.len();
    let has_more = if payload.total > 0 {
        (page as usize * HOME_SECTION_LIST_PAGE_SIZE) < payload.total as usize
    } else {
        source_count > offset + HOME_SECTION_LIST_PAGE_SIZE || source_count >= SOURCE_PAGE_SIZE
    };
    let items = payload
        .content
        .into_iter()
        .skip(offset)
        .take(HOME_SECTION_LIST_PAGE_SIZE)
        .collect();

    Ok(HomeSectionListPayload {
        total: payload.total,
        has_more,
        items,
    })
}

pub(crate) fn is_unsupported_home_section(title: &str) -> bool {
    let title = title.trim();
    UNSUPPORTED_HOME_SECTION_TITLES.contains(&title)
}

fn resolve_home_section_list_mode(section: &HomeFeedSectionPayload) -> Option<HomeSectionListMode> {
    let title = section.title.trim();
    let lower = format!(
        "{} {} {} {}",
        section.id, section.slug, section.section_type, section.filter_val
    )
    .to_lowercase();

    if title.contains("推荐")
        || title.contains("推薦")
        || section.id == "30"
        || title == "禁漫去码&全彩化"
        || title == "禁漫去碼&全彩化"
    {
        return Some(HomeSectionListMode::Promote);
    }

    if section.id == "26" || title.ends_with("连载更新") || title.ends_with("連載更新") {
        return Some(HomeSectionListMode::Weekly);
    }

    if section.id == "998"
        || section.id == "999"
        || section.id == "1000"
        || title == "禁漫汉化组"
        || title == "禁漫漢化組"
        || title == "韩漫更新"
        || title == "韓漫更新"
        || title == "其他更新"
    {
        return Some(HomeSectionListMode::Ranking);
    }

    if title.contains("最新") || lower.contains("latest") {
        return Some(HomeSectionListMode::Latest);
    }

    None
}

fn resolve_home_section_ranking_tag(section: &HomeFeedSectionPayload) -> String {
    let title = section.title.trim();

    if section.id == "998" || title == "禁漫汉化组" || title == "禁漫漢化組" {
        return "禁漫汉化组".to_string();
    }

    if section.id == "999" || title == "韩漫更新" || title == "韓漫更新" {
        return "hanManTypeMap".to_string();
    }

    if section.id == "1000" || title == "其他更新" {
        return "qiTaLeiTypeMap".to_string();
    }

    String::new()
}

pub(crate) fn parse_home_section_list_mode(value: &str) -> ApiResult<HomeSectionListMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "promote" | "promotelist" | "recommend" => Ok(HomeSectionListMode::Promote),
        "weekly" | "week" => Ok(HomeSectionListMode::Weekly),
        "latest" => Ok(HomeSectionListMode::Latest),
        "ranking" | "category" | "categories" | "timeranking" => Ok(HomeSectionListMode::Ranking),
        value => Err(ApiError::new(
            ApiErrorKind::MissingData,
            format!("Unsupported home section list mode: {value}"),
        )),
    }
}

pub(crate) fn default_home_section_list_title(mode: HomeSectionListMode) -> String {
    match mode {
        HomeSectionListMode::Promote => "推荐".to_string(),
        HomeSectionListMode::Weekly => "每周连载更新".to_string(),
        HomeSectionListMode::Latest => "最新".to_string(),
        HomeSectionListMode::Ranking => "分类更新".to_string(),
    }
}

pub(crate) fn local_list_start(page: u32) -> usize {
    page.saturating_sub(1) as usize * HOME_SECTION_LIST_PAGE_SIZE
}

pub(crate) fn current_china_weekday() -> u32 {
    const SECONDS_PER_DAY: u64 = 86_400;
    const CHINA_OFFSET_SECONDS: u64 = 8 * 60 * 60;

    let seconds = current_timestamp().saturating_add(CHINA_OFFSET_SECONDS);
    // 1970-01-01 is Thursday. Breeze uses Sunday=7, Monday=1.
    match ((seconds / SECONDS_PER_DAY) + 4) % 7 {
        0 => 7,
        value => value as u32,
    }
}

pub(crate) fn parse_u32_or_default(value: &str) -> Option<u32> {
    value.trim().parse::<u32>().ok()
}

pub(crate) async fn request_week_data(
    client: &reqwest::Client,
    endpoint: &str,
    auth: &ApiAuth,
) -> ApiResult<WeekPayload> {
    request_api_data(client, endpoint, "week", &[], auth).await
}

pub(crate) async fn request_week_comics(
    client: &reqwest::Client,
    endpoint: &str,
    page: u32,
    category_id: &str,
    type_id: &str,
    auth: &ApiAuth,
) -> ApiResult<WeekComicsPayload> {
    request_api_data(
        client,
        endpoint,
        "week/filter",
        &[
            ("page", page.to_string()),
            ("id", category_id.to_string()),
            ("type", type_id.to_string()),
        ],
        auth,
    )
    .await
}
