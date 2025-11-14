use crate::log_entry::LogEntry;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Datelike, Local, Timelike};
use evtx::{EvtxParser, SerializedEvtxRecord};
use std::path::Path;

pub struct EvtxLogParser;

impl EvtxLogParser {
    pub fn parse_file(path: &Path) -> Result<Vec<LogEntry>> {
        let mut parser = EvtxParser::from_path(path)
            .map_err(|e| anyhow!("Failed to open EVTX file: {}", e))?;

        let mut entries = Vec::new();
        let mut parse_errors = Vec::new();
        let mut total_records = 0;

        for record in parser.records_json_value() {
            total_records += 1;
            match record {
                Ok(record) => {
                    match Self::convert_record_to_entry(record) {
                        Ok(entry) => entries.push(entry),
                        Err(e) => {
                            if parse_errors.len() < 5 {
                                parse_errors.push(format!("Record {}: {}", total_records, e));
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read EVTX record {}: {}", total_records, e);
                }
            }
        }

        if entries.is_empty() {
            eprintln!("Processed {} records but couldn't parse any successfully.", total_records);
            if !parse_errors.is_empty() {
                eprintln!("Sample errors:");
                for err in parse_errors {
                    eprintln!("  - {}", err);
                }
            }
            return Err(anyhow!("No valid EVTX records found"));
        }

        Ok(entries)
    }

    fn convert_record_to_entry(record: SerializedEvtxRecord<serde_json::Value>) -> Result<LogEntry> {
        let data = &record.data;

        // Extract Event data - handle both "Event" wrapper and direct System access
        let event = if let Some(e) = data.get("Event") {
            e
        } else {
            data
        };

        // Extract System information
        let system = event.get("System")
            .ok_or_else(|| anyhow!("No System field"))?;

        // Extract timestamp - try multiple paths
        let timestamp_str = system.get("TimeCreated")
            .and_then(|tc| tc.get("#attributes"))
            .and_then(|attr| attr.get("SystemTime"))
            .and_then(|st| st.as_str())
            .or_else(|| {
                system.get("TimeCreated")
                    .and_then(|tc| tc.get("SystemTime"))
                    .and_then(|st| st.as_str())
            })
            .or_else(|| {
                system.get("TimeCreated")
                    .and_then(|tc| tc.as_str())
            })
            .ok_or_else(|| anyhow!("No timestamp found in System/TimeCreated"))?;

        // Parse the timestamp (format: 2025-11-14T12:00:00.123456Z or variations)
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
            .or_else(|_| {
                // Try with 'Z' appended if missing
                DateTime::parse_from_rfc3339(&format!("{}Z", timestamp_str))
            })
            .map_err(|e| anyhow!("Failed to parse timestamp '{}': {}", timestamp_str, e))?;
        let local_time: DateTime<Local> = timestamp.with_timezone(&Local);

        // Extract provider name (daemon equivalent) - try multiple paths
        let provider = system.get("Provider")
            .and_then(|p| p.get("#attributes"))
            .and_then(|attr| attr.get("Name"))
            .and_then(|n| n.as_str())
            .or_else(|| {
                system.get("Provider")
                    .and_then(|p| p.get("Name"))
                    .and_then(|n| n.as_str())
            })
            .or_else(|| {
                system.get("Provider")
                    .and_then(|p| p.as_str())
            })
            .unwrap_or("Unknown")
            .to_string();

        // Extract computer name (host equivalent)
        let computer = system.get("Computer")
            .and_then(|c| c.as_str())
            .unwrap_or("Unknown")
            .to_string();

        // Extract Event ID
        let event_id = system.get("EventID")
            .and_then(|id| id.as_u64())
            .unwrap_or(0);

        // Extract Event data or message
        let mut log_message = String::new();

        // Try to get EventData
        if let Some(event_data) = event.get("EventData") {
            if let Some(data_obj) = event_data.as_object() {
                let mut parts = Vec::new();
                for (key, value) in data_obj {
                    if key != "#attributes" {
                        parts.push(format!("{}={}", key, value));
                    }
                }
                if !parts.is_empty() {
                    log_message = parts.join(" ");
                }
            } else if let Some(data_str) = event_data.as_str() {
                log_message = data_str.to_string();
            }
        }

        // Try to get UserData if EventData is empty
        if log_message.is_empty() {
            if let Some(user_data) = event.get("UserData") {
                log_message = format!("{:?}", user_data);
            }
        }

        // If still empty, use a default message
        if log_message.is_empty() {
            log_message = format!("EventID {}", event_id);
        } else {
            log_message = format!("EventID {} {}", event_id, log_message);
        }

        // Extract level/severity
        let level = system.get("Level")
            .and_then(|l| l.as_u64())
            .unwrap_or(0);

        let level_str = match level {
            1 => "Critical",
            2 => "Error",
            3 => "Warning",
            4 => "Information",
            5 => "Verbose",
            _ => "Unknown",
        };

        // Prepend level to log message
        log_message = format!("[{}] {}", level_str, log_message);

        Ok(LogEntry {
            year: local_time.year(),
            month: local_time.month(),
            day: local_time.day(),
            hour: local_time.hour(),
            minute: local_time.minute(),
            second: local_time.second(),
            host: computer,
            daemon: provider,
            log_entry: log_message,
        })
    }

    pub fn is_evtx_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            ext.to_string_lossy().to_lowercase() == "evtx"
        } else {
            false
        }
    }
}
