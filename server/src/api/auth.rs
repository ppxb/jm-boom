use crate::{
    jm::{JmError, JmResult},
    AppState,
};
use axum::{extract::State, Json};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub avatar: String,
    pub avatar_url: String,
    pub level: u32,
    pub level_name: String,
    pub current_level_exp: u32,
    pub next_level_exp: u32,
    pub exp_percent: f32,
    pub current_collect_count: u32,
    pub max_collect_count: u32,
    pub j_coin: u32,
}

#[derive(Serialize)]
pub struct LoginResponse {
    user: UserProfile,
}

#[derive(Deserialize)]
pub struct SignInRequest {
    #[serde(rename = "userId")]
    user_id: u32,
    #[serde(rename = "dailyId")]
    daily_id: u32,
}

#[derive(Deserialize)]
pub struct SignInQuery {
    #[serde(rename = "userId")]
    user_id: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInDataResult {
    daily_id: u32,
    three_days_coin: u32,
    three_days_exp: u32,
    seven_days_coin: u32,
    seven_days_exp: u32,
    event_name: String,
    current_progress: String,
    background_pc: String,
    background_phone: String,
    records: Vec<SignInRecord>,
}

#[derive(Serialize)]
pub struct SignInRecord {
    day: u32,
    date: String,
    signed: bool,
    bonus: bool,
}

#[derive(Serialize)]
pub struct SignInResult {
    message: String,
}

pub async fn login(
    State(app): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> JmResult<Json<LoginResponse>> {
    let username = payload.username.trim().to_string();
    if username.is_empty() || payload.password.trim().is_empty() {
        return Err(JmError::MissingData);
    }
    let password = payload.password;
    let login: LoginPayload = app
        .jm_request(move |client, endpoint| {
            let username = username.clone();
            let password = password.clone();
            Box::pin(async move {
                client
                    .post_form(
                        endpoint,
                        "login",
                        &[
                            ("username".to_string(), username),
                            ("password".to_string(), password),
                        ],
                        false,
                    )
                    .await
            })
        })
        .await?;
    let jwt = login
        .jwttoken
        .clone()
        .filter(|token| !token.trim().is_empty())
        .ok_or(JmError::MissingData)?;
    app.jm.set_jwt_token(Some(jwt)).await;
    let img_host = app.img_host().await;
    let user = map_user(login, img_host.as_deref());
    *app.session.write().await = Some(user.clone());
    Ok(Json(LoginResponse { user }))
}

pub async fn get_session(State(app): State<AppState>) -> Json<Option<LoginResponse>> {
    Json(
        app.session
            .read()
            .await
            .clone()
            .map(|user| LoginResponse { user }),
    )
}

pub async fn clear_session(State(app): State<AppState>) {
    app.jm.set_jwt_token(None).await;
    *app.session.write().await = None;
}

pub async fn get_sign_in_data(
    State(app): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<SignInQuery>,
) -> JmResult<Json<SignInDataResult>> {
    if query.user_id == 0 {
        return Err(JmError::MissingData);
    }
    let user_id = query.user_id;
    let payload: SignInDataPayload = app
        .jm_request(move |client, endpoint| {
            Box::pin(async move {
                client
                    .get(endpoint, "daily", &[("user_id", user_id.to_string())])
                    .await
            })
        })
        .await?;
    let records = payload
        .record
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, item)| SignInRecord {
            day: item
                .date
                .rsplit('-')
                .next()
                .and_then(|value| value.parse().ok())
                .unwrap_or(index as u32 + 1),
            date: item.date,
            signed: item.signed,
            bonus: item.bonus,
        })
        .collect();
    Ok(Json(SignInDataResult {
        daily_id: payload.daily_id,
        three_days_coin: payload.three_days_coin,
        three_days_exp: payload.three_days_exp,
        seven_days_coin: payload.seven_days_coin,
        seven_days_exp: payload.seven_days_exp,
        event_name: payload.event_name,
        current_progress: payload.current_progress,
        background_pc: payload.background_pc,
        background_phone: payload.background_phone,
        records,
    }))
}

pub async fn sign_in(
    State(app): State<AppState>,
    Json(payload): Json<SignInRequest>,
) -> JmResult<Json<SignInResult>> {
    if payload.user_id == 0 || payload.daily_id == 0 {
        return Err(JmError::MissingData);
    }
    let user_id = payload.user_id;
    let daily_id = payload.daily_id;
    let result: SignInPayload = app
        .jm_request(move |client, endpoint| {
            Box::pin(async move {
                client
                    .post_form(
                        endpoint,
                        "daily_chk",
                        &[
                            ("user_id".to_string(), user_id.to_string()),
                            ("daily_id".to_string(), daily_id.to_string()),
                        ],
                        true,
                    )
                    .await
            })
        })
        .await?;
    Ok(Json(SignInResult {
        message: result.msg,
    }))
}

