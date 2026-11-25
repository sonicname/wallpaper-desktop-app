use std::{
    collections::VecDeque,
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use ureq::{
    Agent,
    tls::{RootCerts, TlsConfig, TlsProvider},
};
use wallpaper_core::{Http, ProviderError, RateLimit};

const MAX_PAGE_BYTES: u64 = 4 << 20;
const MAX_IMAGE_BYTES: u64 = 40 << 20;

/// Blocking HTTP client. API calls through [`Http`] are rate limited; image downloads are not.
pub struct HostHttp {
    agent: Agent,
    limit: RateLimit,
    sent: Mutex<VecDeque<Instant>>,
}

impl HostHttp {
    pub fn new(limit: RateLimit) -> Self {
        // SChannel and the Windows certificate store keep the binary small.
        let tls = TlsConfig::builder()
            .provider(TlsProvider::NativeTls)
            .root_certs(RootCerts::PlatformVerifier)
            .build();
        let config = Agent::config_builder()
            .https_only(true)
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(10)))
            // Wallhaven's random listing has been seen to take up to 10 seconds.
            .timeout_global(Some(Duration::from_secs(30)))
            .user_agent("WallpaperClient/0.1")
            .tls_config(tls)
            .build();
        Self {
            agent: config.new_agent(),
            limit,
            sent: Mutex::default(),
        }
    }

    /// Downloads an image without counting it against the API limit.
    pub fn image(&self, url: &str) -> Result<Vec<u8>, ProviderError> {
        self.fetch(url, MAX_IMAGE_BYTES)
    }

    fn fetch(&self, url: &str, max: u64) -> Result<Vec<u8>, ProviderError> {
        let network = |e: ureq::Error| ProviderError::Network(e.to_string());
        let mut response = self.agent.get(url).call().map_err(network)?;
        match response.status().as_u16() {
            200 => response
                .body_mut()
                .with_config()
                .limit(max)
                .read_to_vec()
                .map_err(network),
            429 => {
                let header = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok());
                let retry_after = header.and_then(|v| v.parse().ok()).map(Duration::from_secs);
                Err(ProviderError::RateLimited { retry_after })
            }
            code => Err(ProviderError::Status(code)),
        }
    }
}

impl Http for HostHttp {
    fn get(&self, url: &str) -> Result<Vec<u8>, ProviderError> {
        loop {
            let wait = reserve(&mut self.sent.lock().unwrap(), self.limit, Instant::now());
            match wait {
                None => return self.fetch(url, MAX_PAGE_BYTES),
                Some(duration) => thread::sleep(duration),
            }
        }
    }
}

/// Records a request and returns `None`, or returns how long to wait before trying again.
fn reserve(sent: &mut VecDeque<Instant>, limit: RateLimit, now: Instant) -> Option<Duration> {
    while sent
        .front()
        .is_some_and(|t| now.duration_since(*t) >= limit.per)
    {
        sent.pop_front();
    }
    match sent.front() {
        Some(oldest) if sent.len() >= limit.requests => {
            Some(limit.per - now.duration_since(*oldest))
        }
        _ => {
            sent.push_back(now);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waits_until_the_oldest_request_leaves_the_window() {
        let limit = RateLimit {
            requests: 2,
            per: Duration::from_secs(60),
        };
        let (mut sent, start) = (VecDeque::new(), Instant::now());
        let at = |s| start + Duration::from_secs(s);
        assert_eq!(reserve(&mut sent, limit, at(0)), None);
        assert_eq!(reserve(&mut sent, limit, at(10)), None);
        assert_eq!(
            reserve(&mut sent, limit, at(20)),
            Some(Duration::from_secs(40))
        );
        assert_eq!(reserve(&mut sent, limit, at(60)), None);
        assert_eq!(
            reserve(&mut sent, limit, at(60)),
            Some(Duration::from_secs(10))
        );
    }
}
