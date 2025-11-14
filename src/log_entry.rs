use chrono::{Datelike, Local, DateTime, NaiveDate, NaiveDateTime, NaiveTime};
use regex::Regex;
use anyhow::{Result, anyhow};
use std::io::{BufRead, BufReader};
use std::fs::File;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub host: String,
    pub daemon: String,
    pub log_entry: String,
}

impl LogEntry {
    pub fn new() -> Self {
        Self {
            year: 1900,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            host: "#".to_string(),
            daemon: "#".to_string(),
            log_entry: "#".to_string(),
        }
    }

    pub fn set_abnormal(&mut self, value: &str) {
        self.year = 1900;
        self.month = 1;
        self.day = 1;
        self.hour = 0;
        self.minute = 0;
        self.second = 0;
        self.host = "#".to_string();
        self.daemon = "#".to_string();
        self.log_entry = value.to_string();
    }
}

pub trait LogParser: Send + Sync {
    fn is_type(&self, line: &str) -> bool;
    fn parse(&self, line: &str) -> Result<LogEntry>;
    fn name(&self) -> &'static str;
}

pub struct SyslogParser;

impl LogParser for SyslogParser {
    fn is_type(&self, line: &str) -> bool {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return false;
        }

        // Check for month pattern like "Feb", "Jan", etc.
        let month_re = Regex::new(r"^[A-Z][a-z]{2}$").unwrap();
        let day_re = Regex::new(r"^[0-9]{1,2}$").unwrap();
        let time_re = Regex::new(r"^[0-9]{1,2}:[0-9]{2}:[0-9]{2}$").unwrap();

        month_re.is_match(parts[0]) &&
        day_re.is_match(parts[1]) &&
        time_re.is_match(parts[2]) &&
        !parts[4].starts_with("pam_") &&
        !parts.get(3).map(|s| s.starts_with("sshd[")).unwrap_or(false)
    }

    fn parse(&self, line: &str) -> Result<LogEntry> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 5 {
            let mut entry = LogEntry::new();
            entry.set_abnormal(line);
            return Ok(entry);
        }

        let month_str = parts[0];
        let day_str = parts[1];
        let time_str = parts[2];
        let host = parts[3].to_string();
        let daemon = parts[4].to_string();
        let log_entry = parts[5..].join(" ");

        // Parse time
        let time_parts: Vec<&str> = time_str.split(':').collect();
        if time_parts.len() != 3 {
            return Err(anyhow!("Invalid time format"));
        }

        let hour: u32 = time_parts[0].parse()?;
        let minute: u32 = time_parts[1].parse()?;
        let second: u32 = time_parts[2].parse()?;

        // Parse month
        let month = match month_str {
            "Jan" => 1, "Feb" => 2, "Mar" => 3, "Apr" => 4,
            "May" => 5, "Jun" => 6, "Jul" => 7, "Aug" => 8,
            "Sep" => 9, "Oct" => 10, "Nov" => 11, "Dec" => 12,
            _ => return Err(anyhow!("Invalid month")),
        };

        let day: u32 = day_str.parse()?;
        let year = Local::now().year();

        Ok(LogEntry {
            year,
            month,
            day,
            hour,
            minute,
            second,
            host,
            daemon,
            log_entry,
        })
    }

    fn name(&self) -> &'static str {
        "Syslog"
    }
}

pub struct RSyslogParser;

impl LogParser for RSyslogParser {
    fn is_type(&self, line: &str) -> bool {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let timestamp_re = Regex::new(r"^\d{4}-\d{2}-\d{2}T").unwrap();
        timestamp_re.is_match(parts[0])
    }

    fn parse(&self, line: &str) -> Result<LogEntry> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 3 {
            let mut entry = LogEntry::new();
            entry.set_abnormal(line);
            return Ok(entry);
        }

        // Parse timestamp like "2010-06-24T17:56:32.197716-04:00"
        let timestamp_str = parts[0];
        let host = parts[1].to_string();
        let daemon = parts[2].to_string();
        let log_entry = parts[3..].join(" ");

