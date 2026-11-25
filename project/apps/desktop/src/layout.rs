//! Justified rows: every row fills the width and pictures keep their shape. Physical pixels.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    pub index: usize,
    pub x: u32,
    pub width: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub height: u32,
    pub cells: Vec<Placed>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    pub width: u32,
    pub gap: u32,
    pub target_height: u32,
}

/// Extremely tall or wide pictures are cropped to these shapes inside the grid.
const ASPECT: std::ops::RangeInclusive<f32> = 0.56..=2.0;

/// Lays out `aspects[start..]`. Without `finish` an incomplete last row is left for later.
/// Returns the rows and the index of the first picture that was not placed.
pub fn justify(aspects: &[f32], start: usize, metrics: Metrics, finish: bool) -> (Vec<Row>, usize) {
    let mut rows = vec![];
    let (mut first, mut index, mut sum) = (start, start, 0.0);
    while index < aspects.len() {
        let aspect = clamp(aspects[index]);
        let height = |count: usize, sum: f32| {
            let gaps = metrics.gap * (count as u32 - 1);
            metrics.width.saturating_sub(gaps).max(1) as f32 / sum
        };
        let with = height(index + 1 - first, sum + aspect);
        if with > metrics.target_height as f32 {
            sum += aspect;
            index += 1;
            continue;
        }
        // The row is full. Keep this picture only if that gives the better height.
        let distance = |height: f32| (height / metrics.target_height as f32).ln().abs();
        let without = (index > first).then(|| height(index - first, sum));
        match without.filter(|h| distance(*h) < distance(with)) {
            Some(height) => rows.push(row(aspects, first..index, height, metrics.gap)),
            None => {
                rows.push(row(aspects, first..index + 1, with, metrics.gap));
                index += 1;
            }
        }
        (first, sum) = (index, 0.0);
    }
    if finish && first < aspects.len() {
        rows.push(row(
            aspects,
            first..aspects.len(),
            metrics.target_height as f32,
            metrics.gap,
        ));
        first = aspects.len();
    }
    (rows, first)
}

fn clamp(aspect: f32) -> f32 {
    aspect.clamp(*ASPECT.start(), *ASPECT.end())
}

fn row(aspects: &[f32], range: std::ops::Range<usize>, height: f32, gap: u32) -> Row {
    // Rounding the running edge instead of each width keeps the right side straight.
    let (mut edge, mut left) = (0.0_f32, 0_u32);
    let cells = range
        .enumerate()
        .map(|(position, index)| {
            edge += clamp(aspects[index]) * height;
            let right = (edge.round() as u32).max(left + 1);
            let placed = Placed {
                index,
                x: left + gap * position as u32,
                width: right - left,
            };
            left = right;
            placed
        })
        .collect();
    Row {
        height: (height.round() as u32).max(1),
        cells,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const METRICS: Metrics = Metrics {
        width: 1200,
        gap: 12,
        target_height: 220,
    };

    #[test]
    fn full_rows_fill_the_width_exactly() {
        let aspects = [1.78, 0.56, 1.6, 3.56, 1.78, 1.5, 1.78, 1.33, 0.4, 1.78];
        let (rows, _) = justify(&aspects, 0, METRICS, false);
        assert!(rows.len() >= 2);
        for row in rows {
            let last = row.cells.last().unwrap();
            assert!((last.x + last.width).abs_diff(1200) <= 1, "{row:?}");
            assert!((150..=320).contains(&row.height), "{row:?}");
        }
    }

    #[test]
    fn cells_do_not_overlap_and_keep_the_gap() {
        let (rows, _) = justify(&[1.78; 30], 0, METRICS, true);
        for pair in rows.iter().flat_map(|r| r.cells.windows(2)) {
            assert_eq!(pair[1].x, pair[0].x + pair[0].width + 12);
        }
    }

    #[test]
    fn incomplete_row_waits_for_the_next_page() {
        let (rows, next) = justify(&[1.78; 5], 0, METRICS, false);
        let placed: usize = rows.iter().map(|r| r.cells.len()).sum();
        assert_eq!(placed, next);
        assert!(next < 5);
        let (rest, end) = justify(&[1.78; 5], next, METRICS, true);
        assert_eq!((rest.len(), end), (1, 5));
        assert_eq!(rest[0].height, 220);
    }

    #[test]
    fn survives_a_zero_width() {
        let (rows, end) = justify(
            &[1.0, 1.0],
            0,
            Metrics {
                width: 0,
                ..METRICS
            },
            true,
        );
        assert_eq!((rows.len(), end), (2, 2));
    }
}
