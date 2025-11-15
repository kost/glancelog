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
        Self::from_file_with_dir(filename, None)
    }

    pub fn from_file_with_dir(filename: &str, custom_dir: Option<&str>) -> Result<Self> {
        let mut paths = Vec::new();

        // Priority 1: Custom directory from parameter (highest priority)
        if let Some(dir) = custom_dir {
            paths.push(PathBuf::from(dir).join(filename));
        }

        // Priority 2: Environment variable GLANCELOG_FILTERDIR
        if let Ok(env_dir) = std::env::var("GLANCELOG_FILTERDIR") {
            paths.push(PathBuf::from(env_dir).join(filename));
        }

        // Priority 3: User home directory ~/.glancelog/filters
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join(".glancelog").join("filters").join(filename));
        }

        // Priority 4: Default search paths
        paths.extend(vec![
            PathBuf::from(format!("./filters/{}", filename)),
            PathBuf::from(format!("/var/lib/glancelog/filters/{}", filename)),
            PathBuf::from(format!("/usr/local/glancelog/var/lib/filters/{}", filename)),
            PathBuf::from(format!("/opt/glancelog/var/lib/filters/{}", filename)),
        ]);

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