        // Split by 'T'
        let dt_parts: Vec<&str> = timestamp_str.split('T').collect();
        if dt_parts.len() != 2 {
            return Err(anyhow!("Invalid timestamp format"));
        }

        let date_str = dt_parts[0];
        let time_zone_str = dt_parts[1];

        // Parse date
        let date_parts: Vec<&str> = date_str.split('-').collect();
        if date_parts.len() != 3 {
            return Err(anyhow!("Invalid date format"));
        }

        let year: i32 = date_parts[0].parse()?;
        let month: u32 = date_parts[1].parse()?;
        let day: u32 = date_parts[2].parse()?;

        // Parse time (remove timezone info)
        let time_str = time_zone_str.split(&['-', '+'][..]).next().unwrap();
        let time_str = time_str.split('.').next().unwrap(); // Remove microseconds

        let time_parts: Vec<&str> = time_str.split(':').collect();
        if time_parts.len() != 3 {
            return Err(anyhow!("Invalid time format"));
        }

        let hour: u32 = time_parts[0].parse()?;
        let minute: u32 = time_parts[1].parse()?;
        let second: u32 = time_parts[2].parse()?;

        Ok(LogEntry {
            year,
            month,
            day,
            hour,
            minute,
            second,
            host,
            daemon,
            log_entry,
        })
    }

    fn name(&self) -> &'static str {
        "RSyslog"
    }
}

pub struct SecureLogParser;

impl LogParser for SecureLogParser {
    fn is_type(&self, line: &str) -> bool {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            return false;
        }

        let day_re = Regex::new(r"^[0-9]{1,2}$").unwrap();
        let time_re = Regex::new(r"^[0-9]{1,2}:[0-9]{2}:[0-9]{2}$").unwrap();

        day_re.is_match(parts[1]) &&
        time_re.is_match(parts[2]) &&
        (parts[5].starts_with("pam_") || parts[4].starts_with("sshd["))
    }

    fn parse(&self, line: &str) -> Result<LogEntry> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 5 {
            let mut entry = LogEntry::new();
            entry.set_abnormal(line);
            return Ok(entry);
        }

        let month_str = parts[0];
        let day_str = parts[1];
        let time_str = parts[2];
        let host = parts[3].to_string();
        let daemon = parts[4].to_string();
        let log_entry = parts[5..].join(" ");

        // Parse time
        let time_parts: Vec<&str> = time_str.split(':').collect();
        if time_parts.len() != 3 {
            return Err(anyhow!("Invalid time format"));
        }

        let hour: u32 = time_parts[0].parse()?;
        let minute: u32 = time_parts[1].parse()?;
        let second: u32 = time_parts[2].parse()?;

        // Parse month
        let month = match month_str {
            "Jan" => 1, "Feb" => 2, "Mar" => 3, "Apr" => 4,
            "May" => 5, "Jun" => 6, "Jul" => 7, "Aug" => 8,
            "Sep" => 9, "Oct" => 10, "Nov" => 11, "Dec" => 12,
            _ => return Err(anyhow!("Invalid month")),
        };

        let day: u32 = day_str.parse()?;
        let year = Local::now().year();

        Ok(LogEntry {
            year,
            month,
            day,
            hour,
            minute,
            second,
            host,
            daemon,
            log_entry,
        })
    }

    fn name(&self) -> &'static str {
        "SecureLog"
    }
}

pub struct JournalctlParser;

impl LogParser for JournalctlParser {
    fn is_type(&self, line: &str) -> bool {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return false;
        }

        // Check for month pattern like "Feb", "Jan", etc.
        let month_re = Regex::new(r"^[A-Z][a-z]{2}$").unwrap();
        let day_re = Regex::new(r"^[0-9]{1,2}$").unwrap();
        let time_re = Regex::new(r"^[0-9]{1,2}:[0-9]{2}:[0-9]{2}$").unwrap();

