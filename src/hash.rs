use crate::filter::Filter;
use crate::log_entry::{CrunchLog, LogEntry};
use std::collections::HashMap;
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy)]
pub enum HashMode {
    Hash,
    Daemon,
    Host,
    WordCount,
}

#[derive(Debug, Clone, Copy)]
pub enum SampleMode {
    None,
    Threshold,
    All,
}

pub struct SuperHash {
    data: HashMap<String, (usize, Vec<LogEntry>)>,
    filter: Filter,
    sample_mode: SampleMode,
    sample_threshold: usize,
}

impl SuperHash {
    pub fn new(filter: Filter) -> Self {
        Self {
            data: HashMap::new(),
            filter,
            sample_mode: SampleMode::Threshold,
            sample_threshold: 3,
        }
    }

    pub fn set_sample_threshold(&mut self, threshold: usize) {
        self.sample_threshold = threshold;
    }

    pub fn set_sample_mode(&mut self, mode: SampleMode) {
        self.sample_mode = mode;
    }

    pub fn increment(&mut self, key: String, entry: LogEntry) {
        self.data
            .entry(key)
            .and_modify(|(count, entries)| {
                *count += 1;
                entries.push(entry.clone());
            })
            .or_insert((1, vec![entry]));
    }

    pub fn display(&self) {
        // Sort by count (descending) and then alphabetically
        let mut items: Vec<_> = self.data.iter().collect();
        items.sort_by(|a, b| {
            let count_cmp = b.1.0.cmp(&a.1.0);
            if count_cmp == std::cmp::Ordering::Equal {
                a.0.cmp(b.0)
            } else {
                count_cmp
            }
        });

        for (key, (count, entries)) in items {
            if key == "#" {
                continue;
            }

            match self.sample_mode {
                SampleMode::All => {
                    // Show random sample
                    if let Some(entry) = entries.choose(&mut rand::thread_rng()) {
                        println!("{}:\t{}", count, entry.log_entry);
                    }
                }
                SampleMode::None => {
                    println!("{}:\t{}", count, key);
                }
                SampleMode::Threshold => {
                    if *count <= self.sample_threshold {
                        // Show first entry for small counts
                        if let Some(entry) = entries.first() {
                            println!("{}:\t{}", count, entry.log_entry);
                        }
                    } else {
                        println!("{}:\t{}", count, key);
                    }
                }
            }
        }
    }

    pub fn from_log(log: &CrunchLog, mode: HashMode, filter: Filter) -> Self {
        let mut hash = Self::new(filter);

        match mode {
            HashMode::Hash => hash.fill_hash(log),
            HashMode::Daemon => hash.fill_daemon(log),
            HashMode::Host => hash.fill_host(log),
            HashMode::WordCount => hash.fill_wordcount(log),
        }

        // Remove valueless entries
        hash.data.remove("#");

        hash
    }

    fn fill_hash(&mut self, log: &CrunchLog) {
        for entry in &log.entries {
            let key = format!("{} {}", entry.daemon, entry.log_entry);
            let key = self.filter.scrub(&key);
            self.increment(key, entry.clone());
        }
    }

    fn fill_daemon(&mut self, log: &CrunchLog) {
        for entry in &log.entries {
            let key = self.filter.scrub(&entry.daemon);
            self.increment(key, entry.clone());
        }
    }

    fn fill_host(&mut self, log: &CrunchLog) {
        for entry in &log.entries {
            let key = self.filter.scrub(&entry.host);
            self.increment(key, entry.clone());
        }
    }

    fn fill_wordcount(&mut self, log: &CrunchLog) {
        let mut word_map: HashMap<String, Vec<String>> = HashMap::new();

        // First pass: collect all words
        for entry in &log.entries {
            for word in entry.log_entry.split_whitespace() {
                word_map
                    .entry(word.to_string())
                    .or_insert_with(Vec::new)
                    .push(word.to_string());
            }
        }

        // Second pass: scrub and merge
        let mut scrubbed_map: HashMap<String, usize> = HashMap::new();
        for (word, instances) in word_map {
            let scrubbed = self.filter.scrub(&word);
            if scrubbed != "#" {
                *scrubbed_map.entry(scrubbed).or_insert(0) += instances.len();
            }
        }

        // Convert to our data structure
        for (word, count) in scrubbed_map {
            let mut entry = LogEntry::new();
            entry.log_entry = word.clone();
            for _ in 0..count {
                self.increment(word.clone(), entry.clone());
            }
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
