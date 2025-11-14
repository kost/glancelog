use crate::log_entry::CrunchLog;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum GraphType {
    Seconds,
    Minutes,
    Hours,
    Days,
    Months,
    Years,
}

pub struct GraphHash {
    data: HashMap<String, usize>,
    start_date: DateTime<Local>,
    middle_date: DateTime<Local>,
    end_date: DateTime<Local>,
    max_value: usize,
    min_value: usize,
    duration: i64,
    unit: &'static str,
    tick: char,
    wide: bool,
}

impl GraphHash {
    pub fn new(log: &CrunchLog, graph_type: GraphType) -> Self {
        Self::new_with_range(log, graph_type, None, None)
    }

    pub fn new_with_range(
        log: &CrunchLog,
        graph_type: GraphType,
        from: Option<DateTime<Local>>,
        to: Option<DateTime<Local>>
    ) -> Self {
        let mut graph = Self {
            data: HashMap::new(),
            start_date: Local::now(),
            middle_date: Local::now(),
            end_date: Local::now(),
            max_value: 0,
            min_value: 0,
            duration: 0,
            unit: "",
            tick: '#',
            wide: false,
        };

        if log.entries.is_empty() {
            return graph;
        }

        // Determine start date: use --from if provided, otherwise first entry
        let start_date = if let Some(from_dt) = from {
            from_dt
        } else {
            let first_entry = &log.entries[0];
            Self::entry_to_datetime(first_entry)
        };

        // Determine if we have a custom range
        let custom_range = from.is_some() || to.is_some();

        match graph_type {
            GraphType::Seconds => graph.fill_seconds(log, start_date, to, custom_range),
            GraphType::Minutes => graph.fill_minutes(log, start_date, to, custom_range),
            GraphType::Hours => graph.fill_hours(log, start_date, to, custom_range),
            GraphType::Days => graph.fill_days(log, start_date, to, custom_range),
            GraphType::Months => graph.fill_months(log, start_date, to, custom_range),
            GraphType::Years => graph.fill_years(log, start_date, to, custom_range),
        }

        graph.calculate_stats();
        graph
    }

