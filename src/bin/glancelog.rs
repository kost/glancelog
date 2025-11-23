use clap::Parser;
use glancelog::{CrunchLog, Filter, GraphHash, GraphType, HashMode, SuperHash};
use glancelog::hash::SampleMode;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Parser)]
#[command(name = "glancelog")]
#[command(author = "glancelog contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Log analysis tool for systems administrators", long_about = None)]
struct Cli {
    /// Input file (or use stdin if not provided)
    file: Option<String>,

    /// Verbose output
    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Show sample output for small numbered entries
    #[arg(long)]
    sample: bool,

    /// Do not sample output for low count entries
    #[arg(long)]
    nosample: bool,

    /// Show samples instead of munged text for all entries
    #[arg(long)]
    allsample: bool,

    /// Use filter files during processing
    #[arg(long)]
    filter: bool,

    /// Do not use filter files during processing
    #[arg(long)]
    nofilter: bool,

    /// Custom directory for filter files (overrides default paths and GLANCELOG_FILTERDIR)
    #[arg(long)]
    filter_dir: Option<String>,

    /// Export embedded default filters to a directory (defaults to ~/.glancelog/filters)
    #[arg(long)]
    export_filters: Option<Option<String>>,

    /// Use wider graph characters
    #[arg(long)]
    wide: bool,

    /// Change tick character from default
    #[arg(long, default_value = "#")]
    tick: String,

    /// Set threshold for rare vs common events (default: 3)
    #[arg(short = 'l', long, default_value = "3")]
    lowcount: usize,

    /// Filter logs from this datetime (format: "YYYY-MM-DD HH:MM:SS" or "YYYY-MM-DD")
    #[arg(long)]
    from: Option<String>,

    /// Filter logs to this datetime (format: "YYYY-MM-DD HH:MM:SS" or "YYYY-MM-DD")
    #[arg(long)]
    to: Option<String>,

    /// Print log lines as-is (respects --from/--to filters)
    #[arg(short = 'p', long, group = "mode")]
    print: bool,

    /// Show hashes of log files with numbers removed
    #[arg(long, group = "mode")]
    hash: bool,

    /// Show word count for given word
    #[arg(long, group = "mode")]
    wordcount: bool,

    /// Show a report of entries from each daemon
    #[arg(long, group = "mode")]
    daemon: bool,

    /// Show a report of entries from each host
    #[arg(long, group = "mode")]
    host: bool,

    /// Show graph of first 60 seconds
    #[arg(long, group = "mode")]
    sgraph: bool,

    /// Show graph of first 60 minutes
    #[arg(long, group = "mode")]
    mgraph: bool,

    /// Show graph of first 24 hours
    #[arg(long, group = "mode")]
    hgraph: bool,

    /// Show graph of first 31 days
    #[arg(long, group = "mode")]
    dgraph: bool,

    /// Show graph of first 12 months
    #[arg(long, group = "mode")]
    mograph: bool,

    /// Show graph of first 10 years
    #[arg(long, group = "mode")]
    ygraph: bool,
}

