use std::sync::Arc;

#[derive(Clone)]
pub struct AccessGateService {
    password: Option<Arc<str>>,
}

impl AccessGateService {
    pub fn from_env() -> Self {
        let password = std::env::var("JM_BOOM_ACCESS_PASSWORD")
            .ok()
            .filter(|password| !password.is_empty())
            .map(Arc::<str>::from);
        tracing::info!(enabled = password.is_some(), "轻量访问门禁配置完成");
        Self { password }
    }

    pub fn enabled(&self) -> bool {
        self.password.is_some()
    }

    pub fn verify(&self, password: &[u8]) -> bool {
        self.password
            .as_deref()
            .is_none_or(|expected| constant_time_equals(expected.as_bytes(), password))
    }
}

fn constant_time_equals(expected: &[u8], actual: &[u8]) -> bool {
    let mut difference = expected.len() ^ actual.len();
    let length = expected.len().max(actual.len());
    for index in 0..length {
        let left = expected.get(index).copied().unwrap_or_default();
        let right = actual.get(index).copied().unwrap_or_default();
        difference |= usize::from(left ^ right);
    }
    difference == 0
}

#[cfg(test)]
mod tests {
    use super::AccessGateService;
    use std::sync::Arc;

    #[test]
    fn allows_all_passwords_when_the_gate_is_disabled() {
        let gate = AccessGateService { password: None };
        assert!(!gate.enabled());
        assert!(gate.verify(b"anything"));
    }

    #[test]
    fn verifies_the_configured_password() {
        let gate = AccessGateService {
            password: Some(Arc::from("secret")),
        };
        assert!(gate.enabled());
        assert!(gate.verify(b"secret"));
        assert!(!gate.verify(b"wrong"));
    }
}