    fn entry_to_datetime(entry: &crate::log_entry::LogEntry) -> DateTime<Local> {
        let naive_date = NaiveDate::from_ymd_opt(entry.year, entry.month, entry.day)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(1900, 1, 1).unwrap());
        let naive_time = NaiveTime::from_hms_opt(entry.hour, entry.minute, entry.second)
            .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let naive_datetime = NaiveDateTime::new(naive_date, naive_time);
        DateTime::from_naive_utc_and_offset(naive_datetime, *Local::now().offset())
    }

    fn fill_seconds(&mut self, log: &CrunchLog, start_date: DateTime<Local>, to: Option<DateTime<Local>>, custom_range: bool) {
        self.unit = "second";
        self.start_date = start_date;

        // Calculate duration
        if custom_range && to.is_some() {
            let end_dt = to.unwrap();
            let diff = end_dt.signed_duration_since(start_date);
            self.duration = diff.num_seconds().max(1);
        } else {
            self.duration = 60;
        }

        // Initialize all keys with zero
        for i in 0..self.duration {
            let date = start_date + Duration::seconds(i);
            let key = format!("{}{:02}{:02}{:02}{:02}{:02}",
                date.year(), date.month(), date.day(),
                date.hour(), date.minute(), date.second());
            self.data.insert(key, 0);
        }

        self.middle_date = start_date + Duration::seconds(self.duration / 2);
        self.end_date = start_date + Duration::seconds(self.duration - 1);

        // Fill with actual data
        for entry in &log.entries {
            let key = format!("{}{:02}{:02}{:02}{:02}{:02}",
                entry.year, entry.month, entry.day,
                entry.hour, entry.minute, entry.second);
            if let Some(count) = self.data.get_mut(&key) {
                *count += 1;
            }
        }
    }

    fn fill_minutes(&mut self, log: &CrunchLog, start_date: DateTime<Local>, to: Option<DateTime<Local>>, custom_range: bool) {
        self.unit = "minute";
        self.start_date = start_date;

        // Calculate duration
        if custom_range && to.is_some() {
            let end_dt = to.unwrap();
            let diff = end_dt.signed_duration_since(start_date);
            self.duration = diff.num_minutes().max(1);
        } else {
            self.duration = 60;
        }

        for i in 0..self.duration {
            let date = start_date + Duration::minutes(i);
            let key = format!("{}{:02}{:02}{:02}{:02}",
                date.year(), date.month(), date.day(),
                date.hour(), date.minute());
            self.data.insert(key, 0);
        }

        self.middle_date = start_date + Duration::minutes(self.duration / 2);
        self.end_date = start_date + Duration::minutes(self.duration - 1);

        for entry in &log.entries {
            let key = format!("{}{:02}{:02}{:02}{:02}",
                entry.year, entry.month, entry.day,
                entry.hour, entry.minute);
            if let Some(count) = self.data.get_mut(&key) {
                *count += 1;
            }
        }
    }

    fn fill_hours(&mut self, log: &CrunchLog, start_date: DateTime<Local>, to: Option<DateTime<Local>>, custom_range: bool) {
        self.unit = "hour";
        self.start_date = start_date;

        // Calculate duration
        if custom_range && to.is_some() {
            let end_dt = to.unwrap();
            let diff = end_dt.signed_duration_since(start_date);
            self.duration = diff.num_hours().max(1);
        } else {
            self.duration = 24;
        }

        for i in 0..self.duration {
            let date = start_date + Duration::hours(i);
            let key = format!("{}{:02}{:02}{:02}",
                date.year(), date.month(), date.day(), date.hour());
            self.data.insert(key, 0);
        }

        self.middle_date = start_date + Duration::hours(self.duration / 2);
        self.end_date = start_date + Duration::hours(self.duration - 1);

        for entry in &log.entries {
            let key = format!("{}{:02}{:02}{:02}",
                entry.year, entry.month, entry.day, entry.hour);
            if let Some(count) = self.data.get_mut(&key) {
                *count += 1;
            }
        }
    }

    fn fill_days(&mut self, log: &CrunchLog, start_date: DateTime<Local>, to: Option<DateTime<Local>>, custom_range: bool) {
        self.unit = "day";
        self.start_date = start_date;

        // Calculate duration
        if custom_range && to.is_some() {
            let end_dt = to.unwrap();
            let diff = end_dt.signed_duration_since(start_date);
            self.duration = diff.num_days().max(1);
        } else {
            self.duration = 31;
        }

        for i in 0..self.duration {
            let date = start_date + Duration::days(i);
            let key = format!("{}{:02}{:02}",
                date.year(), date.month(), date.day());
            self.data.insert(key, 0);
        }

        self.middle_date = start_date + Duration::days(self.duration / 2);
        self.end_date = start_date + Duration::days(self.duration - 1);

        for entry in &log.entries {
            let key = format!("{}{:02}{:02}",
                entry.year, entry.month, entry.day);
            if let Some(count) = self.data.get_mut(&key) {
                *count += 1;
            }
        }
    }

    fn fill_months(&mut self, log: &CrunchLog, start_date: DateTime<Local>, to: Option<DateTime<Local>>, custom_range: bool) {
        self.unit = "month";
        self.start_date = start_date;

        // Calculate duration
        if custom_range && to.is_some() {
            let end_dt = to.unwrap();
            let diff = end_dt.signed_duration_since(start_date);
            self.duration = (diff.num_days() / 30).max(1);
        } else {
            self.duration = 12;
        }

        for i in 0..self.duration {
            let days_offset = (i * 365) / 12 + 1;
            let date = start_date + Duration::days(days_offset);
            let key = format!("{}{:02}", date.year(), date.month());
            self.data.insert(key, 0);
        }

        self.middle_date = start_date + Duration::days((self.duration * 365) / 24);
        self.end_date = start_date + Duration::days((self.duration * 365) / 12);

        for entry in &log.entries {
            let key = format!("{}{:02}", entry.year, entry.month);
            if let Some(count) = self.data.get_mut(&key) {
                *count += 1;
            }
        }
    }

    fn fill_years(&mut self, log: &CrunchLog, start_date: DateTime<Local>, to: Option<DateTime<Local>>, custom_range: bool) {
        self.unit = "year";
        self.start_date = start_date;

        // Calculate duration
        if custom_range && to.is_some() {
            let end_dt = to.unwrap();
            let diff = end_dt.signed_duration_since(start_date);
            self.duration = (diff.num_days() / 365).max(1);
        } else {
            self.duration = 10;
        }

        for i in 0..self.duration {
            let date = start_date + Duration::days(i * 365);
            let key = format!("{}", date.year());
            self.data.insert(key, 0);
        }

        self.middle_date = start_date + Duration::days((self.duration * 365) / 2);
        self.end_date = start_date + Duration::days(self.duration * 365);

        for entry in &log.entries {
            let key = format!("{}", entry.year);
            if let Some(count) = self.data.get_mut(&key) {
                *count += 1;
            }
        }
    }

    fn calculate_stats(&mut self) {
        self.max_value = *self.data.values().max().unwrap_or(&0);
        self.min_value = *self.data.values().min().unwrap_or(&0);
    }

    pub fn set_tick(&mut self, tick: char) {
        self.tick = tick;
    }

    pub fn set_wide(&mut self, wide: bool) {
        self.wide = wide;
    }

    pub fn display(&self) {
        let graph_height = 6;
        let graph_width = self.data.len();

        if graph_width == 0 {
            println!("No data to graph");
            return;
        }

        let (char_fill, char_blank) = if self.wide {
            (format!("{} ", self.tick), "  ".to_string())
        } else {
            (self.tick.to_string(), " ".to_string())
        };

        // Get sorted keys
        let mut keys: Vec<_> = self.data.keys().cloned().collect();
        keys.sort();

        // Normalize data for display
        let mut normalized: HashMap<String, usize> = HashMap::new();
        let graph_min = self.min_value;
        let graph_max = self.max_value;

        for key in &keys {
            let value = self.data[key];
            if value > 0 {
                let normalized_value = if graph_max > graph_min {
                    ((value - graph_min) as f64 / (graph_max - graph_min) as f64 * graph_height as f64).ceil() as usize
                } else {
                    (value as f64 / graph_max as f64 * graph_height as f64).ceil() as usize
                };
                normalized.insert(key.clone(), normalized_value);
            } else {
                normalized.insert(key.clone(), 0);
            }
        }

        // Print graph
        println!();
        for i in (1..graph_height).rev() {
            for key in &keys {
                if normalized[key] >= i {
                    print!("{}", char_fill);
                } else {
                    print!("{}", char_blank);
                }
            }
            println!();
        }

        // Bottom line
        for _ in &keys {
            print!("{}", char_fill);
        }
        println!();

        // Print time markers
        let display_width = if self.wide { graph_width * 2 } else { graph_width };
        let pos_begin = 1;
        let pos_middle = display_width / 2;
        let pos_end = display_width.saturating_sub(3);

        let val_begin = self.start_date_value();
        let val_middle = self.middle_date_value();
        let val_end = self.end_date_value();

        for i in 1..display_width {
            if i == pos_begin {
                print!("{:02}", val_begin % 2000);
            } else if i == pos_middle {
                print!("{:02}", val_middle % 2000);
            } else if i == pos_end {
                print!("{:02}", val_end % 2000);
            } else {
                print!(" ");
            }
        }
        println!();

        // Summary
        println!();
        println!("Start Time:\t{}\t\tMinimum Value: {}", self.start_date.format("%Y-%m-%d %H:%M:%S"), self.min_value);
        println!("End Time:\t{}\t\tMaximum Value: {}", self.end_date.format("%Y-%m-%d %H:%M:%S"), self.max_value);
        let scale = if graph_height > 0 {
            (self.max_value - self.min_value) as f64 / graph_height as f64
        } else {
            0.0
        };
        println!("Duration:\t{} {}s\t\t\tScale: {:.2}", self.duration, self.unit, scale);
        println!();
    }

    fn start_date_value(&self) -> i64 {
        match self.unit {
            "second" => self.start_date.second() as i64,
            "minute" => self.start_date.minute() as i64,
            "hour" => self.start_date.hour() as i64,
            "day" => self.start_date.day() as i64,
            "month" => self.start_date.month() as i64,
            "year" => self.start_date.year() as i64,
            _ => 0,
        }
    }

    fn middle_date_value(&self) -> i64 {
        match self.unit {
            "second" => self.middle_date.second() as i64,
            "minute" => self.middle_date.minute() as i64,
            "hour" => self.middle_date.hour() as i64,
            "day" => self.middle_date.day() as i64,
            "month" => self.middle_date.month() as i64,
            "year" => self.middle_date.year() as i64,
            _ => 0,
        }
    }

    fn end_date_value(&self) -> i64 {
        match self.unit {
            "second" => self.end_date.second() as i64,
            "minute" => self.end_date.minute() as i64,
            "hour" => self.end_date.hour() as i64,
            "day" => self.end_date.day() as i64,
            "month" => self.end_date.month() as i64,
            "year" => self.end_date.year() as i64,
            _ => 0,
        }
    }
}