fn main() {
    let cli = Cli::parse();

    // Handle filter export if requested
    if let Some(export_path) = &cli.export_filters {
        let result = if let Some(path) = export_path {
            Filter::export_embedded_filters(std::path::Path::new(path))
        } else {
            Filter::export_to_home()
        };

        match result {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("Error exporting filters: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Load log
    let log = if let Some(filename) = &cli.file {
        match CrunchLog::from_file(filename) {
            Ok(log) => log,
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        match CrunchLog::from_stdin() {
            Ok(log) => log,
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                std::process::exit(1);
            }
        }
    };

    if cli.verbose > 0 {
        eprintln!("Detected log format: {}", log.parser_type);
        eprintln!("Loaded {} entries", log.entries.len());
    }

    // Apply time filters if specified
    let log = apply_time_filters(log, &cli);

    if cli.verbose > 0 && (cli.from.is_some() || cli.to.is_some()) {
        eprintln!("After filtering: {} entries", log.entries.len());
    }

    // Parse from/to datetimes for use in graph modes
    let from_dt = cli.from.as_ref().and_then(|s| {
        match parse_datetime(s) {
            Ok(dt) => Some(dt),
            Err(_) => None,  // Already handled in apply_time_filters
        }
    });

    let to_dt = cli.to.as_ref().and_then(|s| {
        match parse_datetime(s) {
            Ok(dt) => Some(dt),
            Err(_) => None,  // Already handled in apply_time_filters
        }
    });

    // Determine mode and execute
    if cli.print {
        mode_print(&log);
    } else if cli.hash {
        mode_hash(&cli, &log);
    } else if cli.wordcount {
        mode_wordcount(&cli, &log);
    } else if cli.daemon {
        mode_daemon(&cli, &log);
    } else if cli.host {
        mode_host(&cli, &log);
    } else if cli.sgraph {
        mode_graph(&cli, &log, GraphType::Seconds, from_dt, to_dt);
    } else if cli.mgraph {
        mode_graph(&cli, &log, GraphType::Minutes, from_dt, to_dt);
    } else if cli.hgraph {
        mode_graph(&cli, &log, GraphType::Hours, from_dt, to_dt);
    } else if cli.dgraph {
        mode_graph(&cli, &log, GraphType::Days, from_dt, to_dt);
    } else if cli.mograph {
        mode_graph(&cli, &log, GraphType::Months, from_dt, to_dt);
    } else if cli.ygraph {
        mode_graph(&cli, &log, GraphType::Years, from_dt, to_dt);
    } else {
        // Default to hash mode
        mode_hash(&cli, &log);
    }
}

fn mode_print(log: &CrunchLog) {
    for entry in &log.entries {
        // Format: YYYY-MM-DDTHH:MM:SS host daemon: message
        // Some parsers include trailing ":" in daemon field, some don't
        let daemon_separator = if entry.daemon.ends_with(':') { "" } else { ":" };

        // Strip leading ": " or " " from log_entry if present (added by some parsers)
        let message = entry.log_entry
            .strip_prefix(": ")
            .or_else(|| entry.log_entry.strip_prefix(" "))
            .unwrap_or(&entry.log_entry);

        println!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {} {}{} {}",
            entry.year, entry.month, entry.day,
            entry.hour, entry.minute, entry.second,
            entry.host,
            entry.daemon,
            daemon_separator,
            message
        );
    }
}

fn mode_hash(cli: &Cli, log: &CrunchLog) {
    let filter = if cli.nofilter {
        Filter::new()
    } else {
        Filter::from_file_with_dir("hash.stopwords", cli.filter_dir.as_deref())
            .unwrap_or_else(|_| Filter::new())
    };

    let mut hash = SuperHash::from_log(log, HashMode::Hash, filter);

    // Set sample threshold
    hash.set_sample_threshold(cli.lowcount);

    // Set sample mode
    if cli.allsample {
        hash.set_sample_mode(SampleMode::All);
    } else if cli.nosample {
        hash.set_sample_mode(SampleMode::None);
    } else {
        hash.set_sample_mode(SampleMode::Threshold);
    }

    hash.display();
}

fn mode_wordcount(cli: &Cli, log: &CrunchLog) {
    let filter = if cli.nofilter {
        Filter::new()
    } else {
        Filter::from_file_with_dir("words.stopwords", cli.filter_dir.as_deref())
            .unwrap_or_else(|_| Filter::new())
    };

    let mut hash = SuperHash::from_log(log, HashMode::WordCount, filter);
    hash.set_sample_mode(SampleMode::None);
    hash.display();
}

fn mode_daemon(cli: &Cli, log: &CrunchLog) {
    let filter = if cli.nofilter {
        Filter::new()
    } else {
        Filter::from_file_with_dir("daemon.stopwords", cli.filter_dir.as_deref())
            .unwrap_or_else(|_| Filter::new())
    };

    let mut hash = SuperHash::from_log(log, HashMode::Daemon, filter);
    hash.set_sample_mode(SampleMode::None);
    hash.display();
}

fn mode_host(cli: &Cli, log: &CrunchLog) {
    let filter = if cli.nofilter {
        Filter::new()
    } else {
        Filter::from_file_with_dir("host.stopwords", cli.filter_dir.as_deref())
            .unwrap_or_else(|_| Filter::new())
    };

    let mut hash = SuperHash::from_log(log, HashMode::Host, filter);
    hash.set_sample_mode(SampleMode::None);
    hash.display();
}

fn mode_graph(cli: &Cli, log: &CrunchLog, graph_type: GraphType, from: Option<DateTime<Local>>, to: Option<DateTime<Local>>) {
    let mut graph = GraphHash::new_with_range(log, graph_type, from, to);

    // Set tick character
    if let Some(tick_char) = cli.tick.chars().next() {
        graph.set_tick(tick_char);
    }

    graph.set_wide(cli.wide);
    graph.display();
}

fn parse_datetime(datetime_str: &str) -> Result<DateTime<Local>, String> {
    // Try parsing "YYYY-MM-DD HH:MM:SS"
    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S") {
        return Ok(DateTime::from_naive_utc_and_offset(naive_dt, *Local::now().offset()));
    }

    // Try parsing "YYYY-MM-DD" (assume start of day)
    if let Ok(naive_date) = NaiveDate::parse_from_str(datetime_str, "%Y-%m-%d") {
        let naive_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let naive_dt = NaiveDateTime::new(naive_date, naive_time);
        return Ok(DateTime::from_naive_utc_and_offset(naive_dt, *Local::now().offset()));
    }

    // Try parsing "YYYY-MM-DD HH:MM"
    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M") {
        return Ok(DateTime::from_naive_utc_and_offset(naive_dt, *Local::now().offset()));
    }

    Err(format!("Invalid datetime format: '{}'. Expected 'YYYY-MM-DD HH:MM:SS', 'YYYY-MM-DD HH:MM', or 'YYYY-MM-DD'", datetime_str))
}

fn apply_time_filters(mut log: CrunchLog, cli: &Cli) -> CrunchLog {
    if cli.from.is_none() && cli.to.is_none() {
        return log;
    }

    let from_dt = cli.from.as_ref().and_then(|s| {
        match parse_datetime(s) {
            Ok(dt) => Some(dt),
            Err(e) => {
                eprintln!("Error parsing --from: {}", e);
                std::process::exit(1);
            }
        }
    });

    let to_dt = cli.to.as_ref().and_then(|s| {
        match parse_datetime(s) {
            Ok(dt) => Some(dt),
            Err(e) => {
                eprintln!("Error parsing --to: {}", e);
                std::process::exit(1);
            }
        }
    });

    log.filter_by_time(from_dt, to_dt);
    log
}
