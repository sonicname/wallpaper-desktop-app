use std::{
    cell::{Cell as StdCell, RefCell},
    collections::VecDeque,
    rc::Rc,
    sync::Arc,
};

use slint::{
    Color, Image, Model, ModelNotify, ModelRc, ModelTracker, Rgb8Pixel, SharedPixelBuffer, VecModel,
};
use wallpaper_core::{Item, Rgb, Size};
use wallpaper_host::Pixels;

use crate::{
    Cell, Row,
    layout::{self, Metrics},
    loader::{Job, Loader},
};

pub const GAP: f32 = 12.0;
pub const ROW_HEIGHT: f32 = 220.0;
/// Must match `Theme.radius` and `Theme.background`.
const RADIUS: f32 = 10.0;
pub const BACKGROUND: Rgb = Rgb(0x0f, 0x10, 0x13);
/// Rows kept decoded outside the visible area.
const SPARE_ROWS: usize = 8;

thread_local! {
    static CURRENT: RefCell<std::rc::Weak<GridModel>> = const { RefCell::new(std::rc::Weak::new()) };
}

struct CachedRow {
    row: usize,
    cells: Rc<VecModel<Cell>>,
    /// Dropping this cancels the thumbnails that were not loaded yet.
    wanted: Arc<()>,
}

/// Rows of the grid. Pictures are requested when a row becomes visible and dropped later.
pub struct GridModel {
    loader: Loader,
    items: RefCell<Vec<Item>>,
    rows: RefCell<Vec<layout::Row>>,
    placed: StdCell<usize>,
    metrics: StdCell<Metrics>,
    scale: StdCell<f32>,
    keep: StdCell<usize>,
    /// Pictures that arrive for an older layout are ignored.
    epoch: StdCell<u64>,
    cache: RefCell<VecDeque<CachedRow>>,
    notify: ModelNotify,
    near_end: Box<dyn Fn()>,
    is_favorite: Box<dyn Fn(&Item) -> bool>,
}

impl GridModel {
    pub fn new(
        loader: Loader,
        near_end: impl Fn() + 'static,
        is_favorite: impl Fn(&Item) -> bool + 'static,
    ) -> Rc<Self> {
        let model = Rc::new(Self {
            loader,
            items: RefCell::default(),
            rows: RefCell::default(),
            placed: StdCell::new(0),
            metrics: StdCell::new(Metrics {
                width: 0,
                gap: 0,
                target_height: 1,
            }),
            scale: StdCell::new(1.0),
            keep: StdCell::new(SPARE_ROWS),
            epoch: StdCell::new(0),
            cache: RefCell::default(),
            notify: ModelNotify::default(),
            near_end: Box::new(near_end),
            is_favorite: Box::new(is_favorite),
        });
        CURRENT.with(|c| *c.borrow_mut() = Rc::downgrade(&model));
        model
    }

    pub fn len(&self) -> usize {
        self.items.borrow().len()
    }

    pub fn item(&self, index: usize) -> Option<Item> {
        self.items.borrow().get(index).cloned()
    }

    /// Applies a new logical size. Returns whether anything moved.
    pub fn resize(&self, width: f32, height: f32, scale: f32, finished: bool) -> bool {
        let px = |v: f32| (v * scale).round().max(0.0) as u32;
        let metrics = Metrics {
            width: px(width),
            gap: px(GAP),
            target_height: px(ROW_HEIGHT),
        };
        self.keep
            .set((height / (ROW_HEIGHT * 0.6)).ceil().max(1.0) as usize + SPARE_ROWS);
        if metrics == self.metrics.get() && scale == self.scale.get() {
            return false;
        }
        self.metrics.set(metrics);
        self.scale.set(scale);
        self.rebuild(finished);
        true
    }

    /// Shows or hides the heart on a picture.
    pub fn mark(&self, index: usize, favorite: bool) {
        for row in self.cache.borrow().iter() {
            let cells = row.cells.iter().enumerate();
            let found = cells.filter(|(_, cell)| cell.index == index as i32);
            for (position, mut cell) in found.collect::<Vec<_>>() {
                cell.favorite = favorite;
                row.cells.set_row_data(position, cell);
            }
        }
    }

    /// Gives back the memory of all pictures. They are loaded again when shown.
    pub fn release(&self) {
        self.epoch.set(self.epoch.get() + 1);
        self.cache.borrow_mut().clear();
        self.cache.borrow_mut().shrink_to_fit();
        self.notify.reset();
    }

    pub fn clear(&self) {
        self.items.borrow_mut().clear();
        self.rebuild(false);
    }

    /// Adds a page. `finished` places the last row even if it is not full.
    pub fn append(&self, items: Vec<Item>, finished: bool) {
        self.items.borrow_mut().extend(items);
        let before = self.rows.borrow().len();
        self.place(finished);
        let added = self.rows.borrow().len() - before;
        if added > 0 {
            self.notify.row_added(before, added);
        }
    }

