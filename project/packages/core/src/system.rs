use std::path::PathBuf;

/// Wall clock time where the user is. Days are counted from 1970-01-01.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalTime {
    pub day: i64,
    /// Minutes since midnight.
    pub minute: u32,
}

impl LocalTime {
    pub fn new(year: i64, month: i64, day: i64, hour: u32, minute: u32) -> Self {
        // Days from civil, by Howard Hinnant: years start in March, so leap days come last.
        let year = if month <= 2 { year - 1 } else { year };
        let era = year.div_euclid(400);
        let year_of_era = year.rem_euclid(400);
        let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        Self {
            day: era * 146_097 + day_of_era - 719_468,
            minute: hour * 60 + minute,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Autostart {
    Off,
    On,
    /// Requested by this app, but turned off in the settings of the system.
    Blocked,
}

/// What the app needs to know about the machine besides its desktop.
pub trait System: Send + Sync {
    fn now(&self) -> LocalTime;
    /// The user is in a full screen game, video or presentation.
    fn do_not_disturb(&self) -> bool;
    fn autostart(&self) -> Autostart;
    /// Returns the state afterwards, or why it could not be changed.
    fn set_autostart(&self, on: bool) -> Result<Autostart, String>;
    /// Lets the user choose a folder. Blocks until they are done; nothing if they cancel.
    fn pick_folder(&self) -> Option<PathBuf>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_days_like_the_calendar() {
        let day = |y, m, d| LocalTime::new(y, m, d, 0, 0).day;
        assert_eq!(day(1970, 1, 1), 0);
        assert_eq!(day(1970, 3, 1), 59);
        assert_eq!(day(2000, 3, 1) - day(2000, 2, 28), 2);
        assert_eq!(day(2100, 3, 1) - day(2100, 2, 28), 1);
        assert_eq!(day(2027, 1, 1) - day(2026, 12, 31), 1);
        assert_eq!(day(2026, 11, 25), 20_782);
        assert_eq!(LocalTime::new(2026, 11, 25, 8, 30).minute, 510);
    }
}
