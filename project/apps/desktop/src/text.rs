//! Vietnamese wording for values that come from providers.

use wallpaper_core::{Category, ProviderError, Rgb, Size, Sort, fit};

pub fn sort(sort: Sort) -> &'static str {
    match sort {
        Sort::Random => "Ngẫu nhiên",
        Sort::Newest => "Mới nhất",
        Sort::Top => "Nổi bật",
        Sort::Favorites => "Yêu thích nhất",
        Sort::Relevance => "Liên quan",
    }
}

pub fn category(category: Category) -> &'static str {
    match category {
        Category::General => "Tổng hợp",
        Category::Anime => "Anime",
        Category::People => "Con người",
    }
}

pub fn shade(Rgb(r, g, b): Rgb) -> String {
    format!("Màu #{r:02x}{g:02x}{b:02x}")
}

pub fn error(error: &ProviderError) -> String {
    match error {
        ProviderError::Network(_) => "Không kết nối được. Kiểm tra mạng rồi thử lại.".into(),
        ProviderError::RateLimited {
            retry_after: Some(wait),
        } => {
            format!(
                "Nguồn ảnh đang giới hạn lượt tải. Thử lại sau {} giây.",
                wait.as_secs()
            )
        }
        ProviderError::RateLimited { retry_after: None } => {
            "Nguồn ảnh đang giới hạn lượt tải. Thử lại sau khoảng một phút.".into()
        }
        ProviderError::Status(code) => format!("Nguồn ảnh báo lỗi {code}."),
        ProviderError::BadResponse(_) => "Nguồn ảnh trả về dữ liệu không đọc được.".into(),
    }
}

pub struct FitText {
    pub title: String,
    pub note: String,
    pub good: bool,
}

/// Thresholds from the research report: at most 12% cropped, and never enlarged.
pub fn fit_text(image: Size, screen: Size) -> FitText {
    let result = fit(image, screen);
    let cropped = (result.crop_loss * 100.0).round() as u32;
    let note = match cropped {
        0 => "Không bị cắt".to_owned(),
        n => format!("Cắt bớt {n}% ảnh"),
    };
    let screen = format!("{} × {}", screen.width, screen.height);
    let (title, good) = match (result.scale > 1.0, result.crop_loss > 0.12) {
        (true, _) => (format!("Nhỏ hơn màn hình {screen}"), false),
        (false, true) => (format!("Khác tỉ lệ màn hình {screen}"), false),
        (false, false) => (format!("Vừa màn hình {screen}"), true),
    };
    let note = if result.scale > 1.0 {
        format!(
            "{note}, phải phóng to {}%",
            ((result.scale - 1.0) * 100.0).round().max(1.0)
        )
    } else {
        note
    };
    FitText { title, note, good }
}

pub fn file(mime: &str, bytes: u64) -> String {
    let kind = mime.rsplit('/').next().unwrap_or(mime).to_uppercase();
    let size = if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1e6).replace('.', ",")
    } else {
        format!("{} KB", bytes.div_ceil(1000))
    };
    format!("{kind}, {size}")
}

pub fn thousands(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_how_an_image_fits() {
        let screen = Size::new(2560, 1440);
        let exact = fit_text(Size::new(3840, 2160), screen);
        assert!(exact.good);
        assert_eq!(exact.note, "Không bị cắt");
        let small = fit_text(Size::new(1920, 1080), screen);
        assert!(!small.good);
        assert_eq!(small.note, "Không bị cắt, phải phóng to 33%");
        let portrait = fit_text(Size::new(2160, 3840), screen);
        assert!(!portrait.good);
        assert!(portrait.title.starts_with("Nhỏ hơn"));
        let tall = fit_text(Size::new(4320, 7680), screen);
        assert!(tall.title.starts_with("Khác tỉ lệ"));
        assert_eq!(tall.note, "Cắt bớt 68% ảnh");
    }

    #[test]
    fn formats_numbers_the_vietnamese_way() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(51650), "51.650");
        assert_eq!(thousands(1234567), "1.234.567");
        assert_eq!(file("image/jpeg", 8_540_000), "JPEG, 8,5 MB");
        assert_eq!(file("image/png", 385_113), "PNG, 386 KB");
    }
}
