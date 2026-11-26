//! The part of the controller for several sources: switching between them, favorites, and the
//! folder on this computer.

use std::{io, path::Path, sync::Arc, thread};

use slint::SharedString;
use wallpaper_core::{Item, ListRequest, Provider, Sort, ThumbCrop};
use wallpaper_host::{HostHttp, dimensions};

use super::{Controller, model, with_current};
use crate::{Option as UiOption, loader::Loader};

/// A provider with what the app keeps for it.
pub(super) struct Source {
    pub provider: Arc<dyn Provider>,
    /// Calls to the provider are counted against its own limit.
    pub http: Arc<HostHttp>,
    /// Pictures the user already has. Nothing is filtered away before they ask for it.
    pub own: bool,
}

impl Source {
    pub fn new(provider: Arc<dyn Provider>, own: bool) -> Self {
        let http = Arc::new(HostHttp::new(provider.info().rate));
        Self {
            provider,
            http,
            own,
        }
    }

    /// What the source shows first: its best pictures if it ranks them, else its own first order.
    pub fn first_request(&self) -> ListRequest {
        let sorts = self.provider.info().capabilities.sorts;
        let best = sorts.iter().find(|s| **s == Sort::Top);
        ListRequest {
            sort: best.or(sorts.first()).copied().unwrap_or(Sort::Random),
            ..Default::default()
        }
    }
}

impl Controller {
    pub(super) fn connect_library(&self) {
        let window = &self.window;
        window.set_can_favorite(self.favorites.is_some());
        window.on_pick_source(|i| with_current(|c| c.pick_source(i as usize)));
        window.on_pick_folder(|| with_current(Controller::pick_folder));
        window.on_toggle_favorite(|| with_current(Controller::toggle_favorite));
        self.show_sources();
    }

    pub(super) fn source(&self) -> &Source {
        &self.sources[self.current.get()]
    }

    pub(super) fn provider(&self) -> &Arc<dyn Provider> {
        &self.source().provider
    }

    fn shows(&self, id: &str) -> bool {
        self.provider().info().id == id
    }

    /// The folder source is open, but no folder was chosen yet.
    pub(super) fn needs_folder(&self) -> bool {
        self.shows(wallpaper_provider_folder::ID)
            && self.folder.as_ref().is_some_and(|f| f.path().is_none())
    }

