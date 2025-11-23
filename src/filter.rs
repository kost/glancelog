use regex::Regex;
use std::fs::{File, create_dir_all};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use anyhow::Result;

// Embedded default filter files
const EMBEDDED_HASH_STOPWORDS: &str = include_str!("../filters/hash.stopwords");
const EMBEDDED_WORDS_STOPWORDS: &str = include_str!("../filters/words.stopwords");
const EMBEDDED_DAEMON_STOPWORDS: &str = include_str!("../filters/daemon.stopwords");
const EMBEDDED_HOST_STOPWORDS: &str = include_str!("../filters/host.stopwords");

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

        // Priority 5: Use embedded default filters as fallback
        if let Some(embedded_content) = Self::get_embedded_filter(filename) {
            return Self::load_from_string(embedded_content);
        }

        // Return empty filter if no embedded filter exists
        Ok(Self::new())
    }

    fn get_embedded_filter(filename: &str) -> Option<&'static str> {
        match filename {
            "hash.stopwords" => Some(EMBEDDED_HASH_STOPWORDS),
            "words.stopwords" => Some(EMBEDDED_WORDS_STOPWORDS),
            "daemon.stopwords" => Some(EMBEDDED_DAEMON_STOPWORDS),
            "host.stopwords" => Some(EMBEDDED_HOST_STOPWORDS),
            _ => None,
        }
    }

    fn load_from_string(content: &str) -> Result<Self> {
        let mut stopwords = Vec::new();

        for line in content.lines() {
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

    /// Export all embedded filters to a directory
    pub fn export_embedded_filters(target_dir: &Path) -> Result<()> {
        create_dir_all(target_dir)?;

        let filters = vec![
            ("hash.stopwords", EMBEDDED_HASH_STOPWORDS),
            ("words.stopwords", EMBEDDED_WORDS_STOPWORDS),
            ("daemon.stopwords", EMBEDDED_DAEMON_STOPWORDS),
            ("host.stopwords", EMBEDDED_HOST_STOPWORDS),
        ];

        for (filename, content) in filters {
            let file_path = target_dir.join(filename);
            let mut file = File::create(&file_path)?;
            file.write_all(content.as_bytes())?;
            eprintln!("Exported: {}", file_path.display());
        }

        Ok(())
    }

    /// Export embedded filters to user's home directory
    pub fn export_to_home() -> Result<()> {
        if let Some(home_dir) = dirs::home_dir() {
            let target_dir = home_dir.join(".glancelog").join("filters");
            Self::export_embedded_filters(&target_dir)?;
            eprintln!("Filters exported to: {}", target_dir.display());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Could not determine home directory"))
        }
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}
