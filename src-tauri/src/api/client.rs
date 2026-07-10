use super::*;

static NETWORK_CLIENT_STATE: OnceLock<Mutex<NetworkClientState>> = OnceLock::new();

struct NetworkClientState {
    proxy_config: NetworkProxyConfig,
    shared_client: Option<reqwest::Client>,
}

impl Default for NetworkClientState {
    fn default() -> Self {
        Self {
            proxy_config: NetworkProxyConfig::default(),
            shared_client: None,
        }
    }
}

impl NetworkClientState {
    /// 原子更新代理配置，并在配置变化时使共享客户端失效。
    fn apply_proxy_config(&mut self, next_config: NetworkProxyConfig) -> bool {
        if self.proxy_config == next_config {
            return false;
        }

        self.proxy_config = next_config;
        self.shared_client = None;
        true
    }

    /// 返回与当前代理配置一致的共享 HTTP 客户端。
    fn shared_client(&mut self) -> ApiResult<reqwest::Client> {
        if let Some(client) = self.shared_client.as_ref() {
            return Ok(client.clone());
        }

        let next_client = create_http_client_for_config(&self.proxy_config)?;
        self.shared_client = Some(next_client.clone());
        Ok(next_client)
    }
}

/// 返回统一管理代理配置和共享客户端的全局网络状态。
fn network_client_state() -> &'static Mutex<NetworkClientState> {
    NETWORK_CLIENT_STATE.get_or_init(|| Mutex::new(NetworkClientState::default()))
}

/// 获取全局网络状态锁，并统一转换锁中毒错误。
fn lock_network_client_state() -> ApiResult<std::sync::MutexGuard<'static, NetworkClientState>> {
    network_client_state()
        .lock()
        .map_err(|error| ApiError::new(ApiErrorKind::Client, error.to_string()))
}

/// 清理当前进程内的登录令牌。
pub fn clear_session() {
    if let Some(jwt_token) = JWT_TOKEN.get() {
        if let Ok(mut jwt_token) = jwt_token.lock() {
            *jwt_token = None;
        }
    }
}

/// 更新网络代理，并在同一临界区内使旧共享客户端失效。
pub fn configure_network_proxy(
    mode: String,
    host: Option<String>,
    port: Option<u16>,
) -> ApiResult<()> {
    let next_config = normalize_network_proxy_config(mode, host, port)?;
    let mut state = lock_network_client_state()?;
    state.apply_proxy_config(next_config);

    Ok(())
}

/// 获取共享 HTTP 客户端，首次访问时按当前代理配置创建。
pub(crate) fn build_http_client() -> ApiResult<reqwest::Client> {
    lock_network_client_state()?.shared_client()
}

/// 按当前代理配置创建独立 HTTP 客户端。
pub(crate) fn create_http_client() -> ApiResult<reqwest::Client> {
    let state = lock_network_client_state()?;
    create_http_client_for_config(&state.proxy_config)
}

/// 按指定代理配置创建 HTTP 客户端，避免客户端构建过程再次获取网络状态锁。
fn create_http_client_for_config(proxy_config: &NetworkProxyConfig) -> ApiResult<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(8));

    if let Some(proxy_url) = proxy_url_from_config(proxy_config) {
        let proxy = reqwest::Proxy::all(&proxy_url).map_err(|error| {
            ApiError::new(
                ApiErrorKind::Client,
                format!("Invalid proxy {proxy_url}: {error}"),
            )
        })?;
        builder = builder.proxy(proxy);
    }

    builder
        .build()
        .map_err(|error| ApiError::new(ApiErrorKind::Client, error.to_string()))
}

/// 更新当前进程内的登录令牌。
pub(crate) fn set_jwt_token(token: Option<&str>) -> ApiResult<()> {
    let token = token
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string);
    let jwt_token = JWT_TOKEN.get_or_init(|| Mutex::new(None));
    let mut jwt_token = jwt_token
        .lock()
        .map_err(|error| ApiError::new(ApiErrorKind::Client, error.to_string()))?;

    *jwt_token = token;

    Ok(())
}

pub(crate) fn current_jwt_token() -> ApiResult<Option<String>> {
    let jwt_token = JWT_TOKEN.get_or_init(|| Mutex::new(None));
    jwt_token
        .lock()
        .map(|token| token.clone())
        .map_err(|error| ApiError::new(ApiErrorKind::Client, error.to_string()))
}

pub(crate) trait JmRequestBuilderExt {
    fn with_jm_headers(
        self,
        url: &str,
        auth: &ApiAuth,
        use_jwt: bool,
    ) -> ApiResult<reqwest::RequestBuilder>;
}

impl JmRequestBuilderExt for reqwest::RequestBuilder {
    fn with_jm_headers(
        self,
        url: &str,
        auth: &ApiAuth,
        use_jwt: bool,
    ) -> ApiResult<reqwest::RequestBuilder> {
        let builder = self
            .header("accept", "application/json")
            .header("token", &auth.token)
            .header("tokenparam", &auth.tokenparam)
            .header("user-agent", android_user_agent());
        let builder = if let Some(host) = request_url_host(url) {
            builder.header("Host", host)
        } else {
            builder
        };
        let builder = if use_jwt {
            if let Some(jwt) = current_jwt_token()? {
                builder.header("Authorization", format!("Bearer {jwt}"))
            } else {
                builder
            }
        } else {
            builder
        };

        Ok(builder)
    }
}