    fn rebuild(&self, finished: bool) {
        self.epoch.set(self.epoch.get() + 1);
        self.cache.borrow_mut().clear();
        self.rows.borrow_mut().clear();
        self.placed.set(0);
        self.place(finished);
        self.notify.reset();
    }

    fn place(&self, finished: bool) {
        if self.metrics.get().width == 0 {
            return;
        }
        let aspects: Vec<f32> = self
            .items
            .borrow()
            .iter()
            .map(|i| i.size.aspect())
            .collect();
        let (rows, placed) =
            layout::justify(&aspects, self.placed.get(), self.metrics.get(), finished);
        self.rows.borrow_mut().extend(rows);
        self.placed.set(placed);
    }

    fn deliver(&self, epoch: u64, row: usize, cell: usize, pixels: Pixels) {
        if epoch != self.epoch.get() {
            return;
        }
        let cells = self
            .cache
            .borrow()
            .iter()
            .find(|c| c.row == row)
            .map(|c| c.cells.clone());
        if let Some(cells) = cells
            && let Some(mut data) = cells.row_data(cell)
        {
            data.image = to_image(&pixels);
            data.ready = true;
            cells.set_row_data(cell, data);
        }
    }

    fn build_row(&self, row: usize) -> Option<(f32, Rc<VecModel<Cell>>)> {
        let rows = self.rows.borrow();
        let layout = rows.get(row)?;
        let height = layout.height as f32 / self.scale.get();
        let mut cache = self.cache.borrow_mut();
        if let Some(found) = cache.iter().find(|c| c.row == row) {
            return Some((height, found.cells.clone()));
        }
        let items = self.items.borrow();
        let wanted = Arc::new(());
        let cells = Rc::new(VecModel::default());
        for (position, placed) in layout.cells.iter().enumerate() {
            let item = &items[placed.index];
            let target = Size::new(placed.width, layout.height);
            cells.push(Cell {
                index: placed.index as i32,
                x: placed.x as f32 / self.scale.get(),
                width: placed.width as f32 / self.scale.get(),
                image: Image::default(),
                ready: false,
                tint: tint(item),
                label: size_label(item.size).into(),
                favorite: (self.is_favorite)(item),
            });
            let Some(thumb) = item.best_thumb(target) else {
                continue;
            };
            let epoch = self.epoch.get();
            self.loader.push(Job {
                wanted: Arc::downgrade(&wanted),
                url: thumb.url.clone(),
                target,
                radius: RADIUS * self.scale.get(),
                background: BACKGROUND,
                done: Box::new(move |pixels| {
                    // Fails only while the app is closing.
                    let _ = slint::invoke_from_event_loop(move || {
                        let model = CURRENT.with(|c| c.borrow().upgrade());
                        if let Some(model) = model {
                            model.deliver(epoch, row, position, pixels);
                        }
                    });
                }),
            });
        }
        cache.push_back(CachedRow {
            row,
            cells: cells.clone(),
            wanted,
        });
        while cache.len() > self.keep.get() {
            // The row furthest from the one being shown is the least likely to be needed.
            let furthest = (0..cache.len())
                .max_by_key(|i| cache[*i].row.abs_diff(row))
                .unwrap_or(0);
            if let Some(dropped) = cache.remove(furthest) {
                drop(dropped.wanted);
            }
        }
        Some((height, cells))
    }
}

impl Model for GridModel {
    type Data = Row;

    fn row_count(&self) -> usize {
        self.rows.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Row> {
        let (height, cells) = self.build_row(row)?;
        if row + 4 >= self.row_count() {
            (self.near_end)();
        }
        Some(Row {
            height,
            cells: ModelRc::from(cells),
        })
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn to_image(pixels: &Pixels) -> Image {
    Image::from_rgb8(SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
        &pixels.rgb,
        pixels.size.width,
        pixels.size.height,
    ))
}

pub fn color(rgb: Rgb) -> Color {
    Color::from_rgb_u8(rgb.0, rgb.1, rgb.2)
}

/// A dimmed version of the main colour, shown until the picture arrives.
pub fn tint(item: &Item) -> Color {
    let Rgb(r, g, b) = item
        .colors
        .first()
        .copied()
        .unwrap_or(Rgb(0x40, 0x40, 0x48));
    let dim = |v: u8, base: u8| ((v as u16 * 2 + base as u16 * 3) / 5) as u8;
    Color::from_rgb_u8(dim(r, 0x19), dim(g, 0x1b), dim(b, 0x20))
}

pub fn size_label(size: Size) -> String {
    format!("{} × {}", size.width, size.height)
}
