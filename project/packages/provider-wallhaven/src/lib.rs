//! Wallhaven API v1, without an API key and limited to SFW images.
//! Docs: <https://wallhaven.cc/help/api>

use std::time::Duration;

use serde::{Deserialize, Deserializer};
use wallpaper_core::{
    Capabilities, Category, Cursor, Http, Item, ListRequest, Page, Provider, ProviderError,
    ProviderInfo, RateLimit, Ratio, Rgb, Size, Sort, Thumb, ThumbCrop,
};

const ID: &str = "wallhaven";
const SEARCH: &str = "https://wallhaven.cc/api/v1/search";
const CATEGORIES: [Category; 3] = [Category::General, Category::Anime, Category::People];

static INFO: ProviderInfo = ProviderInfo {
    id: ID,
    name: "Wallhaven",
    capabilities: Capabilities {
        search: true,
        sorts: &[Sort::Random, Sort::Top, Sort::Newest, Sort::Favorites],
        categories: &CATEGORIES,
        ratios: &[
            Ratio(16, 9),
            Ratio(16, 10),
            Ratio(21, 9),
            Ratio(32, 9),
            Ratio(48, 9),
            Ratio(9, 16),
            Ratio(10, 16),
            Ratio(9, 18),
            Ratio(1, 1),
            Ratio(3, 2),
            Ratio(4, 3),
            Ratio(5, 4),
        ],
        // A subset of the 29 colours the API accepts, in hue order.
        colors: &[
            Rgb(0xcc, 0x00, 0x00),
            Rgb(0xff, 0x66, 0x00),
            Rgb(0xff, 0xcc, 0x33),
            Rgb(0x66, 0x99, 0x00),
            Rgb(0x33, 0x66, 0x00),
            Rgb(0x66, 0xcc, 0xcc),
            Rgb(0x00, 0x66, 0xcc),
            Rgb(0x33, 0x33, 0x99),
            Rgb(0x66, 0x33, 0x99),
            Rgb(0xea, 0x4c, 0x88),
            Rgb(0x66, 0x33, 0x00),
            Rgb(0x00, 0x00, 0x00),
            Rgb(0x99, 0x99, 0x99),
            Rgb(0xff, 0xff, 0xff),
        ],
        min_size: true,
    },
    // The documented limit is 45 per minute. Stay under it.
    rate: RateLimit {
        requests: 40,
        per: Duration::from_secs(60),
    },
};

pub struct Wallhaven;

impl Provider for Wallhaven {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn list(&self, request: &ListRequest, http: &dyn Http) -> Result<Page, ProviderError> {
        parse(&http.get(&search_url(request))?, request)
    }
}

fn search_url(request: &ListRequest) -> String {
    let wanted = |c| request.categories.is_empty() || request.categories.contains(&c);
    let sorting = match request.sort {
        Sort::Random => "random",
        Sort::Newest => "date_added",
        Sort::Top => "toplist",
        Sort::Favorites => "favorites",
        Sort::Relevance => "relevance",
    };
    let mut query: Vec<(&str, String)> = vec![
        // Always SFW. Without a key the API still returns sketchy images when asked for them.
        ("purity", "100".into()),
        (
            "categories",
            CATEGORIES
                .map(|c| if wanted(c) { '1' } else { '0' })
                .iter()
                .collect(),
        ),
        ("sorting", sorting.into()),
    ];
    let text = request.text.trim();
    if !text.is_empty() {
        query.push(("q", text.into()));
    }
    if let Some(size) = request.min_size {
        query.push(("atleast", format!("{}x{}", size.width, size.height)));
    }
    if !request.ratios.is_empty() {
        let all: Vec<_> = request
            .ratios
            .iter()
            .map(|r| format!("{}x{}", r.0, r.1))
            .collect();
        query.push(("ratios", all.join(",")));
    }
    if let Some(Rgb(r, g, b)) = request.color {
        query.push(("colors", format!("{r:02x}{g:02x}{b:02x}")));
    }
    if let Some(Cursor(page)) = &request.cursor {
        query.push(("page", page.clone()));
    }
    let pairs: Vec<_> = query
        .iter()
        .map(|(key, value)| format!("{key}={}", encode(value)))
        .collect();
    format!("{SEARCH}?{}", pairs.join("&"))
}

