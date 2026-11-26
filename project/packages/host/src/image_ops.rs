use std::{fmt, io::Cursor};

use fast_image_resize::{PixelType, ResizeOptions, Resizer, images::Image};
use image::{ExtendedColorType, ImageEncoder, ImageReader, Limits, codecs::jpeg::JpegEncoder};
use wallpaper_core::{Rgb, Size};

/// Tightly packed 8-bit RGB.
pub struct Pixels {
    pub size: Size,
    pub rgb: Vec<u8>,
}

#[derive(Debug)]
pub struct ImageError(String);

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ImageError {}

fn fail(error: impl fmt::Display) -> ImageError {
    ImageError(error.to_string())
}

/// Decodes `bytes`, scales it to cover `target` around the centre and rounds its corners
/// against `background`. Drawing the result needs neither scaling nor blending.
pub fn cover(
    bytes: &[u8],
    target: Size,
    radius: f32,
    background: Rgb,
) -> Result<Pixels, ImageError> {
    if target.width == 0 || target.height == 0 {
        return Err(fail("empty target"));
    }
    let source = decode(bytes)?;
    let mut scaled = Image::new(target.width, target.height, PixelType::U8x3);
    let options = ResizeOptions::new().fit_into_destination(Some((0.5, 0.5)));
    Resizer::new()
        .resize(&source, &mut scaled, &options)
        .map_err(fail)?;
    let mut pixels = Pixels {
        size: target,
        rgb: scaled.into_vec(),
    };
    round_corners(&mut pixels, radius, background);
    Ok(pixels)
}

/// The size of a picture, read from its header.
pub fn dimensions(bytes: &[u8]) -> Option<Size> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let (width, height) = reader.into_dimensions().ok()?;
    Some(Size::new(width, height))
}

/// A JPEG copy that fits into a square of `side` pixels. Never larger than the picture.
pub fn thumbnail(bytes: &[u8], side: u32) -> Result<Vec<u8>, ImageError> {
    let source = decode(bytes)?;
    let shrink = (side as f32 / source.width().max(source.height()) as f32).min(1.0);
    let scaled = |v: u32| ((v as f32 * shrink).round() as u32).max(1);
    let (width, height) = (scaled(source.width()), scaled(source.height()));
    let mut small = Image::new(width, height, PixelType::U8x3);
    Resizer::new()
        .resize(&source, &mut small, None)
        .map_err(fail)?;
    let mut jpeg = vec![];
    JpegEncoder::new_with_quality(&mut jpeg, 85)
        .write_image(small.buffer(), width, height, ExtendedColorType::Rgb8)
        .map_err(fail)?;
    Ok(jpeg)
}

fn decode(bytes: &[u8]) -> Result<Image<'static>, ImageError> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(fail)?;
    let mut limits = Limits::default();
    limits.max_alloc = Some(512 << 20);
    reader.limits(limits);
    let source = reader.decode().map_err(fail)?.into_rgb8();
    let (width, height) = source.dimensions();
    Image::from_vec_u8(width, height, source.into_raw(), PixelType::U8x3).map_err(fail)
}

fn round_corners(pixels: &mut Pixels, radius: f32, background: Rgb) {
    let (w, h) = (pixels.size.width as usize, pixels.size.height as usize);
    let radius = radius.min(w as f32 / 2.0).min(h as f32 / 2.0);
    let reach = radius.ceil() as usize;
    let background = [background.0, background.1, background.2];
    for y in 0..reach {
        for x in 0..reach {
            let (dx, dy) = (radius - x as f32 - 0.5, radius - y as f32 - 0.5);
            let keep = (radius - dx.max(0.0).hypot(dy.max(0.0)) + 0.5).clamp(0.0, 1.0);
            if keep >= 1.0 {
                continue;
            }
            for (px, py) in [
                (x, y),
                (w - 1 - x, y),
                (x, h - 1 - y),
                (w - 1 - x, h - 1 - y),
            ] {
                let at = (py * w + px) * 3;
                for (channel, bg) in pixels.rgb[at..at + 3].iter_mut().zip(background) {
                    *channel = (*channel as f32 * keep + bg as f32 * (1.0 - keep)).round() as u8;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, RgbImage};

    fn png(width: u32, height: u32, color: impl Fn(u32, u32) -> [u8; 3]) -> Vec<u8> {
        let source = RgbImage::from_fn(width, height, |x, y| image::Rgb(color(x, y)));
        let mut out = Cursor::new(vec![]);
        source.write_to(&mut out, ImageFormat::Png).unwrap();
        out.into_inner()
    }

    #[test]
    fn covers_the_target_by_cropping_the_sides() {
        // Left and right thirds are red, the middle is green.
        let paint = |x, _| {
            if (100..200).contains(&x) {
                [0, 255, 0]
            } else {
                [255, 0, 0]
            }
        };
        let out = cover(&png(300, 100, paint), Size::new(50, 50), 0.0, Rgb(0, 0, 0)).unwrap();
        assert_eq!(out.rgb.len(), 50 * 50 * 3);
        assert_eq!(&out.rgb[(25 * 50 + 1) * 3..][..3], &[0, 255, 0]);
        assert_eq!(&out.rgb[(25 * 50 + 48) * 3..][..3], &[0, 255, 0]);
    }

    #[test]
    fn rounds_only_the_corners() {
        let bytes = png(40, 40, |_, _| [255, 255, 255]);
        let out = cover(&bytes, Size::new(40, 40), 10.0, Rgb(1, 2, 3)).unwrap();
        let at = |x: usize, y: usize| out.rgb[(y * 40 + x) * 3..][..3].to_vec();
        for corner in [at(0, 0), at(39, 0), at(0, 39), at(39, 39)] {
            assert_eq!(corner, [1, 2, 3]);
        }
        assert_eq!(at(20, 0), [255, 255, 255]);
        assert_eq!(at(20, 20), [255, 255, 255]);
    }

    #[test]
    fn thumbnails_keep_the_shape_and_never_grow() {
        let wide = png(400, 100, |x, _| [x as u8, 0, 0]);
        assert_eq!(dimensions(&wide), Some(Size::new(400, 100)));
        let small = thumbnail(&wide, 200).unwrap();
        assert_eq!(dimensions(&small), Some(Size::new(200, 50)));
        let same = thumbnail(&wide, 1000).unwrap();
        assert_eq!(dimensions(&same), Some(Size::new(400, 100)));
        assert!(thumbnail(b"not an image", 100).is_err());
        assert_eq!(dimensions(b"not an image"), None);
    }

    #[test]
    fn rejects_garbage_without_panicking() {
        let black = Rgb(0, 0, 0);
        assert!(cover(b"not an image", Size::new(10, 10), 0.0, black).is_err());
        assert!(cover(&png(4, 4, |_, _| [0; 3]), Size::new(0, 10), 0.0, black).is_err());
    }
}