    /// Why nothing is shown, when the reason is not the filters.
    pub(super) fn empty_reason(&self) -> Option<&'static str> {
        let favorites = self.favorites.as_ref()?;
        (self.shows(wallpaper_favorites::ID) && favorites.is_empty())
            .then_some("Chưa có ảnh yêu thích. Mở một ảnh rồi bấm hình trái tim để giữ lại ở đây.")
    }

    /// The name of the source a picture first came from.
    pub(super) fn source_name(&self, item: &Item) -> String {
        let from = |s: &&Source| s.provider.info().id == item.provider;
        let known = self.sources.iter().find(from);
        known.map_or(item.provider.clone(), |s| s.provider.info().name.to_owned())
    }

    fn pick_source(&self, index: usize) {
        if index >= self.sources.len() || index == self.current.get() {
            return;
        }
        let text = {
            let mut listing = self.listing.borrow_mut();
            let mut saved = self.saved.borrow_mut();
            saved[self.current.get()] = (listing.request.clone(), listing.fit_screen);
            (listing.request, listing.fit_screen) = saved[index].clone();
            listing.request.text.clone()
        };
        self.current.set(index);
        self.window.set_search_text(text.into());
        self.window.set_detail_open(false);
        self.show_sources();
        self.reload();
    }

    pub(super) fn show_sources(&self) {
        let current = self.current.get();
        let names = self.sources.iter().enumerate().map(|(i, source)| UiOption {
            label: source.provider.info().name.into(),
            on: i == current,
        });
        self.window.set_sources(model(names));
        let folder = self.shows(wallpaper_provider_folder::ID);
        self.window
            .set_can_search(self.provider().info().capabilities.search);
        self.window.set_search_hint(if folder {
            "Tìm theo tên tệp".into()
        } else {
            "Tìm hình nền, ví dụ: mountain".into()
        });
        let chosen = self
            .folder
            .as_ref()
            .and_then(|f| f.path())
            .filter(|_| folder);
        let label = chosen.map(|path| {
            let name = path.file_name().unwrap_or(path.as_os_str());
            format!("Thư mục: {}", name.to_string_lossy())
        });
        self.window
            .set_folder_label(label.unwrap_or_default().into());
        self.window.set_needs_folder(self.needs_folder());
    }

    fn pick_folder(&self) {
        let Some(folder) = self.folder.clone() else {
            return;
        };
        // The dialog blocks, so it runs on a thread of its own, once at a time.
        if self.picking.replace(true) {
            return;
        }
        let system = self.system.clone();
        thread::spawn(move || {
            let saved = system.pick_folder().map(|path| folder.set(path));
            let _ = slint::invoke_from_event_loop(move || {
                with_current(|c| {
                    c.picking.set(false);
                    c.folder_picked(saved);
                })
            });
        });
    }

    fn folder_picked(&self, saved: Option<io::Result<()>>) {
        match saved {
            None => {}
            Some(Err(_)) => self.show_toast("Không lưu được thư mục đã chọn.".into()),
            Some(Ok(())) => {
                self.show_sources();
                if self.shows(wallpaper_provider_folder::ID) {
                    self.reload();
                }
                self.source_changed(wallpaper_provider_folder::ID);
            }
        }
    }

    fn toggle_favorite(&self) {
        let index = self.selected.get();
        let (Some(favorites), Some(item)) = (self.favorites.clone(), self.grid.item(index)) else {
            return;
        };
        if favorites.contains(&item) {
            let result = favorites.remove(&item);
            return self.favorite_changed(index, &item, result);
        }
        let loader = self.loader.share();
        thread::spawn(move || {
            let result = favorites.add(item.clone(), thumb_copy(&item, &loader));
            let _ = slint::invoke_from_event_loop(move || {
                with_current(|c| c.favorite_changed(index, &item, result))
            });
        });
    }

    fn favorite_changed(&self, index: usize, item: &Item, result: io::Result<()>) {
        if result.is_err() {
            self.show_toast(
                "Không lưu được danh sách yêu thích. Kiểm tra dung lượng trống.".into(),
            );
        }
        let now = self.is_favorite(item);
        // The user may have moved on while the thumbnail was copied.
        let same = |other: Item| other.key() == item.key();
        if self.grid.item(index).is_some_and(same) {
            self.grid.mark(index, now);
            if self.selected.get() == index {
                let mut detail = self.window.get_detail();
                detail.favorite = now;
                self.window.set_detail(detail);
            }
        }
        self.source_changed(wallpaper_favorites::ID);
    }

    pub(super) fn is_favorite(&self, item: &Item) -> bool {
        self.favorites.as_ref().is_some_and(|f| f.contains(item))
    }

    /// What the button under a picture opens, if anything.
    pub(super) fn page_label(item: &Item) -> &'static str {
        if item.page_url.starts_with("https://") {
            "Mở trang gốc"
        } else if item.local_file().is_some() {
            "Mở thư mục chứa ảnh"
        } else {
            ""
        }
    }

    pub(super) fn open_page(item: &Item) {
        match item.local_file() {
            Some(file) => drop(wallpaper_platform_windows::reveal(file)),
            None => drop(wallpaper_platform_windows::open_link(&item.page_url)),
        }
    }

    /// How a picture is named next to its source.
    pub(super) fn short_id(item: &Item) -> SharedString {
        let name = item.local_file().and_then(Path::file_name);
        name.map_or(item.id.as_str().into(), |n| {
            n.to_string_lossy().as_ref().into()
        })
    }
}

/// The thumbnail to keep with a favorite: the whole picture if there is one, as large as possible.
fn thumb_copy(item: &Item, loader: &Loader) -> Option<(Vec<u8>, wallpaper_core::Size, ThumbCrop)> {
    let area = |t: &&wallpaper_core::Thumb| t.size.width * t.size.height;
    let whole = item.thumbs.iter().filter(|t| t.crop == ThumbCrop::Whole);
    let thumb = whole
        .max_by_key(area)
        .or(item.thumbs.iter().max_by_key(area))?;
    let bytes = loader.thumb(&thumb.url)?;
    let size = dimensions(&bytes)?;
    Some((bytes, size, thumb.crop))
}