        // Journalctl typically has daemon[pid] format or just daemon:
        let daemon_re = Regex::new(r"^[a-zA-Z0-9_\-\.]+(\[[0-9]+\])?:?$").unwrap();

        month_re.is_match(parts[0]) &&
        day_re.is_match(parts[1]) &&
        time_re.is_match(parts[2]) &&
        parts.len() >= 4 &&
        daemon_re.is_match(parts[4])
    }

    fn parse(&self, line: &str) -> Result<LogEntry> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 5 {
            let mut entry = LogEntry::new();
            entry.set_abnormal(line);
            return Ok(entry);
        }

        let month_str = parts[0];
        let day_str = parts[1];
        let time_str = parts[2];
        let host = parts[3].to_string();

        // Daemon might be like "systemd[1]:" or "kernel:" or just "sshd"
        let daemon_raw = parts[4];
        let daemon = daemon_raw.trim_end_matches(':').to_string();

        // The log entry starts after the daemon field
        let log_entry = parts[5..].join(" ");

        // Parse time
        let time_parts: Vec<&str> = time_str.split(':').collect();
        if time_parts.len() != 3 {
            return Err(anyhow!("Invalid time format"));
        }

        let hour: u32 = time_parts[0].parse()?;
        let minute: u32 = time_parts[1].parse()?;
        let second: u32 = time_parts[2].parse()?;

        // Parse month
        let month = match month_str {
            "Jan" => 1, "Feb" => 2, "Mar" => 3, "Apr" => 4,
            "May" => 5, "Jun" => 6, "Jul" => 7, "Aug" => 8,
            "Sep" => 9, "Oct" => 10, "Nov" => 11, "Dec" => 12,
            _ => return Err(anyhow!("Invalid month")),
        };

        let day: u32 = day_str.parse()?;
        let year = Local::now().year();

        Ok(LogEntry {
            year,
            month,
            day,
            hour,
            minute,
            second,
            host,
            daemon,
            log_entry,
        })
    }

    fn name(&self) -> &'static str {
        "Journalctl"
    }
}

pub struct ApacheCommonParser;