fn encode(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b',' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[derive(Deserialize)]
struct Answer {
    data: Vec<Wallpaper>,
    meta: Meta,
}

#[derive(Deserialize)]
struct Meta {
    current_page: u32,
    last_page: u32,
    total: Option<u64>,
}

#[derive(Deserialize)]
struct Wallpaper {
    id: String,
    url: String,
    purity: String,
    dimension_x: u32,
    dimension_y: u32,
    file_size: u64,
    file_type: String,
    #[serde(deserialize_with = "colors")]
    colors: Vec<Rgb>,
    path: String,
    thumbs: Thumbs,
}

#[derive(Deserialize)]
struct Thumbs {
    large: String,
    original: String,
}

/// Colours are decoration, so an unreadable one is skipped instead of failing the page.
fn colors<'de, D: Deserializer<'de>>(from: D) -> Result<Vec<Rgb>, D::Error> {
    Ok(Vec::<String>::deserialize(from)?
        .iter()
        .filter_map(|c| Rgb::from_hex(c))
        .collect())
}

fn parse(bytes: &[u8], request: &ListRequest) -> Result<Page, ProviderError> {
    let answer: Answer =
        serde_json::from_slice(bytes).map_err(|e| ProviderError::BadResponse(e.to_string()))?;
    let meta = answer.meta;
    let next = match request.sort {
        // `seed` does not repeat a random listing, so every random page is simply a new draw.
        Sort::Random => (meta.last_page > 1).then(|| Cursor("1".into())),
        _ => (meta.current_page < meta.last_page)
            .then(|| Cursor((meta.current_page + 1).to_string())),
    };
    let items = answer
        .data
        .into_iter()
        .filter(|w| w.purity == "sfw")
        .map(item)
        .collect();
    Ok(Page {
        items,
        next,
        total: meta.total,
    })
}

fn item(w: Wallpaper) -> Item {
    let size = Size::new(w.dimension_x, w.dimension_y);
    // The whole thumbnail fits in a 300 pixel box; the large one is always cropped to 432x243.
    let shrink = 300.0 / size.width.max(size.height).max(1) as f32;
    let scaled = |v: u32| ((v as f32 * shrink).round() as u32).max(1);
    let large = Size::new(432, 243);
    let uncut = (size.aspect() / large.aspect()).ln().abs() < 0.01;
    Item {
        provider: ID.into(),
        id: w.id,
        size,
        image_url: w.path,
        page_url: w.url,
        file_bytes: w.file_size,
        mime: w.file_type,
        colors: w.colors,
        thumbs: vec![
            Thumb {
                url: w.thumbs.large,
                size: large,
                crop: if uncut {
                    ThumbCrop::Whole
                } else {
                    ThumbCrop::Cropped
                },
            },
            Thumb {
                url: w.thumbs.original,
                size: Size::new(scaled(size.width), scaled(size.height)),
                crop: ThumbCrop::Whole,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &[u8] = include_bytes!("../tests/search.json");

    struct Canned(&'static [u8]);

    impl Http for Canned {
        fn get(&self, _: &str) -> Result<Vec<u8>, ProviderError> {
            Ok(self.0.to_vec())
        }
    }

    #[test]
    fn builds_a_fully_filtered_url() {
        let request = ListRequest {
            text: " núi & biển ".into(),
            sort: Sort::Top,
            categories: vec![Category::General, Category::People],
            min_size: Some(Size::new(2560, 1440)),
            ratios: vec![Ratio(16, 9), Ratio(21, 9)],
            color: Some(Rgb(0, 0x66, 0xcc)),
            cursor: Some(Cursor("3".into())),
        };
        assert_eq!(
            search_url(&request),
            "https://wallhaven.cc/api/v1/search?purity=100&categories=101&sorting=toplist\
             &q=n%C3%BAi%20%26%20bi%E1%BB%83n&atleast=2560x1440&ratios=16x9,21x9\
             &colors=0066cc&page=3"
        );
    }

    #[test]
    fn default_request_asks_for_every_category_and_only_sfw() {
        let expected = format!("{SEARCH}?purity=100&categories=111&sorting=random");
        assert_eq!(search_url(&ListRequest::default()), expected);
    }

    #[test]
    fn maps_items_and_drops_anything_not_sfw() {
        let request = ListRequest {
            sort: Sort::Top,
            ..Default::default()
        };
        let page = Wallhaven.list(&request, &Canned(FIXTURE)).unwrap();
        assert_eq!(page.items.len(), 2, "the sketchy entry must be dropped");
        let first = &page.items[0];
        assert_eq!(first.size, Size::new(2560, 1440));
        assert_eq!(first.colors[0], Rgb(0x42, 0x41, 0x53));
        assert_eq!(first.thumbs[0].crop, ThumbCrop::Whole);
        let portrait = &page.items[1];
        assert_eq!(portrait.thumbs[0].crop, ThumbCrop::Cropped);
        assert_eq!(portrait.thumbs[1].size, Size::new(169, 300));
        assert_eq!(page.next, Some(Cursor("2".into())));
    }

    #[test]
    fn random_listing_always_draws_a_new_first_page() {
        let page = Wallhaven
            .list(&ListRequest::default(), &Canned(FIXTURE))
            .unwrap();
        assert_eq!(page.next, Some(Cursor("1".into())));
    }

    #[test]
    fn unreadable_answer_is_an_error() {
        let result = Wallhaven.list(&ListRequest::default(), &Canned(b"<html>"));
        assert!(matches!(result, Err(ProviderError::BadResponse(_))));
    }
}
