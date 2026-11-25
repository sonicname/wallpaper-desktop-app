//! Drives the real window against a scripted source. Slint allows one platform per process,
//! so the steps share a single test.

#[path = "../examples/support/stage.rs"]
mod stage;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use slint::{ComponentHandle, Model};
use stage::Stage;
use wallpaper_core::{
    Capabilities, Category, Cursor, Http, Item, ListRequest, Page, Provider, ProviderError,
    ProviderInfo, RateLimit, Ratio, Rgb, Size, Sort,
};
use wallpaper_desktop::{App, AppWindow, Config};

static INFO: ProviderInfo = ProviderInfo {
    id: "fake",
    name: "Fake",
    capabilities: Capabilities {
        search: true,
        sorts: &[Sort::Random, Sort::Newest],
        categories: &[Category::General, Category::Anime],
        ratios: &[Ratio(16, 9), Ratio(9, 16)],
        colors: &[Rgb(255, 0, 0)],
        min_size: true,
    },
    rate: RateLimit {
        requests: 1000,
        per: Duration::from_secs(1),
    },
};

type Answer = dyn Fn(&ListRequest) -> Result<Page, ProviderError> + Send + Sync;

struct Fake {
    answer: Mutex<Arc<Answer>>,
    asked: Mutex<Vec<ListRequest>>,
}

impl Fake {
    fn answers(
        &self,
        f: impl Fn(&ListRequest) -> Result<Page, ProviderError> + Send + Sync + 'static,
    ) {
        *self.answer.lock().unwrap() = Arc::new(f);
    }

    fn count(&self) -> usize {
        self.asked.lock().unwrap().len()
    }

    fn last(&self) -> ListRequest {
        self.asked.lock().unwrap().last().unwrap().clone()
    }
}

impl Provider for Fake {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn list(&self, request: &ListRequest, _: &dyn Http) -> Result<Page, ProviderError> {
        self.asked.lock().unwrap().push(request.clone());
        let answer = self.answer.lock().unwrap().clone();
        answer(request)
    }
}

fn items(ids: std::ops::Range<u32>) -> Vec<Item> {
    ids.map(|id| Item {
        provider: "fake",
        id: id.to_string(),
        size: Size::new(2560, 1440),
        image_url: String::new(),
        page_url: String::new(),
        file_bytes: 1,
        mime: "image/jpeg".into(),
        colors: vec![],
        thumbs: vec![],
    })
    .collect()
}

fn cells(window: &AppWindow) -> usize {
    window
        .get_rows()
        .iter()
        .map(|row| row.cells.row_count())
        .sum()
}

fn on(options: slint::ModelRc<wallpaper_desktop::Option>) -> Vec<bool> {
    options.iter().map(|o| o.on).collect()
}

#[test]
fn browsing() {
    const WAIT: Duration = Duration::from_secs(5);
    let mut stage = Stage::new(1280, 820, 1.0);
    let fake = Arc::new(Fake {
        answer: Mutex::new(Arc::new(|_| Err(ProviderError::Network("down".into())))),
        asked: Mutex::default(),
    });
    let app = App::new(Config {
        provider: fake.clone(),
        screen: Some(Size::new(2560, 1440)),
        cache_dir: None,
    })
    .unwrap();
    let window = app.window().clone_strong();
    window.show().unwrap();

    // The first request already asks for pictures that fit the screen.
    assert!(
        stage.run_until(WAIT, || window.get_can_retry()),
        "the error must be shown"
    );
    assert!(window.get_message().contains("Không kết nối được"));
    assert!(!window.get_busy());
    assert_eq!(fake.last().min_size, Some(Size::new(2560, 1440)));
    assert_eq!(fake.last().ratios, vec![Ratio(16, 9)]);
    let after_error = fake.count();
    stage.run_for(Duration::from_millis(200));
    assert_eq!(
        fake.count(),
        after_error,
        "an error must not be retried on its own"
    );

    // Retry works, and a random listing that only repeats itself ends.
    fake.answers(|_| {
        Ok(Page {
            items: items(0..7),
            next: Some(Cursor("1".into())),
            total: Some(7),
        })
    });
    window.invoke_retry();
    assert!(
        stage.run_until(WAIT, || cells(&window) == 7),
        "{} cells",
        cells(&window)
    );
    stage.run_for(Duration::from_millis(300));
    assert_eq!(cells(&window), 7, "repeated pictures must not be added");
    assert!(!window.get_busy());
    assert_eq!(window.get_message(), "");
    let settled = fake.count();
    stage.run_for(Duration::from_millis(300));
    assert_eq!(
        fake.count(),
        settled,
        "a listing without news must stop loading"
    );

    // Paging follows the cursor of the source and stops at its end.
    fake.answers(|request| {
        let page: u32 = request.cursor.as_ref().map_or(1, |c| c.0.parse().unwrap());
        let next = (page < 3).then(|| Cursor((page + 1).to_string()));
        Ok(Page {
            items: items(page * 100..page * 100 + 4),
            next,
            total: Some(12),
        })
    });
    window.invoke_pick_sort(1);
    assert!(
        stage.run_until(WAIT, || cells(&window) == 12),
        "{} cells",
        cells(&window)
    );
    assert_eq!(fake.last().sort, Sort::Newest);
    assert_eq!(on(window.get_sorts()), [false, true]);

    // Filters change the request; the last category cannot be turned off.
    window.invoke_toggle_screen_fit();
    stage.run_until(WAIT, || cells(&window) == 12);
    assert_eq!((fake.last().min_size, fake.last().ratios.len()), (None, 0));
    window.invoke_pick_category(0);
    stage.run_until(WAIT, || cells(&window) == 12);
    assert_eq!(fake.last().categories, vec![Category::Anime]);
    let before = fake.count();
    window.invoke_pick_category(1);
    stage.run_for(Duration::from_millis(200));
    assert_eq!(fake.count(), before);
    assert_eq!(on(window.get_categories()), [false, true]);
    window.invoke_pick_category(0);
    stage.run_until(WAIT, || cells(&window) == 12);
    assert!(
        fake.last().categories.is_empty(),
        "every category on means no filter"
    );

    // A slow answer to an old search must not replace the new one.
    fake.answers(|request| {
        if request.text == "slow" {
            std::thread::sleep(Duration::from_millis(400));
            return Ok(Page {
                items: items(0..20),
                next: None,
                total: Some(20),
            });
        }
        Ok(Page {
            items: items(50..53),
            next: None,
            total: Some(3),
        })
    });
    window.invoke_search("slow".into());
    stage.run_for(Duration::from_millis(50));
    window.invoke_search("fast".into());
    stage.run_for(Duration::from_millis(800));
    assert_eq!(cells(&window), 3);
    assert_eq!(window.get_summary(), "3 ảnh");

    // Stepping through the preview stays inside the listing.
    window.invoke_open(2);
    assert!(window.get_detail_open());
    assert_eq!(window.get_detail().resolution, "2560 × 1440");
    assert!(window.get_detail().fit_good);
    window.invoke_step(1);
    assert!(window.get_detail().source.ends_with("52"));
    window.invoke_step(-1);
    assert!(window.get_detail().source.ends_with("51"));

    // No result is explained.
    fake.answers(|_| Ok(Page::default()));
    window.set_detail_open(false);
    window.invoke_search("nothing".into());
    assert!(stage.run_until(WAIT, || window.get_message().contains("Không có ảnh nào")));
    assert!(!window.get_can_retry());
}