pub(crate) fn normalize_network_proxy_config(
    mode: String,
    host: Option<String>,
    port: Option<u16>,
) -> ApiResult<NetworkProxyConfig> {
    let default_config = NetworkProxyConfig::default();
    let mode = match mode.trim().to_ascii_lowercase().as_str() {
        "" | "off" | "none" | "disabled" => NetworkProxyMode::Off,
        "http" | "https" => NetworkProxyMode::Http,
        "socks" | "socks5" => NetworkProxyMode::Socks5,
        value => {
            return Err(ApiError::new(
                ApiErrorKind::UnsupportedEndpoint,
                format!("Unsupported proxy mode: {value}"),
            ));
        }
    };

    if mode == NetworkProxyMode::Off {
        return Ok(default_config);
    }

    let host = host
        .unwrap_or(default_config.host)
        .trim()
        .trim_end_matches('/')
        .to_string();
    let port = port.unwrap_or(default_config.port);

    if host.is_empty() {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Proxy host is required",
        ));
    }

    if port == 0 {
        return Err(ApiError::new(
            ApiErrorKind::MissingData,
            "Proxy port must be greater than 0",
        ));
    }

    Ok(NetworkProxyConfig { mode, host, port })
}

/// 返回当前代理配置对应的代理 URL。
pub(crate) fn current_proxy_url() -> ApiResult<Option<String>> {
    let state = lock_network_client_state()?;
    Ok(proxy_url_from_config(&state.proxy_config))
}

/// 将代理配置格式化为 reqwest 可识别的代理 URL。
fn proxy_url_from_config(proxy_config: &NetworkProxyConfig) -> Option<String> {
    let scheme = match proxy_config.mode {
        NetworkProxyMode::Off => return None,
        NetworkProxyMode::Http => "http",
        NetworkProxyMode::Socks5 => "socks5h",
    };
    let host = if proxy_config.host.contains(':')
        && !proxy_config.host.starts_with('[')
        && !proxy_config.host.ends_with(']')
    {
        format!("[{}]", proxy_config.host)
    } else {
        proxy_config.host.clone()
    };

    Some(format!("{scheme}://{host}:{}", proxy_config.port))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{mpsc, Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    /// 验证代理变化会原子清除旧客户端，而相同配置不会重复失效。
    #[test]
    fn proxy_update_invalidates_shared_client_atomically() {
        let mut state = NetworkClientState::default();
        state.shared_client().expect("默认客户端应创建成功");

        assert!(!state.apply_proxy_config(NetworkProxyConfig::default()));
        assert!(state.shared_client.is_some());

        let next_config = NetworkProxyConfig {
            mode: NetworkProxyMode::Http,
            host: "127.0.0.1".to_string(),
            port: 7890,
        };
        assert!(state.apply_proxy_config(next_config));
        assert!(state.shared_client.is_none());
    }

    /// 验证并发代理更新和客户端构建由单一状态锁串行化并能在超时前完成。
    #[test]
    fn concurrent_proxy_updates_and_client_builds_complete() {
        const ITERATIONS: usize = 64;

        let state = Arc::new(Mutex::new(NetworkClientState::default()));
        let start = Arc::new(Barrier::new(3));
        let (completion_tx, completion_rx) = mpsc::channel();

        let proxy_state = Arc::clone(&state);
        let proxy_start = Arc::clone(&start);
        let proxy_completion = completion_tx.clone();
        let proxy_thread = thread::spawn(move || {
            proxy_start.wait();
            for index in 0..ITERATIONS {
                let next_config = if index % 2 == 0 {
                    NetworkProxyConfig {
                        mode: NetworkProxyMode::Http,
                        host: "127.0.0.1".to_string(),
                        port: 7890,
                    }
                } else {
                    NetworkProxyConfig::default()
                };
                proxy_state
                    .lock()
                    .expect("网络状态锁不应中毒")
                    .apply_proxy_config(next_config);
                thread::yield_now();
            }
            proxy_completion.send(()).expect("应发送代理线程完成信号");
        });

        let client_state = Arc::clone(&state);
        let client_start = Arc::clone(&start);
        let client_completion = completion_tx;
        let client_thread = thread::spawn(move || {
            client_start.wait();
            for _ in 0..ITERATIONS {
                client_state
                    .lock()
                    .expect("网络状态锁不应中毒")
                    .shared_client()
                    .expect("共享客户端应创建成功");
                thread::yield_now();
            }
            client_completion
                .send(())
                .expect("应发送客户端线程完成信号");
        });

        start.wait();
        for _ in 0..2 {
            completion_rx
                .recv_timeout(Duration::from_secs(5))
                .expect("并发网络状态操作不应阻塞");
        }

        proxy_thread.join().expect("代理线程应正常退出");
        client_thread.join().expect("客户端线程应正常退出");
    }
}
