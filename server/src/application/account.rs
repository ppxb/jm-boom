use crate::{
    endpoint::{request_with_failover, EndpointManager},
    jm::{decrypt_aes256_ecb, JmClient, JmError, JmResult},
};
use aes::{
    cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyInit},
    Aes256,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, Row, SqlitePool};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

const CREDENTIAL_CRYPTO_SEED: &str = "jm-boom-local-auto-login";

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LoginStatus {
    LoggedOut,
    LoggingIn,
    LoggedIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SignInStatus {
    Pending,
    SigningIn,
    SignedIn,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountState {
    pub username: Option<String>,
    pub auto_login: bool,
    pub auto_sign_in: bool,
    pub login_status: LoginStatus,
    pub sign_in_status: SignInStatus,
}

impl AccountState {
    fn empty() -> Self {
        Self {
            username: None,
            auto_login: true,
            auto_sign_in: true,
            login_status: LoginStatus::LoggedOut,
            sign_in_status: SignInStatus::Pending,
        }
    }
}

pub struct AccountInput {
    pub username: String,
    pub password: Option<String>,
    pub auto_login: bool,
    pub auto_sign_in: bool,
}

#[derive(Clone)]
struct SavedCredentials {
    username: String,
    password: String,
    auto_login: bool,
    auto_sign_in: bool,
}

pub struct AccountService {
    db: SqlitePool,
    jm: Arc<JmClient>,
    endpoints: Arc<EndpointManager>,
    credentials: RwLock<Option<SavedCredentials>>,
    user_id: RwLock<Option<u32>>,
    state: RwLock<AccountState>,
    operation: Mutex<()>,
}

impl AccountService {
    pub async fn new(
        db: SqlitePool,
        jm: Arc<JmClient>,
        endpoints: Arc<EndpointManager>,
    ) -> anyhow::Result<Self> {
        let saved = sqlx::query(
            "SELECT username, password_cipher, auto_login, auto_sign_in FROM jm_account WHERE id = 1",
        )
        .fetch_optional(&db)
        .await?
        .map(decode_saved_credentials)
        .transpose()?;

        let state = saved
            .as_ref()
            .map_or_else(AccountState::empty, |saved| AccountState {
                username: Some(saved.username.clone()),
                auto_login: saved.auto_login,
                auto_sign_in: saved.auto_sign_in,
                login_status: if saved.auto_login {
                    LoginStatus::LoggingIn
                } else {
                    LoginStatus::LoggedOut
                },
                sign_in_status: SignInStatus::Pending,
            });

        Ok(Self {
            db,
            jm,
            endpoints,
            credentials: RwLock::new(saved),
            user_id: RwLock::new(None),
            state: RwLock::new(state),
            operation: Mutex::new(()),
        })
    }

    pub async fn state(&self) -> AccountState {
        self.state.read().await.clone()
    }

    pub fn start_auto_login(self: &Arc<Self>) {
        let service = self.clone();
        tokio::spawn(async move {
            let Some(saved) = service.credentials.read().await.clone() else {
                return;
            };
            if !saved.auto_login {
                return;
            }

            if let Err(error) = service.login(saved).await {
                tracing::warn!(%error, "automatic JM login failed");
            }
        });
    }

    pub async fn save_and_login(self: &Arc<Self>, input: AccountInput) -> JmResult<AccountState> {
        let username = input.username.trim().to_string();
        if username.is_empty() {
            return Err(JmError::MissingData);
        }

        let password = match input.password.map(|password| password.trim().to_string()) {
            Some(password) if !password.is_empty() => password,
            _ => self
                .credentials
                .read()
                .await
                .as_ref()
                .filter(|saved| saved.username == username)
                .map(|saved| saved.password.clone())
                .ok_or(JmError::MissingData)?,
        };
        let saved = SavedCredentials {
            username,
            password,
            auto_login: input.auto_login,
            auto_sign_in: input.auto_sign_in,
        };

        self.login(saved).await?;
        Ok(self.state().await)
    }

    pub async fn clear(self: &Arc<Self>) -> anyhow::Result<AccountState> {
        let _operation = self.operation.lock().await;
        self.jm.set_jwt_token(None).await;
        sqlx::query("DELETE FROM jm_account WHERE id = 1")
            .execute(&self.db)
            .await?;
        *self.credentials.write().await = None;
        *self.user_id.write().await = None;
        *self.state.write().await = AccountState::empty();
        Ok(self.state().await)
    }

    async fn login(self: &Arc<Self>, saved: SavedCredentials) -> JmResult<()> {
        let _operation = self.operation.lock().await;
        self.set_login_state(LoginStatus::LoggingIn).await;

        let login = match self.login_remote(&saved.username, &saved.password).await {
            Ok(login) => login,
            Err(error) => {
                self.jm.set_jwt_token(None).await;
                self.set_login_state(LoginStatus::LoggedOut).await;
                return Err(error);
            }
        };

        self.jm.set_jwt_token(Some(login.jwt_token)).await;
        if let Err(error) = self.persist(&saved).await {
            self.jm.set_jwt_token(None).await;
            self.set_login_state(LoginStatus::LoggedOut).await;
            return Err(JmError::Other(error.to_string()));
        }

        *self.credentials.write().await = Some(saved.clone());
        *self.user_id.write().await = Some(login.user_id);
        {
            let mut state = self.state.write().await;
            state.username = Some(saved.username);
            state.auto_login = saved.auto_login;
            state.auto_sign_in = saved.auto_sign_in;
            state.login_status = LoginStatus::LoggedIn;
            state.sign_in_status = if saved.auto_sign_in {
                SignInStatus::SigningIn
            } else {
                SignInStatus::Pending
            };
        }

        let service = self.clone();
        tokio::spawn(async move {
            if let Err(error) = service.update_sign_in().await {
                tracing::debug!(%error, "JM sign-in status refresh failed");
            }
        });
        Ok(())
    }

    async fn update_sign_in(&self) -> JmResult<()> {
        let _operation = self.operation.lock().await;
        if self.state.read().await.login_status != LoginStatus::LoggedIn {
            return Ok(());
        }

        let user_id = (*self.user_id.read().await).ok_or(JmError::MissingData)?;
        let auto_sign_in = self.state.read().await.auto_sign_in;
        if auto_sign_in {
            self.set_sign_in_status(SignInStatus::SigningIn).await;
        }
        let daily = match self.fetch_daily(user_id).await {
            Ok(daily) => daily,
            Err(error) => {
                self.set_sign_in_status(SignInStatus::Pending).await;
                return Err(error);
            }
        };
        let today = Local::now().format("%Y-%m-%d").to_string();
        let signed = daily
            .record
            .iter()
            .flatten()
            .any(|record| record.date == today && record.signed);

        if signed {
            self.set_sign_in_status(SignInStatus::SignedIn).await;
            return Ok(());
        }

        if !auto_sign_in {
            self.set_sign_in_status(SignInStatus::Pending).await;
            return Ok(());
        }

        if let Err(error) = self.sign_in(user_id, daily.daily_id).await {
            self.set_sign_in_status(SignInStatus::Pending).await;
            return Err(error);
        }
        self.set_sign_in_status(SignInStatus::SignedIn).await;
        Ok(())
    }

    async fn login_remote(&self, username: &str, password: &str) -> JmResult<LoginPayload> {
        let username = username.to_string();
        let password = password.to_string();
        request_with_failover(&self.jm, &self.endpoints, move |client, endpoint| {
            let fields = vec![
                ("username".to_string(), username.clone()),
                ("password".to_string(), password.clone()),
            ];
            Box::pin(async move {
                client
                    .post_form::<LoginResponse>(endpoint, "login", &fields, false)
                    .await
            })
        })
        .await
        .map(|(_, login)| {
            let jwt_token = login
                .jwttoken
                .filter(|token| !token.trim().is_empty())
                .ok_or(JmError::MissingData)?;
            if login.uid == 0 {
                return Err(JmError::MissingData);
            }
            Ok(LoginPayload {
                user_id: login.uid,
                jwt_token,
            })
        })?
    }

    async fn fetch_daily(&self, user_id: u32) -> JmResult<DailyPayload> {
        request_with_failover(&self.jm, &self.endpoints, move |client, endpoint| {
            Box::pin(async move {
                client
                    .get(endpoint, "daily", &[("user_id", user_id.to_string())])
                    .await
            })
        })
        .await
        .map(|(_, daily)| daily)
    }

    async fn sign_in(&self, user_id: u32, daily_id: u32) -> JmResult<()> {
        request_with_failover(&self.jm, &self.endpoints, move |client, endpoint| {
            let fields = vec![
                ("user_id".to_string(), user_id.to_string()),
                ("daily_id".to_string(), daily_id.to_string()),
            ];
            Box::pin(async move {
                client
                    .post_form::<SignInPayload>(endpoint, "daily_chk", &fields, true)
                    .await
            })
        })
        .await
        .map(|_| ())
    }

    async fn persist(&self, saved: &SavedCredentials) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO jm_account (id, username, password_cipher, auto_login, auto_sign_in, updated_at) VALUES (1, ?, ?, ?, ?, strftime('%s','now')) ON CONFLICT(id) DO UPDATE SET username = excluded.username, password_cipher = excluded.password_cipher, auto_login = excluded.auto_login, auto_sign_in = excluded.auto_sign_in, updated_at = excluded.updated_at",
        )
        .bind(&saved.username)
        .bind(encrypt_password(&saved.password)?)
        .bind(saved.auto_login)
        .bind(saved.auto_sign_in)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    async fn set_login_state(&self, status: LoginStatus) {
        self.state.write().await.login_status = status;
    }

    async fn set_sign_in_status(&self, status: SignInStatus) {
        self.state.write().await.sign_in_status = status;
    }
}

#[derive(Deserialize)]
struct LoginResponse {
    #[serde(default)]
    jwttoken: Option<String>,
    #[serde(default, deserialize_with = "u32_from_value")]
    uid: u32,
}

struct LoginPayload {
    user_id: u32,
    jwt_token: String,
}

#[derive(Deserialize)]
struct DailyPayload {
    #[serde(default, deserialize_with = "u32_from_value")]
    daily_id: u32,
    #[serde(default)]
    record: Vec<Vec<DailyRecord>>,
}

#[derive(Deserialize)]
struct DailyRecord {
    #[serde(default)]
    date: String,
    #[serde(default, deserialize_with = "bool_from_value")]
    signed: bool,
}

#[derive(Deserialize)]
struct SignInPayload {
    #[allow(dead_code)]
    #[serde(default)]
    msg: String,
}

fn decode_saved_credentials(row: SqliteRow) -> anyhow::Result<SavedCredentials> {
    let password_cipher: String = row.try_get("password_cipher")?;
    Ok(SavedCredentials {
        username: row.try_get("username")?,
        password: decrypt_password(&password_cipher)?,
        auto_login: row.try_get::<i64, _>("auto_login")? != 0,
        auto_sign_in: row.try_get::<i64, _>("auto_sign_in")? != 0,
    })
}

fn encrypt_password(password: &str) -> anyhow::Result<String> {
    let encrypted = ecb::Encryptor::<Aes256>::new_from_slice(credential_key().as_bytes())
        .map_err(|error| anyhow::anyhow!(error.to_string()))?
        .encrypt_padded_vec_mut::<Pkcs7>(password.as_bytes());
    Ok(BASE64.encode(encrypted))
}

fn decrypt_password(password: &str) -> anyhow::Result<String> {
    decrypt_aes256_ecb(password, &credential_key())
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

fn credential_key() -> String {
    format!("{:x}", md5::compute(CREDENTIAL_CRYPTO_SEED))
}

fn u32_from_value<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(value
        .as_u64()
        .map(|value| value as u32)
        .or_else(|| value.as_str()?.parse().ok())
        .unwrap_or_default())
}

fn bool_from_value<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Bool(value) => value,
        serde_json::Value::Number(value) => value.as_i64().unwrap_or_default() != 0,
        serde_json::Value::String(value) => matches!(value.as_str(), "1" | "true" | "yes"),
        _ => false,
    })
}
