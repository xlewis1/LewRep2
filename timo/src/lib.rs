use chrono::{DateTime, TimeZone, Utc, Datelike, Offset};
use chrono_tz::Tz;


pub struct TimoDateTime {
    pub inner: DateTime<Tz>,
}

impl TimoDateTime {
    pub fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32, tz_str: &str) -> Result<Self, String> {
        let tz: Tz = tz_str.parse().map_err(|_| format!("Unknown timezone: {}", tz_str))?;

        match tz.with_ymd_and_hms(year, month, day, hour, minute, second) {
            chrono::LocalResult::Single(dt) => Ok(TimoDateTime { inner: dt }),
            chrono::LocalResult::Ambiguous(dt1, _dt2) => {
                Ok(TimoDateTime { inner: dt1 })
            }
            chrono::LocalResult::None => Err("Invalid local time (skipped due to seasonal DST jump).".to_string()),
        }
    }

    pub fn now(tz_str: &str) -> Result<Self, String> {
        let tz: Tz = tz_str.parse().map_err(|_| format!("Unknown timezone: {}", tz_str))?;
        let utc_now = Utc::now();
        Ok(TimoDateTime {
            inner: utc_now.with_timezone(&tz),
        })
    }

    pub fn switch_timezone(&self, new_tz_str: &str) -> Result<Self, String> {
        let new_tz: Tz = new_tz_str.parse().map_err(|_| format!("Unknown timezone: {}", new_tz_str))?;
        Ok(TimoDateTime {
            inner: self.inner.with_timezone(&new_tz),
        })
    }

    pub fn is_summertime(&self) -> bool {
        let current_offset = self.inner.offset().fix().local_minus_utc();

        let winter_time = self.inner.timezone().with_ymd_and_hms(self.inner.year(), 1, 1, 0, 0, 0).unwrap();
        let winter_offset = winter_time.offset().fix().local_minus_utc();
        
        current_offset > winter_offset

    }

    pub fn status_summary(&self) -> String {
        use chrono_tz::OffsetName;
        format!(
            "Time: {} | Zone: {} | Season: {} | Moon: {}",
            self.inner.format("%Y-%m-%d %H:%M:%S"),
            self.inner.offset().abbreviation(),
            if self.is_summertime() { "Summertime (DST)"} else { "Wintertime / Standard Time"},
            self.moon_phase()
        )
    }

    pub fn days_in_month(&self) -> u32 {
        let year = self.inner.year();
        let month = self.inner.month();

        if month == 12 {
            31
        } else {
            let next_month = self.inner.timezone()
                .with_ymd_and_hms(year, month + 1, 1, 0, 0, 0)
                .unwrap();

            let last_day_of_current = next_month - chrono::Duration::days(1);
            last_day_of_current.day()
        }
    }

    pub fn weekday_of_first(&self) -> u32 {
        let first_of_month = self.inner.timezone()
            .with_ymd_and_hms(self.inner.year(), self.inner.month(), 1, 0, 0, 0)
            .unwrap();

        first_of_month.weekday().num_days_from_monday()
    }

    
    pub fn days_until_next_month(&self) -> u32 {
        let total_days = self.days_in_month();
        let current_day = self.inner.day();
        
        total_days - current_day
    }

   
    pub fn next_month_name(&self) -> String {
        let month = self.inner.month();
        let next_month_num = if month == 12 { 1 } else { month + 1 };
        
        match next_month_num {
            1 => "January", 2 => "February", 3 => "March", 4 => "April",
            5 => "May", 6 => "June", 7 => "July", 8 => "August",
            9 => "September", 10 => "October", 11 => "November", _ => "December",
        }.to_string()
    }

    pub fn moon_phase(&self) -> &'static str {
        let year = self.inner.year() as i32;
        let month = self.inner.month() as i32;
        let day = self.inner.day() as i32;

        let c = year / 100;
        let mut y = year % 100;

        let mut m = month;
        if m < 3 {
            y -= 1;
            m += 12;
        }

        let mut epact = (11 * (y % 19) + 11) % 30;

        if c == 20 {
            epact = (epact + 29) % 30;
        }

        let mut age = (epact + day + (m - 3) + 2) % 30;
        if age < 0 {
            age += 30;
        }

        match age {
            0 | 29 => "🌑 New Moon",
            1..=6  => "🌒 Waxing Crescent",
            7..=8  => "🌓 First Quarter",
            9..=13 => "🌔 Waxing Gibbous",
            14..=15 => "🌕 Full Moon",
            16..=21 => "🌖 Waning Gibbous",
            22..=23 => "🌗 Third Quarter",
            _       => "🌘 Waning Crescent",
        }
    }
}