#[derive(Deserialize)]
struct LoginPayload {
    #[serde(default, deserialize_with = "optional_string_from_value")]
    jwttoken: Option<String>,
    #[serde(default, deserialize_with = "u32_from_value")]
    uid: u32,
    #[serde(default, deserialize_with = "string_from_value")]
    username: String,
    #[serde(default, deserialize_with = "string_from_value")]
    email: String,
    #[serde(default, deserialize_with = "string_from_value")]
    photo: String,
    #[serde(default, deserialize_with = "u32_from_value")]
    coin: u32,
    #[serde(default, deserialize_with = "u32_from_value")]
    album_favorites: u32,
    #[serde(default, deserialize_with = "string_from_value")]
    level_name: String,
    #[serde(default, deserialize_with = "u32_from_value")]
    level: u32,
    #[serde(default, rename = "nextLevelExp", deserialize_with = "u32_from_value")]
    next_level_exp: u32,
    #[serde(default, deserialize_with = "u32_from_value")]
    exp: u32,
    #[serde(default, rename = "expPercent", deserialize_with = "f32_from_value")]
    exp_percent: f32,
    #[serde(default, deserialize_with = "u32_from_value")]
    album_favorites_max: u32,
}

#[derive(Deserialize)]
struct SignInDataPayload {
    #[serde(deserialize_with = "u32_from_value")]
    daily_id: u32,
    #[serde(default, deserialize_with = "u32_from_value")]
    three_days_coin: u32,
    #[serde(default, deserialize_with = "u32_from_value")]
    three_days_exp: u32,
    #[serde(default, deserialize_with = "u32_from_value")]
    seven_days_coin: u32,
    #[serde(default, deserialize_with = "u32_from_value")]
    seven_days_exp: u32,
    #[serde(default)]
    event_name: String,
    #[serde(default, rename = "currentProgress")]
    current_progress: String,
    #[serde(default)]
    background_pc: String,
    #[serde(default)]
    background_phone: String,
    #[serde(default)]
    record: Vec<Vec<SignInRecordPayload>>,
}

#[derive(Deserialize)]
struct SignInRecordPayload {
    #[serde(default)]
    date: String,
    #[serde(default, deserialize_with = "bool_from_value")]
    signed: bool,
    #[serde(default, deserialize_with = "bool_from_value")]
    bonus: bool,
}

#[derive(Deserialize)]
struct SignInPayload {
    #[serde(default)]
    msg: String,
}

fn map_user(payload: LoginPayload, img_host: Option<&str>) -> UserProfile {
    let avatar_url = if payload.photo.starts_with("http") {
        payload.photo.clone()
    } else {
        img_host
            .map(|host| {
                format!(
                    "{}/media/users/{}",
                    host.trim_end_matches('/'),
                    payload.photo.trim_start_matches('/')
                )
            })
            .unwrap_or_default()
    };
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

fn value_from<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
where
    D: Deserializer<'de>,
{
    serde_json::Value::deserialize(deserializer)
}

fn string_from_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = value_from(deserializer)?;
    Ok(match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(value) => value,
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        value => value.to_string(),
    })
}

fn optional_string_from_value<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = string_from_value(deserializer)?;
    Ok((!value.trim().is_empty()).then_some(value))
}

fn u32_from_value<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = value_from(deserializer)?;
    Ok(value
        .as_u64()
        .map(|value| value as u32)
        .or_else(|| value.as_str()?.trim_end_matches('%').parse().ok())
        .unwrap_or_default())
}

fn f32_from_value<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = value_from(deserializer)?;
    Ok(value
        .as_f64()
        .map(|value| value as f32)
        .or_else(|| value.as_str()?.trim_end_matches('%').parse().ok())
        .unwrap_or_default())
}

fn bool_from_value<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = value_from(deserializer)?;
    Ok(match value {
        serde_json::Value::Bool(value) => value,
        serde_json::Value::Number(value) => value.as_i64().unwrap_or_default() != 0,
        serde_json::Value::String(value) => matches!(value.as_str(), "1" | "true" | "yes"),
        _ => false,
    })
}