impl LogParser for ApacheCommonParser {
    fn is_type(&self, line: &str) -> bool {
        // Apache Common Log Format: IP - user [timestamp] "request" status bytes
        let re = Regex::new(r#"^\S+ \S+ \S+ \[\d{2}/\w{3}/\d{4}:\d{2}:\d{2}:\d{2} [+-]\d{4}\] "\S+ \S+ \S+" \d+ (?:\d+|-)$"#).unwrap();
        re.is_match(line)
    }

    fn parse(&self, line: &str) -> Result<LogEntry> {
        // Format: IP ident authuser [timestamp] "request" status bytes
        let re = Regex::new(r#"^(\S+) (\S+) (\S+) \[(\d{2})/(\w{3})/(\d{4}):(\d{2}):(\d{2}):(\d{2}) ([+-]\d{4})\] "([^"]+)" (\d+) (\S+)"#).unwrap();

        let caps = re.captures(line).ok_or_else(|| anyhow!("Failed to parse Apache Common Log"))?;

        let ip = caps.get(1).unwrap().as_str();
        let day: u32 = caps.get(4).unwrap().as_str().parse()?;
        let month_str = caps.get(5).unwrap().as_str();
        let year: i32 = caps.get(6).unwrap().as_str().parse()?;
        let hour: u32 = caps.get(7).unwrap().as_str().parse()?;
        let minute: u32 = caps.get(8).unwrap().as_str().parse()?;
        let second: u32 = caps.get(9).unwrap().as_str().parse()?;
        let request = caps.get(11).unwrap().as_str();
        let status = caps.get(12).unwrap().as_str();
        let bytes = caps.get(13).unwrap().as_str();

        let month = match month_str {
            "Jan" => 1, "Feb" => 2, "Mar" => 3, "Apr" => 4,
            "May" => 5, "Jun" => 6, "Jul" => 7, "Aug" => 8,
            "Sep" => 9, "Oct" => 10, "Nov" => 11, "Dec" => 12,
            _ => return Err(anyhow!("Invalid month")),
        };

        // Extract HTTP method from request
        let daemon = request.split_whitespace().next().unwrap_or("HTTP").to_string();
        let log_entry = format!("{} {} {}", request, status, bytes);

        Ok(LogEntry {
            year,
            month,
            day,
            hour,
            minute,
            second,
            host: ip.to_string(),
            daemon,
            log_entry,
        })
    }

    fn name(&self) -> &'static str {
        "ApacheCommon"
    }
}

pub struct ApacheCombinedParser;

impl LogParser for ApacheCombinedParser {
    fn is_type(&self, line: &str) -> bool {
        // Apache Combined Log Format: IP - user [timestamp] "request" status bytes "referer" "user-agent"
        let re = Regex::new(r#"^\S+ \S+ \S+ \[\d{2}/\w{3}/\d{4}:\d{2}:\d{2}:\d{2} [+-]\d{4}\] "\S+ \S+ \S+" \d+ (?:\d+|-) "[^"]*" "[^"]*"$"#).unwrap();
        re.is_match(line)
    }

    fn parse(&self, line: &str) -> Result<LogEntry> {
        // Format: IP ident authuser [timestamp] "request" status bytes "referer" "user-agent"
        let re = Regex::new(r#"^(\S+) (\S+) (\S+) \[(\d{2})/(\w{3})/(\d{4}):(\d{2}):(\d{2}):(\d{2}) ([+-]\d{4})\] "([^"]+)" (\d+) (\S+) "([^"]*)" "([^"]*)"#).unwrap();

        let caps = re.captures(line).ok_or_else(|| anyhow!("Failed to parse Apache Combined Log"))?;

        let ip = caps.get(1).unwrap().as_str();
        let day: u32 = caps.get(4).unwrap().as_str().parse()?;
        let month_str = caps.get(5).unwrap().as_str();
        let year: i32 = caps.get(6).unwrap().as_str().parse()?;
        let hour: u32 = caps.get(7).unwrap().as_str().parse()?;
        let minute: u32 = caps.get(8).unwrap().as_str().parse()?;
        let second: u32 = caps.get(9).unwrap().as_str().parse()?;
        let request = caps.get(11).unwrap().as_str();
        let status = caps.get(12).unwrap().as_str();
        let bytes = caps.get(13).unwrap().as_str();
        let referer = caps.get(14).unwrap().as_str();
        let user_agent = caps.get(15).unwrap().as_str();

        let month = match month_str {
            "Jan" => 1, "Feb" => 2, "Mar" => 3, "Apr" => 4,
            "May" => 5, "Jun" => 6, "Jul" => 7, "Aug" => 8,
            "Sep" => 9, "Oct" => 10, "Nov" => 11, "Dec" => 12,
            _ => return Err(anyhow!("Invalid month")),
        };

        // Extract HTTP method from request
        let daemon = request.split_whitespace().next().unwrap_or("HTTP").to_string();
        let log_entry = format!("{} {} {} \"{}\" \"{}\"", request, status, bytes, referer, user_agent);

        Ok(LogEntry {
            year,
            month,
            day,
            hour,
            minute,
            second,
            host: ip.to_string(),
            daemon,
            log_entry,
        })
    }

    fn name(&self) -> &'static str {
        "ApacheCombined"
    }
}

pub struct AwsElbParser;

impl LogParser for AwsElbParser {
    fn is_type(&self, line: &str) -> bool {
        // AWS ELB format: timestamp elb client:port backend:port request_time backend_time response_time elb_status backend_status ...
        // Backend field can be "-" when no backend connection
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z \S+ \d+\.\d+\.\d+\.\d+:\d+ (\d+\.\d+\.\d+\.\d+:\d+|-) [\d\.-]+ [\d\.-]+ [\d\.-]+ \d+ ").unwrap();
        re.is_match(line)
    }

