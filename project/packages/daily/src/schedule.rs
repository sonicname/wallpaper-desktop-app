use wallpaper_core::LocalTime;

/// The day a wallpaper belongs to. Until the hour of the change it is still yesterday.
pub fn current_day(now: LocalTime, change_at: u32) -> i64 {
    if now.minute >= change_at {
        now.day
    } else {
        now.day - 1
    }
}

/// Asking again never changes the wallpaper twice, whatever woke the app up.
pub fn is_due(last: Option<i64>, now: LocalTime, change_at: u32) -> bool {
    last.is_none_or(|last| last < current_day(now, change_at))
}

/// A last change in the future means that the clock was wrong. Left alone, nothing would change
/// until that day comes.
pub fn correct(last: Option<i64>, now: LocalTime, change_at: u32) -> Option<i64> {
    last.map(|last| last.min(current_day(now, change_at)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EIGHT: u32 = 8 * 60;

    fn at(day: i64, hour: u32, minute: u32) -> LocalTime {
        LocalTime {
            day,
            minute: hour * 60 + minute,
        }
    }

    #[test]
    fn changes_once_a_day_at_the_chosen_time() {
        assert!(!is_due(Some(99), at(100, 7, 59), EIGHT));
        assert!(is_due(Some(99), at(100, 8, 0), EIGHT));
        assert!(!is_due(Some(100), at(100, 8, 0), EIGHT));
        assert!(!is_due(Some(100), at(100, 23, 59), EIGHT));
        assert!(!is_due(Some(100), at(101, 0, 1), EIGHT));
        assert!(is_due(Some(100), at(101, 8, 0), EIGHT));
    }

    #[test]
    fn missed_days_are_made_up_once() {
        let now = at(107, 9, 0);
        assert!(is_due(Some(100), now, EIGHT));
        assert!(!is_due(Some(current_day(now, EIGHT)), now, EIGHT));
    }

    #[test]
    fn the_first_time_is_due_at_any_hour() {
        assert!(is_due(None, at(100, 3, 0), EIGHT));
        // Changed at three in the morning, which still belongs to yesterday.
        assert_eq!(current_day(at(100, 3, 0), EIGHT), 99);
    }

    #[test]
    fn moving_the_clock_back_never_changes_twice_nor_blocks() {
        // Flying west, or the end of summer time.
        assert!(!is_due(Some(100), at(100, 7, 0), EIGHT));
        assert!(!is_due(Some(100), at(99, 23, 0), EIGHT));
        // The clock was a year ahead.
        let corrected = correct(Some(465), at(100, 9, 0), EIGHT);
        assert_eq!(corrected, Some(100));
        assert!(is_due(corrected, at(101, 9, 0), EIGHT));
    }

    #[test]
    fn midnight_works_as_the_chosen_time() {
        assert!(is_due(Some(99), at(100, 0, 0), 0));
        assert!(!is_due(Some(100), at(100, 0, 0), 0));
    }
}
