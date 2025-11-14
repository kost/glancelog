use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct Filter {
    stopwords: Vec<Regex>,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            stopwords: Vec::new(),
        }
    }

    pub fn from_file(filename: &str) -> Result<Self> {
        let paths = vec![
            PathBuf::from(format!("/var/lib/glancelog/filters/{}", filename)),
            PathBuf::from(format!("/usr/local/glancelog/var/lib/filters/{}", filename)),
            PathBuf::from(format!("/opt/glancelog/var/lib/filters/{}", filename)),
            PathBuf::from(format!("./filters/{}", filename)),
        ];

        for path in paths {
            if path.exists() {
                return Self::load_from_path(&path);
            }
        }

        // Return empty filter if file not found
        Ok(Self::new())
    }

    fn load_from_path(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut stopwords = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                match Regex::new(trimmed) {
                    Ok(re) => stopwords.push(re),
                    Err(e) => eprintln!("Warning: Invalid regex '{}': {}", trimmed, e),
                }
            }
        }

        Ok(Self { stopwords })
    }

    pub fn scrub(&self, input: &str) -> String {
        let mut result = input.to_string();

        for stopword in &self.stopwords {
            result = stopword.replace_all(&result, "#").to_string();
        }

        result
    }

    pub fn bleach(&self, input: &str) -> bool {
        self.scrub(input) == "#"
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}
