use std::{fmt, time::Duration};

use crate::{Category, Cursor, Page, Ratio, Rgb, Size, Sort};

/// What a source can do. The UI builds its filters from this, so a new source needs no UI code.
#[derive(Clone, Copy, Debug)]
pub struct Capabilities {
    pub search: bool,
    pub sorts: &'static [Sort],
    pub categories: &'static [Category],
    pub ratios: &'static [Ratio],
    pub colors: &'static [Rgb],
    pub min_size: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct RateLimit {
    pub requests: usize,
    pub per: Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct ProviderInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub capabilities: Capabilities,
    pub rate: RateLimit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListRequest {
    pub text: String,
    pub sort: Sort,
    /// Empty means every category the source has.
    pub categories: Vec<Category>,
    pub min_size: Option<Size>,
    pub ratios: Vec<Ratio>,
    pub color: Option<Rgb>,
    pub cursor: Option<Cursor>,
}

impl Default for ListRequest {
    fn default() -> Self {
        Self {
            text: String::new(),
            sort: Sort::Random,
            categories: vec![],
            min_size: None,
            ratios: vec![],
            color: None,
            cursor: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderError {
    Network(String),
    RateLimited { retry_after: Option<Duration> },
    Status(u16),
    BadResponse(String),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(reason) => write!(f, "network error: {reason}"),
            Self::RateLimited { .. } => write!(f, "rate limited by the source"),
            Self::Status(code) => write!(f, "the source answered with status {code}"),
            Self::BadResponse(reason) => write!(f, "unexpected answer from the source: {reason}"),
        }
    }
}

impl std::error::Error for ProviderError {}

/// Network access handed to providers. The host behind it owns rate limits and timeouts.
pub trait Http {
    fn get(&self, url: &str) -> Result<Vec<u8>, ProviderError>;
}

/// An image source. A provider only maps requests to URLs and answers to items.
/// Calls block, and always run on a worker thread.
pub trait Provider: Send + Sync {
    fn info(&self) -> &ProviderInfo;
    fn list(&self, request: &ListRequest, http: &dyn Http) -> Result<Page, ProviderError>;
}
