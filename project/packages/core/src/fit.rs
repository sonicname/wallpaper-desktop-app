use crate::Size;

/// A named aspect ratio such as 16x9.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio(pub u32, pub u32);

impl Ratio {
    /// Shapes of common screens, for providers that have no list of their own.
    pub const COMMON: &'static [Ratio] = &[
        Ratio(16, 9),
        Ratio(16, 10),
        Ratio(21, 9),
        Ratio(32, 9),
        Ratio(9, 16),
        Ratio(10, 16),
        Ratio(3, 2),
        Ratio(4, 3),
        Ratio(5, 4),
        Ratio(1, 1),
    ];

    pub fn value(self) -> f32 {
        self.0 as f32 / self.1 as f32
    }

    /// The entry of `known` whose shape is closest to `size`.
    pub fn nearest(size: Size, known: &[Ratio]) -> Option<Ratio> {
        let distance = |r: &Ratio| (r.value() / size.aspect()).ln().abs();
        known
            .iter()
            .copied()
            .min_by(|a, b| distance(a).total_cmp(&distance(b)))
    }
}

/// How an image fills a screen when scaled to cover it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    /// Above 1.0 the image has to be enlarged and loses sharpness.
    pub scale: f32,
    /// Share of the image cut away, from 0.0 to 1.0.
    pub crop_loss: f32,
}

pub fn fit(image: Size, screen: Size) -> Fit {
    let scale = (screen.width as f32 / image.width.max(1) as f32)
        .max(screen.height as f32 / image.height.max(1) as f32);
    let (i, s) = (image.aspect(), screen.aspect());
    Fit {
        scale,
        crop_loss: 1.0 - (s / i).min(i / s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOWN: &[Ratio] = &[
        Ratio(16, 9),
        Ratio(16, 10),
        Ratio(21, 9),
        Ratio(32, 9),
        Ratio(9, 16),
        Ratio(3, 2),
    ];

    #[test]
    fn exact_match_needs_no_crop_or_scale() {
        let f = fit(Size::new(2560, 1440), Size::new(2560, 1440));
        assert_eq!((f.scale, f.crop_loss), (1.0, 0.0));
    }

    #[test]
    fn larger_image_scales_down() {
        assert!(fit(Size::new(3840, 2160), Size::new(2560, 1440)).scale < 1.0);
    }

    #[test]
    fn landscape_on_portrait_screen_loses_most_of_the_image() {
        let f = fit(Size::new(3840, 2160), Size::new(1440, 2560));
        assert!((f.crop_loss - 0.684).abs() < 0.001);
    }

    #[test]
    fn real_screens_map_to_named_ratios() {
        let nearest = |w, h| Ratio::nearest(Size::new(w, h), KNOWN).unwrap();
        assert_eq!(nearest(2560, 1440), Ratio(16, 9));
        assert_eq!(nearest(1366, 768), Ratio(16, 9));
        assert_eq!(nearest(3440, 1440), Ratio(21, 9));
        assert_eq!(nearest(5120, 1440), Ratio(32, 9));
        assert_eq!(nearest(2880, 1920), Ratio(3, 2));
        assert_eq!(nearest(1440, 2560), Ratio(9, 16));
    }
}
