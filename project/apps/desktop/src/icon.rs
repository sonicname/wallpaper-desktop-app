use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

const SIZE: u32 = 64;
/// Samples per pixel and direction.
const DETAIL: u32 = 4;

/// A placeholder until the app has a name and a logo: hills under a sun.
pub fn icon() -> Image {
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(SIZE, SIZE);
    let step = 1.0 / (SIZE * DETAIL) as f32;
    for (index, pixel) in buffer.make_mut_slice().iter_mut().enumerate() {
        let (column, row) = (index as u32 % SIZE, index as u32 / SIZE);
        let mut sum = [0.0_f32; 4];
        for sample in 0..DETAIL * DETAIL {
            let x = (column * DETAIL + sample % DETAIL) as f32 * step;
            let y = (row * DETAIL + sample / DETAIL) as f32 * step;
            if let Some(color) = color_at(x, y) {
                for (total, channel) in sum.iter_mut().zip(color) {
                    *total += channel as f32;
                }
                sum[3] += 1.0;
            }
        }
        if sum[3] > 0.0 {
            let channel = |i: usize| (sum[i] / sum[3]).round() as u8;
            let alpha = sum[3] * 255.0 / (DETAIL * DETAIL) as f32;
            *pixel = Rgba8Pixel::new(channel(0), channel(1), channel(2), alpha.round() as u8);
        }
    }
    Image::from_rgba8(buffer)
}

/// The colour at a point of the unit square, or nothing outside the rounded shape.
fn color_at(x: f32, y: f32) -> Option<[u8; 3]> {
    const RADIUS: f32 = 0.22;
    let outside = |v: f32| (RADIUS - v).max(v - (1.0 - RADIUS)).max(0.0);
    if outside(x).hypot(outside(y)) > RADIUS {
        return None;
    }
    let near = 0.62 + (x - 0.3).abs() * 0.55;
    let far = 0.42 + (x - 0.72).abs() * 0.8;
    Some(if y > near {
        [0x1b, 0x13, 0x06]
    } else if y > far {
        [0x5a, 0x3d, 0x16]
    } else if (x - 0.3).hypot(y - 0.3) < 0.12 {
        [0xff, 0xf3, 0xdc]
    } else {
        [0xf2, 0xb4, 0x5c]
    })
}
