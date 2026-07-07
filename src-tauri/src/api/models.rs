use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    pub(crate) code: i32,
    pub(crate) data: Option<T>,
    #[serde(rename = "errorMsg")]
    pub(crate) error_msg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SearchPayload {
    #[serde(deserialize_with = "u32_from_string_or_number")]
    pub(crate) total: u32,
    #[serde(default)]
    pub(crate) redirect_aid: Option<String>,
    #[serde(default)]
    pub(crate) content: Vec<SearchComicPayload>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SearchComicPayload {
    #[serde(default, deserialize_with = "string_or_number_or_empty")]
    pub(crate) id: String,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) author: String,
    #[serde(default, deserialize_with = "optional_lossy_string_from_scalar")]
    pub(crate) description: Option<String>,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) name: String,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) image: String,
    #[serde(default)]
    pub(crate) category: Option<SearchCategory>,
    #[serde(default)]
    pub(crate) category_sub: Option<SearchCategory>,
    #[serde(default, deserialize_with = "optional_i64_from_string_or_number")]
    pub(crate) update_at: Option<i64>,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) likes: u32,
    #[serde(
        default,
        alias = "totalViews",
        deserialize_with = "u32_from_string_or_number_or_empty"
    )]
    pub(crate) total_views: u32,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub(crate) tags: Vec<String>,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub(crate) works: Vec<String>,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub(crate) actors: Vec<String>,
    #[serde(default, deserialize_with = "bool_or_int_string_or_empty")]
    pub(crate) liked: bool,
    #[serde(default, deserialize_with = "bool_or_int_string_or_empty")]
    pub(crate) is_favorite: bool,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SearchCategory {
    #[serde(default, deserialize_with = "string_or_number_or_empty")]
    pub(crate) id: String,
    pub(crate) title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ComicListItemPayload {
    #[serde(default, deserialize_with = "string_or_number_or_empty")]
    pub(crate) id: String,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) author: String,
    #[serde(default, deserialize_with = "optional_lossy_string_from_scalar")]
    pub(crate) description: Option<String>,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) name: String,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) image: String,
    #[serde(default)]
    pub(crate) category: Option<SearchCategory>,
    #[serde(default)]
    pub(crate) category_sub: Option<SearchCategory>,
    #[serde(default, deserialize_with = "optional_i64_from_string_or_number")]
    pub(crate) update_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct HomeFeedSectionPayload {
    #[serde(deserialize_with = "string_or_number")]
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) slug: String,
    #[serde(rename = "type")]
    pub(crate) section_type: String,
    #[serde(deserialize_with = "string_or_number")]
    pub(crate) filter_val: String,
    pub(crate) content: Vec<ComicListItemPayload>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WeekPayload {
    pub(crate) categories: Vec<WeekCategoryPayload>,
    #[serde(rename = "type")]
    pub(crate) types: Vec<WeekTypePayload>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WeekCategoryPayload {
    pub(crate) id: String,
    pub(crate) time: String,
    pub(crate) title: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WeekTypePayload {
    pub(crate) id: String,
    pub(crate) title: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WeekComicsPayload {
    #[serde(deserialize_with = "u32_from_string_or_number")]
    pub(crate) total: u32,
    pub(crate) list: Vec<ComicListItemPayload>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PromoteListPayload {
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) total: u32,
    #[serde(default)]
    pub(crate) list: Vec<ComicListItemPayload>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WeeklyUpdatePayload {
    #[serde(default)]
    pub(crate) list: Vec<ComicListItemPayload>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CategoryFilterPayload {
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) total: u32,
    #[serde(default)]
    pub(crate) content: Vec<ComicListItemPayload>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FavoriteListPayload {
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) total: u32,
    #[serde(default)]
    pub(crate) list: Vec<FavoriteComicPayload>,
    #[serde(default)]
    pub(crate) folder_list: Vec<FavoriteFolderPayload>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FavoriteComicPayload {
    #[serde(
        default,
        alias = "AID",
        alias = "aid",
        deserialize_with = "string_or_number_or_empty"
    )]
    pub(crate) id: String,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) name: String,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) author: String,
    #[serde(default, deserialize_with = "optional_lossy_string_from_scalar")]
    pub(crate) description: Option<String>,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) image: String,
    #[serde(default)]
    pub(crate) category: Option<SearchCategory>,
    #[serde(default)]
    pub(crate) category_sub: Option<SearchCategory>,
    #[serde(default, deserialize_with = "optional_i64_from_string_or_number")]
    pub(crate) update_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FavoriteFolderPayload {
    #[serde(
        default,
        rename = "FID",
        alias = "id",
        alias = "folder_id",
        deserialize_with = "string_or_number_or_empty"
    )]
    pub(crate) id: String,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ComicDetailPayload {
    #[serde(deserialize_with = "string_or_number")]
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub(crate) author: Vec<String>,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) total_views: u32,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) likes: u32,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) comment_total: u32,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub(crate) tags: Vec<String>,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub(crate) actors: Vec<String>,
    #[serde(default, deserialize_with = "lossy_string_vec_from_array_or_scalar")]
    pub(crate) works: Vec<String>,
    #[serde(default, deserialize_with = "bool_or_int_string_or_empty")]
    pub(crate) is_favorite: bool,
    #[serde(default, deserialize_with = "bool_or_int_string_or_empty")]
    pub(crate) liked: bool,
    #[serde(default)]
    pub(crate) related_list: Vec<ComicDetailRelatedPayload>,
    #[serde(default)]
    pub(crate) series: Vec<ComicDetailSeriesPayload>,
    #[serde(default, deserialize_with = "string_or_number_or_empty")]
    pub(crate) series_id: String,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) price: u32,
    #[serde(default, deserialize_with = "bool_or_int_string_or_empty")]
    pub(crate) purchased: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ComicDetailRelatedPayload {
    #[serde(deserialize_with = "string_or_number")]
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) author: String,
    #[serde(default)]
    pub(crate) image: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ComicDetailSeriesPayload {
    #[serde(deserialize_with = "string_or_number")]
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(default, deserialize_with = "string_or_number_or_empty")]
    pub(crate) sort: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CommentListPayload {
    #[serde(default)]
    pub(crate) list: Vec<CommentPayload>,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) total: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CommentPayload {
    #[serde(
        default,
        rename = "AID",
        deserialize_with = "optional_string_or_number"
    )]
    pub(crate) aid: Option<String>,
    #[serde(
        default,
        rename = "CID",
        deserialize_with = "string_or_number_or_empty"
    )]
    pub(crate) cid: String,
    #[serde(
        default,
        rename = "UID",
        deserialize_with = "string_or_number_or_empty"
    )]
    pub(crate) uid: String,
    #[serde(default)]
    pub(crate) username: String,
    #[serde(default)]
    pub(crate) nickname: String,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) likes: u32,
    #[serde(default)]
    pub(crate) update_at: String,
    #[serde(default)]
    pub(crate) addtime: String,
    #[serde(
        default,
        rename = "parent_CID",
        deserialize_with = "string_or_number_or_empty"
    )]
    pub(crate) parent_cid: String,
    #[serde(default)]
    pub(crate) content: String,
    #[serde(default)]
    pub(crate) photo: String,
    #[serde(default, deserialize_with = "bool_or_int_string_or_empty")]
    pub(crate) spoiler: bool,
    #[serde(default)]
    pub(crate) replys: Option<Vec<CommentPayload>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct RemoteSettingPayload {
    #[serde(default, deserialize_with = "string_or_number_or_empty")]
    pub(crate) img_host: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NetworkProxyConfig {
    pub(crate) mode: NetworkProxyMode,
    pub(crate) host: String,
    pub(crate) port: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NetworkProxyMode {
    Off,
    Http,
    Socks5,
}

impl Default for NetworkProxyConfig {
    fn default() -> Self {
        Self {
            mode: NetworkProxyMode::Off,
            host: "127.0.0.1".to_string(),
            port: 7890,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct HostConfigPayload {
    #[serde(default, rename = "Server")]
    pub(crate) server: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginPayload {
    #[serde(default, deserialize_with = "optional_string_or_number")]
    pub(crate) jwttoken: Option<String>,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) uid: u32,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) username: String,
    #[serde(default, deserialize_with = "lossy_string_from_scalar")]
    pub(crate) email: String,
    #[serde(default)]
    pub(crate) photo: String,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) coin: u32,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) album_favorites: u32,
    #[serde(default)]
    pub(crate) level_name: String,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) level: u32,
    #[serde(
        default,
        rename = "nextLevelExp",
        deserialize_with = "u32_from_string_or_number_or_empty"
    )]
    pub(crate) next_level_exp: u32,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) exp: u32,
    #[serde(
        default,
        rename = "expPercent",
        deserialize_with = "f32_from_percent_string_or_number_or_empty"
    )]
    pub(crate) exp_percent: f32,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) album_favorites_max: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SignInDataPayload {
    pub(crate) daily_id: u32,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) three_days_coin: u32,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) three_days_exp: u32,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) seven_days_coin: u32,
    #[serde(default, deserialize_with = "u32_from_string_or_number_or_empty")]
    pub(crate) seven_days_exp: u32,
    #[serde(default)]
    pub(crate) event_name: String,
    #[serde(default)]
    pub(crate) background_pc: String,
    #[serde(default)]
    pub(crate) background_phone: String,
    #[serde(default, rename = "currentProgress")]
    pub(crate) current_progress: String,
    #[serde(default)]
    pub(crate) record: Vec<Vec<SignInRecordPayload>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SignInRecordPayload {
    #[serde(default)]
    pub(crate) date: String,
    #[serde(default, deserialize_with = "bool_or_int_string_or_empty")]
    pub(crate) signed: bool,
    #[serde(default, deserialize_with = "bool_or_int_string_or_empty")]
    pub(crate) bonus: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SignInPayload {
    pub(crate) msg: String,
}

#[derive(Debug, Serialize)]
pub struct RemoteSettingResult {
    pub endpoint: String,
    #[serde(rename = "imgHost")]
    pub img_host: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiEndpointProbe {
    pub endpoint: String,
    pub available: bool,
    #[serde(rename = "latencyMs")]
    pub latency_ms: Option<u64>,
    #[serde(rename = "imgHost")]
    pub img_host: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginResult {
    pub endpoint: String,
    pub user: UserProfile,
}

#[derive(Debug, Serialize)]
pub struct SavedLoginConfig {
    pub endpoint: String,
    pub username: String,
    #[serde(rename = "autoLogin")]
    pub auto_login: bool,
    #[serde(rename = "hasPassword")]
    pub has_password: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserProfile {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub avatar: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: String,
    pub level: u32,
    #[serde(rename = "levelName")]
    pub level_name: String,
    #[serde(rename = "currentLevelExp")]
    pub current_level_exp: u32,
    #[serde(rename = "nextLevelExp")]
    pub next_level_exp: u32,
    #[serde(rename = "expPercent")]
    pub exp_percent: f32,
    #[serde(rename = "currentCollectCount")]
    pub current_collect_count: u32,
    #[serde(rename = "maxCollectCount")]
    pub max_collect_count: u32,
    #[serde(rename = "jCoin")]
    pub j_coin: u32,
}

#[derive(Debug, Serialize)]
pub struct SignInDataResult {
    pub endpoint: String,
    #[serde(rename = "dailyId")]
    pub daily_id: u32,
    #[serde(rename = "threeDaysCoin")]
    pub three_days_coin: u32,
    #[serde(rename = "threeDaysExp")]
    pub three_days_exp: u32,
    #[serde(rename = "sevenDaysCoin")]
    pub seven_days_coin: u32,
    #[serde(rename = "sevenDaysExp")]
    pub seven_days_exp: u32,
    #[serde(rename = "eventName")]
    pub event_name: String,
    #[serde(rename = "currentProgress")]
    pub current_progress: String,
    #[serde(rename = "backgroundPc")]
    pub background_pc: String,
    #[serde(rename = "backgroundPhone")]
    pub background_phone: String,
    pub records: Vec<SignInRecord>,
}

#[derive(Debug, Serialize)]
pub struct SignInRecord {
    pub day: u32,
    pub date: String,
    pub signed: bool,
    pub bonus: bool,
}

#[derive(Debug, Serialize)]
pub struct SignInResult {
    pub endpoint: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResultContract {
    pub source: String,
    pub r#extern: HashMap<String, serde_json::Value>,
    pub scheme: SearchResultScheme,
    pub data: SearchResultData,
    pub paging: SearchPagingInfo,
    pub items: Vec<PluginComicListItem>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchResultScheme {
    pub version: String,
    #[serde(rename = "type")]
    pub scheme_type: String,
    pub source: String,
    pub list: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchResultData {
    pub paging: SearchPagingInfo,
    pub items: Vec<PluginComicListItem>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchPagingInfo {
    pub page: u32,
    pub pages: u32,
    pub total: u32,
    #[serde(rename = "hasReachedMax")]
    pub has_reached_max: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PluginComicListItem {
    pub source: String,
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub finished: bool,
    #[serde(rename = "likesCount")]
    pub likes_count: u32,
    #[serde(rename = "viewsCount")]
    pub views_count: u32,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub cover: PluginImageItem,
    pub metadata: Vec<PluginMetadataListItem>,
    pub raw: HashMap<String, serde_json::Value>,
    pub r#extern: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PluginImageItem {
    pub id: String,
    pub url: String,
    pub name: String,
    pub path: String,
    pub r#extern: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PluginMetadataListItem {
    #[serde(rename = "type")]
    pub metadata_type: String,
    pub name: String,
    pub value: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct HomeFeedResult {
    pub endpoint: String,
    pub sections: Vec<HomeFeedSection>,
}

#[derive(Debug, Serialize)]
pub struct HomeFeedSection {
    pub id: String,
    pub title: String,
    pub slug: String,
    #[serde(rename = "type")]
    pub section_type: String,
    #[serde(rename = "filterValue")]
    pub filter_value: String,
    #[serde(rename = "listMode")]
    pub list_mode: Option<HomeSectionListMode>,
    #[serde(rename = "rankTag")]
    pub rank_tag: String,
    pub items: Vec<FeedComic>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HomeSectionListMode {
    Promote,
    Weekly,
    Latest,
    Ranking,
}

#[derive(Debug, Serialize)]
pub struct HomeSectionListResult {
    pub endpoint: String,
    pub mode: HomeSectionListMode,
    pub page: u32,
    #[serde(rename = "pageSize")]
    pub page_size: u32,
    pub total: u32,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
    pub title: String,
    pub items: Vec<FeedComic>,
}

#[derive(Debug, Serialize)]
pub struct WeekFiltersResult {
    pub endpoint: String,
    pub categories: Vec<WeekCategory>,
    pub types: Vec<WeekType>,
    #[serde(rename = "defaultCategoryId")]
    pub default_category_id: Option<String>,
    #[serde(rename = "defaultTypeId")]
    pub default_type_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WeekItemsResult {
    pub endpoint: String,
    pub page: u32,
    pub total: u32,
    pub items: Vec<FeedComic>,
}

#[derive(Debug, Serialize)]
pub struct ComicDetailResult {
    pub endpoint: String,
    pub comic: ComicDetail,
}

#[derive(Debug, Serialize)]
pub struct FavoriteToggleResult {
    pub endpoint: String,
    pub favorited: bool,
}

#[derive(Debug, Serialize)]
pub struct FavoriteListResult {
    pub endpoint: String,
    pub page: u32,
    pub total: u32,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
    pub folders: Vec<FavoriteFolder>,
    pub items: Vec<FeedComic>,
}

#[derive(Debug, Serialize)]
pub struct FavoriteFolder {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ComicDetail {
    pub id: String,
    pub title: String,
    pub author: Vec<String>,
    pub description: String,
    #[serde(rename = "totalViews")]
    pub total_views: u32,
    pub likes: u32,
    #[serde(rename = "commentTotal")]
    pub comment_total: u32,
    pub tags: Vec<String>,
    pub actors: Vec<String>,
    pub works: Vec<String>,
    #[serde(rename = "isFavorite")]
    pub is_favorite: bool,
    pub liked: bool,
    #[serde(rename = "relatedList")]
    pub related_list: Vec<RelatedComic>,
    pub series: Vec<ComicChapter>,
    #[serde(rename = "seriesId")]
    pub series_id: String,
    pub price: u32,
    pub purchased: bool,
    pub image: String,
}

#[derive(Debug, Serialize)]
pub struct RelatedComic {
    pub id: String,
    pub title: String,
    pub author: String,
    pub image: String,
}

#[derive(Debug, Serialize)]
pub struct ComicChapter {
    pub id: String,
    pub title: String,
    pub sort: String,
}

#[derive(Debug, Serialize)]
pub struct ComicCommentsResult {
    pub endpoint: String,
    pub page: u32,
    pub total: u32,
    pub comments: Vec<ComicComment>,
}

#[derive(Debug, Serialize)]
pub struct ComicComment {
    pub id: String,
    #[serde(rename = "comicId")]
    pub comic_id: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub username: String,
    pub nickname: String,
    pub content: String,
    #[serde(rename = "likeCount")]
    pub like_count: u32,
    pub time: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub avatar: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    pub spoiler: bool,
    pub replies: Vec<ComicComment>,
}

#[derive(Debug, Serialize)]
pub struct WeekCategory {
    pub id: String,
    pub time: String,
    pub title: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct WeekType {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct FeedComic {
    pub id: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub image: String,
    pub tags: Vec<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<i64>,
}

pub(crate) fn map_feed_comic(item: ComicListItemPayload, img_host: Option<&str>) -> FeedComic {
    let mut tags = Vec::new();

    if let Some(title) = item.category.and_then(|category| category.title) {
        if !title.is_empty() {
            tags.push(title);
        }
    }

    if let Some(title) = item.category_sub.and_then(|category| category.title) {
        if !title.is_empty() && !tags.contains(&title) {
            tags.push(title);
        }
    }

    FeedComic {
        image: cover_image_url(img_host, &item.id).unwrap_or(item.image),
        id: item.id,
        title: item.name,
        author: item.author,
        description: item.description.unwrap_or_default(),
        tags,
        updated_at: item.update_at,
    }
}

pub(crate) fn map_favorite_comic(item: FavoriteComicPayload, img_host: Option<&str>) -> FeedComic {
    let mut tags = Vec::new();

    if let Some(title) = item.category.and_then(|category| category.title) {
        if !title.is_empty() {
            tags.push(title);
        }
    }

    if let Some(title) = item.category_sub.and_then(|category| category.title) {
        if !title.is_empty() && !tags.contains(&title) {
            tags.push(title);
        }
    }

    FeedComic {
        image: cover_image_url(img_host, &item.id).unwrap_or(item.image),
        id: item.id,
        title: item.name,
        author: item.author,
        description: item.description.unwrap_or_default(),
        tags,
        updated_at: item.update_at,
    }
}

pub(crate) fn build_search_result(
    page: u32,
    total: u32,
    items: Vec<PluginComicListItem>,
    extern_payload: HashMap<String, serde_json::Value>,
) -> SearchResultContract {
    let has_reached_max = items.is_empty()
        || items.len() < SEARCH_PAGE_SIZE
        || (total > 0
            && ((page.saturating_sub(1) as usize) * SEARCH_PAGE_SIZE + items.len())
                >= total as usize);
    let paging = SearchPagingInfo {
        page,
        pages: page,
        total,
        has_reached_max,
    };
    let scheme = SearchResultScheme {
        version: "1.0.0".to_string(),
        scheme_type: "searchResult".to_string(),
        source: JM_PLUGIN_ID.to_string(),
        list: "comicGrid".to_string(),
    };
    let data = SearchResultData {
        paging: paging.clone(),
        items: items.clone(),
    };

    SearchResultContract {
        source: JM_PLUGIN_ID.to_string(),
        r#extern: extern_payload,
        scheme,
        data,
        paging,
        items,
    }
}

pub(crate) fn normalize_search_extern(
    extern_payload: Option<HashMap<String, serde_json::Value>>,
) -> HashMap<String, serde_json::Value> {
    let mut extern_payload = extern_payload.unwrap_or_default();
    let sort_by = extern_payload
        .get("sortBy")
        .and_then(json_value_to_u32)
        .unwrap_or(1);

    extern_payload.insert(
        "sortBy".to_string(),
        serde_json::Value::Number(serde_json::Number::from(sort_by)),
    );

    extern_payload
}

pub(crate) fn search_order_from_extern(
    extern_payload: &HashMap<String, serde_json::Value>,
) -> String {
    let order = extern_payload
        .get("sort")
        .and_then(json_value_to_string)
        .unwrap_or_default();

    if !order.trim().is_empty() {
        return order.trim().to_string();
    }

    match extern_payload.get("sortBy").and_then(json_value_to_u32) {
        Some(2) => "mv".to_string(),
        Some(3) => "mp".to_string(),
        Some(4) => "tf".to_string(),
        _ => "mr".to_string(),
    }
}

pub(crate) fn json_value_to_u32(value: &serde_json::Value) -> Option<u32> {
    match value {
        serde_json::Value::Number(number) => {
            number.as_u64().and_then(|value| u32::try_from(value).ok())
        }
        serde_json::Value::String(value) => value.trim().parse::<u32>().ok(),
        _ => None,
    }
}

pub(crate) fn json_value_to_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) => Some(value.to_string()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

pub(crate) fn direct_search_comic_id(keyword: &str) -> Option<String> {
    let keyword = keyword.trim();

    if keyword.is_empty() {
        return None;
    }

    let lowercase = keyword.to_ascii_lowercase();
    let comic_id = if lowercase.starts_with("jm") {
        keyword.get(2..).unwrap_or_default().trim()
    } else {
        keyword
    };

    if comic_id
        .parse::<u32>()
        .ok()
        .filter(|value| *value >= 100)
        .is_some()
    {
        Some(comic_id.to_string())
    } else {
        None
    }
}

pub(crate) fn build_jm_cover_url(img_host: Option<&str>, comic_id: &str, image: &str) -> String {
    let image = image.trim();

    if image.starts_with("http://") || image.starts_with("https://") {
        return image.to_string();
    }

    let Some(img_host) = img_host else {
        return image.to_string();
    };
    let img_host = img_host.trim().trim_end_matches('/');

    if img_host.is_empty() {
        return image.to_string();
    }

    if image.starts_with('/') {
        return format!("{img_host}{image}");
    }

    if image.starts_with("media/") {
        return format!("{img_host}/{image}");
    }

    if comic_id.trim().is_empty() {
        image.to_string()
    } else {
        format!("{img_host}/media/albums/{comic_id}_3x4.jpg")
    }
}

pub(crate) fn search_payload_from_detail(payload: ComicDetailPayload) -> SearchComicPayload {
    SearchComicPayload {
        id: payload.id,
        author: payload.author.join("/"),
        description: Some(payload.description),
        name: payload.name,
        image: String::new(),
        category: None,
        category_sub: None,
        update_at: None,
        likes: payload.likes,
        total_views: payload.total_views,
        tags: payload.tags,
        works: payload.works,
        actors: payload.actors,
        liked: payload.liked,
        is_favorite: payload.is_favorite,
    }
}

pub(crate) fn non_empty_string(value: Option<&str>) -> Option<String> {
    let value = value?.trim();

    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

pub(crate) fn map_search_comic_item(
    item: SearchComicPayload,
    img_host: Option<&str>,
) -> PluginComicListItem {
    let id = item.id;
    let cover_url = build_jm_cover_url(img_host, &id, &item.image);
    let updated_at = item
        .update_at
        .map(|value| value.to_string())
        .unwrap_or_default();
    let author = item.author;
    let description = item.description.unwrap_or_default();
    let category = item.category;
    let category_sub = item.category_sub;
    let category_title = category
        .as_ref()
        .and_then(|category| non_empty_string(category.title.as_deref()));
    let category_sub_title = category_sub
        .as_ref()
        .and_then(|category| non_empty_string(category.title.as_deref()));
    let category_id = category.map(|category| category.id);
    let category_sub_id = category_sub.map(|category| category.id);
    let name = item.name;
    let tags = item.tags;
    let works = item.works;
    let actors = item.actors;
    let image = item.image;

    PluginComicListItem {
        source: JM_PLUGIN_ID.to_string(),
        id: id.clone(),
        title: name.clone(),
        subtitle: String::new(),
        finished: false,
        likes_count: item.likes,
        views_count: item.total_views,
        updated_at: updated_at.clone(),
        cover: PluginImageItem {
            id: id.clone(),
            url: cover_url,
            name: String::new(),
            path: format!("{id}.jpg"),
            r#extern: hashmap_from_pairs([(
                "path",
                serde_json::Value::String(format!("{id}.jpg")),
            )]),
        },
        metadata: search_metadata(
            author.as_str(),
            category_title.as_deref(),
            category_sub_title.as_deref(),
            &tags,
            &works,
            &actors,
        ),
        raw: search_raw(
            &id,
            &name,
            &author,
            &description,
            &updated_at,
            &image,
            category_id,
            category_title,
            category_sub_id,
            category_sub_title,
            item.liked,
            item.is_favorite,
            item.likes,
            item.total_views,
            &tags,
            &works,
            &actors,
        ),
        r#extern: HashMap::new(),
    }
}

pub(crate) fn search_metadata(
    author: &str,
    category_title: Option<&str>,
    category_sub_title: Option<&str>,
    tags: &[String],
    works: &[String],
    actors: &[String],
) -> Vec<PluginMetadataListItem> {
    let mut metadata = Vec::new();

    push_metadata(&mut metadata, "author", "作者", [author.to_string()]);
    push_metadata(
        &mut metadata,
        "categories",
        "分类",
        [
            category_title.unwrap_or_default().to_string(),
            category_sub_title.unwrap_or_default().to_string(),
        ],
    );
    push_metadata(&mut metadata, "tags", "标签", tags.iter().cloned());
    push_metadata(&mut metadata, "works", "作品", works.iter().cloned());
    push_metadata(&mut metadata, "actors", "角色", actors.iter().cloned());

    metadata
}

pub(crate) fn push_metadata<I>(
    metadata: &mut Vec<PluginMetadataListItem>,
    metadata_type: &str,
    name: &str,
    values: I,
) where
    I: IntoIterator<Item = String>,
{
    let value = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if value.is_empty() {
        return;
    }

    metadata.push(PluginMetadataListItem {
        metadata_type: metadata_type.to_string(),
        name: name.to_string(),
        value,
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn search_raw(
    id: &str,
    name: &str,
    author: &str,
    description: &str,
    updated_at: &str,
    image: &str,
    category_id: Option<String>,
    category_title: Option<String>,
    category_sub_id: Option<String>,
    category_sub_title: Option<String>,
    liked: bool,
    is_favorite: bool,
    likes: u32,
    total_views: u32,
    tags: &[String],
    works: &[String],
    actors: &[String],
) -> HashMap<String, serde_json::Value> {
    hashmap_from_pairs([
        ("id", serde_json::Value::String(id.to_string())),
        ("author", serde_json::Value::String(author.to_string())),
        (
            "description",
            serde_json::Value::String(description.to_string()),
        ),
        ("name", serde_json::Value::String(name.to_string())),
        ("image", serde_json::Value::String(image.to_string())),
        (
            "category",
            serde_json::json!({
                "id": category_id.unwrap_or_default(),
                "title": category_title.unwrap_or_default()
            }),
        ),
        (
            "category_sub",
            serde_json::json!({
                "id": category_sub_id,
                "title": category_sub_title
            }),
        ),
        ("liked", serde_json::Value::Bool(liked)),
        ("is_favorite", serde_json::Value::Bool(is_favorite)),
        (
            "update_at",
            serde_json::Value::Number(serde_json::Number::from(
                updated_at.parse::<i64>().unwrap_or_default(),
            )),
        ),
        (
            "likes",
            serde_json::Value::Number(serde_json::Number::from(likes)),
        ),
        (
            "totalViews",
            serde_json::Value::Number(serde_json::Number::from(total_views)),
        ),
        ("tags", serde_json::json!(tags)),
        ("works", serde_json::json!(works)),
        ("actors", serde_json::json!(actors)),
    ])
}

pub(crate) fn hashmap_from_pairs<const N: usize>(
    pairs: [(&str, serde_json::Value); N],
) -> HashMap<String, serde_json::Value> {
    pairs
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

pub(crate) fn map_comic_detail(payload: ComicDetailPayload, img_host: Option<&str>) -> ComicDetail {
    let image = cover_image_url(img_host, &payload.id).unwrap_or_default();

    ComicDetail {
        id: payload.id,
        title: payload.name,
        author: payload.author,
        description: payload.description,
        total_views: payload.total_views,
        likes: payload.likes,
        comment_total: payload.comment_total,
        tags: payload.tags,
        actors: payload.actors,
        works: payload.works,
        is_favorite: payload.is_favorite,
        liked: payload.liked,
        related_list: payload
            .related_list
            .into_iter()
            .map(|item| map_related_comic(item, img_host))
            .collect(),
        series: payload
            .series
            .into_iter()
            .map(|item| ComicChapter {
                id: item.id,
                title: item.name,
                sort: item.sort,
            })
            .collect(),
        series_id: payload.series_id,
        price: payload.price,
        purchased: payload.purchased,
        image,
    }
}

pub(crate) fn map_related_comic(
    item: ComicDetailRelatedPayload,
    img_host: Option<&str>,
) -> RelatedComic {
    let image = cover_image_url(img_host, &item.id).unwrap_or(item.image);

    RelatedComic {
        id: item.id,
        title: item.name,
        author: item.author,
        image,
    }
}

pub(crate) fn map_comment(payload: CommentPayload, img_host: Option<&str>) -> ComicComment {
    ComicComment {
        id: payload.cid,
        comic_id: payload.aid,
        user_id: payload.uid,
        username: payload.username,
        nickname: payload.nickname,
        content: payload.content,
        like_count: payload.likes,
        time: payload.addtime,
        updated_at: payload.update_at,
        avatar: user_avatar_url(img_host, &payload.photo).unwrap_or_default(),
        parent_id: payload.parent_cid,
        spoiler: payload.spoiler,
        replies: payload
            .replys
            .unwrap_or_default()
            .into_iter()
            .map(|reply| map_comment(reply, img_host))
            .collect(),
    }
}

pub(crate) fn map_login_user(payload: LoginPayload, img_host: Option<&str>) -> UserProfile {
    let avatar_url = user_avatar_url(img_host, &payload.photo).unwrap_or_default();

    UserProfile {
        id: payload.uid,
        username: payload.username,
        email: payload.email,
        avatar: payload.photo,
        avatar_url,
        level: payload.level,
        level_name: payload.level_name,
        current_level_exp: payload.exp,
        next_level_exp: payload.next_level_exp,
        exp_percent: payload.exp_percent,
        current_collect_count: payload.album_favorites,
        max_collect_count: payload.album_favorites_max,
        j_coin: payload.coin,
    }
}

pub(crate) fn map_sign_in_records(records: Vec<Vec<SignInRecordPayload>>) -> Vec<SignInRecord> {
    records
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, record)| SignInRecord {
            day: index as u32 + 1,
            date: record.date,
            signed: record.signed,
            bonus: record.bonus,
        })
        .collect()
}

pub(crate) fn map_week_categories(categories: Vec<WeekCategoryPayload>) -> Vec<WeekCategory> {
    categories
        .into_iter()
        .map(|item| WeekCategory {
            label: if item.time.is_empty() {
                item.title.clone()
            } else {
                format!("{} ({})", item.title, item.time)
            },
            id: item.id,
            time: item.time,
            title: item.title,
        })
        .collect()
}

pub(crate) fn map_week_types(types: Vec<WeekTypePayload>) -> Vec<WeekType> {
    types
        .into_iter()
        .map(|item| WeekType {
            id: item.id,
            title: item.title,
        })
        .collect()
}
