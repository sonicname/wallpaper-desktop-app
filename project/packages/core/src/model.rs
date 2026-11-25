#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn aspect(self) -> f32 {
        self.width.max(1) as f32 / self.height.max(1) as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// Parses `#rrggbb` or `rrggbb`.
    pub fn from_hex(text: &str) -> Option<Self> {
        let hex = text.strip_prefix('#').unwrap_or(text);
        if hex.len() != 6 || !hex.is_ascii() {
            return None;
        }
        let byte = |at: usize| u8::from_str_radix(&hex[at..at + 2], 16).ok();
        Some(Self(byte(0)?, byte(2)?, byte(4)?))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    Random,
    Newest,
    Top,
    Favorites,
    Relevance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    General,
    Anime,
    People,
}

/// How a thumbnail relates to the full image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThumbCrop {
    Whole,
    Cropped,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thumb {
    pub url: String,
    pub size: Size,
    pub crop: ThumbCrop,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub provider: &'static str,
    pub id: String,
    pub size: Size,
    pub image_url: String,
    pub page_url: String,
    pub file_bytes: u64,
    pub mime: String,
    pub colors: Vec<Rgb>,
    pub thumbs: Vec<Thumb>,
}

impl Item {
    /// Picks a thumbnail for a cell of `target` pixels. A cropped thumbnail is only used when
    /// its shape is close to the cell, so that portrait images are not cut twice.
    pub fn best_thumb(&self, target: Size) -> Option<&Thumb> {
        let usable = |t: &&Thumb| {
            t.crop == ThumbCrop::Whole || (t.size.aspect() / target.aspect()).ln().abs() < 0.4
        };
        let candidates = || self.thumbs.iter().filter(usable);
        candidates()
            .filter(|t| t.size.width >= target.width && t.size.height >= target.height)
            .min_by_key(|t| t.size.width * t.size.height)
            .or_else(|| candidates().max_by_key(|t| t.size.width * t.size.height))
    }
}

/// Opaque position in a listing. Only the provider that produced it can read it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cursor(pub String);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Page {
    pub items: Vec<Item>,
    pub next: Option<Cursor>,
    pub total: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(size: Size) -> Item {
        let thumb = |url: &str, w, h, crop| Thumb {
            url: url.into(),
            size: Size::new(w, h),
            crop,
        };
        Item {
            provider: "test",
            id: "1".into(),
            size,
            image_url: String::new(),
            page_url: String::new(),
            file_bytes: 0,
            mime: String::new(),
            colors: vec![],
            thumbs: vec![
                thumb("large", 432, 243, ThumbCrop::Cropped),
                thumb("whole", 169, 300, ThumbCrop::Whole),
            ],
        }
    }

    #[test]
    fn portrait_cell_uses_the_whole_thumbnail() {
        let found = item(Size::new(2160, 3840));
        assert_eq!(found.best_thumb(Size::new(124, 220)).unwrap().url, "whole");
    }

    #[test]
    fn wide_cell_uses_the_larger_cropped_thumbnail() {
        let found = item(Size::new(3840, 2160));
        assert_eq!(found.best_thumb(Size::new(390, 220)).unwrap().url, "large");
    }

    #[test]
    fn oversized_cell_falls_back_to_the_largest_thumbnail() {
        let found = item(Size::new(3840, 2160));
        assert_eq!(found.best_thumb(Size::new(800, 450)).unwrap().url, "large");
    }

    #[test]
    fn parses_hex_colours() {
        assert_eq!(Rgb::from_hex("#0a0B0c"), Some(Rgb(10, 11, 12)));
        assert_eq!(Rgb::from_hex("#12345"), None);
        assert_eq!(Rgb::from_hex("zzzzzz"), None);
        assert_eq!(Rgb::from_hex("ééé"), None);
    }
}