    fn parse(&self, line: &str) -> Result<LogEntry> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 13 {
            return Err(anyhow!("Invalid AWS ELB log format"));
        }

        // Parse timestamp: 2015-05-24T19:21:39.218145Z
        let timestamp_str = parts[0];
        let re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})").unwrap();
        let caps = re.captures(timestamp_str).ok_or_else(|| anyhow!("Failed to parse timestamp"))?;

        let year: i32 = caps.get(1).unwrap().as_str().parse()?;
        let month: u32 = caps.get(2).unwrap().as_str().parse()?;
        let day: u32 = caps.get(3).unwrap().as_str().parse()?;
        let hour: u32 = caps.get(4).unwrap().as_str().parse()?;
        let minute: u32 = caps.get(5).unwrap().as_str().parse()?;
        let second: u32 = caps.get(6).unwrap().as_str().parse()?;

        let _elb_name = parts[1];
        let client = parts[2].split(':').next().unwrap_or("unknown");
        let elb_status = parts[7];
        let backend_status = parts[8];

        // Extract HTTP method from request (parts 11 onwards contain the quoted request)
        let request_start = line.find('"').unwrap_or(0);
        let request_end = line.rfind('"').unwrap_or(line.len());
        let request = if request_start < request_end {
            &line[request_start+1..request_end]
        } else {
            "-"
        };

        let daemon = request.split_whitespace().next().unwrap_or("HTTP").to_string();
        let log_entry = format!("{} elb_status={} backend_status={}", request, elb_status, backend_status);

        Ok(LogEntry {
            year,
            month,
            day,
            hour,
            minute,
            second,
            host: client.to_string(),
            daemon,
            log_entry,
        })
    }

    fn name(&self) -> &'static str {
        "AWS-ELB"
    }
}

pub struct AwsAlbParser;

impl LogParser for AwsAlbParser {
    fn is_type(&self, line: &str) -> bool {
        // AWS ALB format starts with: http/https/h2/grpc/ws/wss timestamp
        let re = Regex::new(r"^(http|https|h2|grpc|ws|wss) \d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z").unwrap();
        re.is_match(line)
    }

    fn parse(&self, line: &str) -> Result<LogEntry> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 15 {
            return Err(anyhow!("Invalid AWS ALB log format"));
        }

        // Parse timestamp: 2018-07-02T22:23:00.186641Z
        let timestamp_str = parts[1];
        let re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})").unwrap();
        let caps = re.captures(timestamp_str).ok_or_else(|| anyhow!("Failed to parse timestamp"))?;

        let year: i32 = caps.get(1).unwrap().as_str().parse()?;
        let month: u32 = caps.get(2).unwrap().as_str().parse()?;
        let day: u32 = caps.get(3).unwrap().as_str().parse()?;
        let hour: u32 = caps.get(4).unwrap().as_str().parse()?;
        let minute: u32 = caps.get(5).unwrap().as_str().parse()?;
        let second: u32 = caps.get(6).unwrap().as_str().parse()?;

        let protocol = parts[0];
        let client = parts[3].split(':').next().unwrap_or("unknown");
        let elb_status = parts[8];
        let target_status = parts[9];

        // Extract HTTP request (in quotes)
        let request_start = line.find('"').unwrap_or(0);
        let request_end = line[request_start+1..].find('"').map(|i| i + request_start + 1).unwrap_or(line.len());
        let request = if request_start < request_end {
            &line[request_start+1..request_end]
        } else {
            "-"
        };

        let daemon = request.split_whitespace().next().unwrap_or(protocol).to_string();
        let log_entry = format!("{} elb_status={} target_status={} protocol={}", request, elb_status, target_status, protocol);

        Ok(LogEntry {
            year,
            month,
            day,
            hour,
            minute,
            second,
            host: client.to_string(),
            daemon,
            log_entry,
        })
    }

    fn name(&self) -> &'static str {
        "AWS-ALB"
    }
}

pub struct MysqlGeneralParser;

impl LogParser for MysqlGeneralParser {
    fn is_type(&self, line: &str) -> bool {
        // MySQL general log: YYYY-MM-DDTHH:MM:SS.microsZ    thread_id command_type    query
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z\s+\d+\s+(Query|Connect|Quit|Init|Execute)").unwrap();
        re.is_match(line)
    }

    fn parse(&self, line: &str) -> Result<LogEntry> {
        // Format: 2023-11-14T10:30:45.123456Z    42 Query     SELECT * FROM users
        // Query text is optional (e.g., Quit command has no query)
        let re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})\.\d+Z\s+(\d+)\s+(\w+)\s*(.*)$").unwrap();
        let caps = re.captures(line).ok_or_else(|| anyhow!("Failed to parse MySQL general log"))?;

        let year: i32 = caps.get(1).unwrap().as_str().parse()?;
        let month: u32 = caps.get(2).unwrap().as_str().parse()?;
        let day: u32 = caps.get(3).unwrap().as_str().parse()?;
        let hour: u32 = caps.get(4).unwrap().as_str().parse()?;
        let minute: u32 = caps.get(5).unwrap().as_str().parse()?;
        let second: u32 = caps.get(6).unwrap().as_str().parse()?;
        let thread_id = caps.get(7).unwrap().as_str();
        let command_type = caps.get(8).unwrap().as_str();
        let query = caps.get(9).unwrap().as_str();

        Ok(LogEntry {
            year,
            month,
            day,
            hour,
            minute,
            second,
            host: format!("thread_{}", thread_id),
            daemon: command_type.to_string(),
            log_entry: query.to_string(),
        })
    }

    fn name(&self) -> &'static str {
        "MySQL-General"
    }
}

pub struct PostgresqlParser;

impl LogParser for PostgresqlParser {
    fn is_type(&self, line: &str) -> bool {
        // PostgreSQL log: YYYY-MM-DD HH:MM:SS.mmm TZ [pid] user@database LEVEL: message
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d+ \w+ \[\d+\] \S+@\S+ (LOG|ERROR|WARNING|FATAL|PANIC|DEBUG|INFO|NOTICE|STATEMENT):").unwrap();
        re.is_match(line)
    }

    fn parse(&self, line: &str) -> Result<LogEntry> {
        // Format: 2023-11-14 10:30:45.123 UTC [12345] postgres@testdb LOG: message
        let re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2}):(\d{2})\.\d+ \w+ \[(\d+)\] (\S+)@(\S+) (\w+):\s*(.*)$").unwrap();
        let caps = re.captures(line).ok_or_else(|| anyhow!("Failed to parse PostgreSQL log"))?;

        let year: i32 = caps.get(1).unwrap().as_str().parse()?;
        let month: u32 = caps.get(2).unwrap().as_str().parse()?;
        let day: u32 = caps.get(3).unwrap().as_str().parse()?;
        let hour: u32 = caps.get(4).unwrap().as_str().parse()?;
        let minute: u32 = caps.get(5).unwrap().as_str().parse()?;
        let second: u32 = caps.get(6).unwrap().as_str().parse()?;
        let _pid = caps.get(7).unwrap().as_str();
        let user = caps.get(8).unwrap().as_str();
        let database = caps.get(9).unwrap().as_str();
        let level = caps.get(10).unwrap().as_str();
        let message = caps.get(11).unwrap().as_str();

        Ok(LogEntry {
            year,
            month,
            day,
            hour,
            minute,
            second,
            host: format!("{}@{}", user, database),
            daemon: level.to_string(),
            log_entry: message.to_string(),
        })
    }

    fn name(&self) -> &'static str {
        "PostgreSQL"
    }
}

pub struct RawParser;

impl LogParser for RawParser {
    fn is_type(&self, line: &str) -> bool {
        !line.trim().is_empty()
    }

    fn parse(&self, line: &str) -> Result<LogEntry> {
        let mut entry = LogEntry::new();
        entry.set_abnormal(line);
        Ok(entry)
    }

    fn name(&self) -> &'static str {
        "Raw"
    }
}

pub struct CrunchLog {
    pub entries: Vec<LogEntry>,
    pub parser_type: String,
}

impl CrunchLog {
    pub fn from_stdin() -> Result<Self> {
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());
        Self::from_reader(reader)
    }

    pub fn from_file(filename: &str) -> Result<Self> {
        use std::path::Path;

        // Check if it's an EVTX file
        let path = Path::new(filename);
        if crate::evtx_parser::EvtxLogParser::is_evtx_file(path) {
            let entries = crate::evtx_parser::EvtxLogParser::parse_file(path)?;
            return Ok(CrunchLog {
                entries,
                parser_type: "EVTX".to_string(),
            });
        }

        // Otherwise, use text-based parsing
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        Self::from_reader(reader)
    }

    fn from_reader<R: BufRead>(reader: R) -> Result<Self> {
        let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;

        if lines.is_empty() {
            return Err(anyhow!("No data found"));
        }

        // Try to detect the log format
        // Order matters: more specific parsers should come first
        let parsers: Vec<Box<dyn LogParser>> = vec![
            Box::new(AwsElbParser),
            Box::new(AwsAlbParser),
            Box::new(MysqlGeneralParser),
            Box::new(PostgresqlParser),
            Box::new(RSyslogParser),
            Box::new(JournalctlParser),
            Box::new(ApacheCombinedParser),
            Box::new(ApacheCommonParser),
            Box::new(SyslogParser),
            Box::new(SecureLogParser),
            Box::new(RawParser),
        ];

        let parser_idx = Self::detect_parser(&lines, &parsers)?;
        let detected_parser = &parsers[parser_idx];
        let parser_type = detected_parser.name().to_string();

        let mut entries = Vec::new();
        for line in lines {
            match detected_parser.parse(&line) {
                Ok(entry) => entries.push(entry),
                Err(_) => {
                    // Try to parse as abnormal entry
                    let mut entry = LogEntry::new();
                    entry.set_abnormal(&line);
                    entries.push(entry);
                }
            }
        }

        Ok(CrunchLog {
            entries,
            parser_type,
        })
    }

    fn detect_parser(lines: &[String], parsers: &[Box<dyn LogParser>]) -> Result<usize> {
        let sample_size = 10.min(lines.len());
        let mut scores = vec![0; parsers.len()];

        for _ in 0..sample_size {
            let idx = rand::random::<usize>() % lines.len();
            let line = &lines[idx];

            for (i, parser) in parsers.iter().enumerate() {
                if parser.is_type(line) {
                    scores[i] += 1;
                }
            }
        }

        // Find parser with highest score
        let max_score = scores.iter().max().unwrap_or(&0);
        let threshold = sample_size / 4;

        for (i, score) in scores.iter().enumerate() {
            if score >= &threshold && score == max_score {
                return Ok(i);
            }
        }

        // Default to raw parser
        Ok(parsers.len() - 1)
    }

    fn entry_to_datetime(entry: &LogEntry) -> DateTime<Local> {
        let naive_date = NaiveDate::from_ymd_opt(entry.year, entry.month, entry.day)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(1900, 1, 1).unwrap());
        let naive_time = NaiveTime::from_hms_opt(entry.hour, entry.minute, entry.second)
            .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let naive_datetime = NaiveDateTime::new(naive_date, naive_time);
        DateTime::from_naive_utc_and_offset(naive_datetime, *Local::now().offset())
    }

    pub fn filter_by_time(&mut self, from: Option<DateTime<Local>>, to: Option<DateTime<Local>>) {
        self.entries.retain(|entry| {
            let entry_dt = Self::entry_to_datetime(entry);

            // Check 'from' filter
            if let Some(from_dt) = from {
                if entry_dt < from_dt {
                    return false;
                }
            }

            // Check 'to' filter
            if let Some(to_dt) = to {
                if entry_dt > to_dt {
                    return false;
                }
            }

            true
        });
    }
}
