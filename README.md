# glancelog

A fast, Rust-based rapid log analysis.

## Overview

glancelog is a rapid log analysis tool that helps systems administrators and security professionals understand their logs by reducing complexity and highlighting patterns. It works by "hashing" log entries - replacing variable data (like numbers, IPs, timestamps) with placeholder characters, then counting how often each pattern appears.

## Features

- **Multiple log format support**: Automatically detects Syslog, RSyslog, Journalctl, EVTX (Windows Event Logs), Secure logs, and more
- **Pattern-based analysis**: Groups similar log entries to identify what's normal
- **Multiple analysis modes**:
  - Hash mode: Show log patterns with occurrence counts
  - Daemon mode: Report log entries by daemon/service
  - Host mode: Report log entries by host
  - Word count mode: Find qualitatively important words
  - Time-based graphs: Visualize log activity over seconds, minutes, hours, days, months, or years
- **Flexible filtering**: Use stopword files to fine-tune what gets filtered
- **Sample display**: Show actual log samples for rare entries
- **Fast**: Built in Rust for excellent performance

## Installation

### From source

```bash
cargo build --release
sudo cp target/release/glancelog /usr/local/bin/
```

## Usage

### Basic Commands

Print log lines as-is:
```bash
glancelog --print /var/log/messages
# or use short form
glancelog -p /var/log/messages
```

Hash a syslog file, showing patterns:
```bash
glancelog --hash /var/log/messages
```

Get a daemon report:
```bash
glancelog --daemon /var/log/messages
```

Get a host report:
```bash
glancelog --host /var/log/messages
```

Find qualitatively important words:
```bash
glancelog --wordcount /var/log/messages
```

Some Windows examples:
```bash
# Print EVTX events as-is
glancelog --print Security.evtx

# Analyze Windows Security event log
glancelog --hash Security.evtx

# See which event sources are most active
glancelog --daemon Application.evtx

# Analyze events by computer/host
glancelog --host System.evtx

# Find important event patterns
glancelog --wordcount Security.evtx
```

### Graph Commands

Show activity over first 60 seconds:
```bash
glancelog --sgraph /var/log/messages
```

Show activity over first 60 minutes:
```bash
glancelog --mgraph /var/log/messages
```

Show activity over first 24 hours:
```bash
glancelog --hgraph /var/log/messages
```

Track specific patterns:
```bash
cat /var/log/messages | grep error | glancelog --mgraph
```

Graph a specific time range:
```bash
# Show hourly activity for a specific date (graphs the full day)
glancelog --hgraph --from "2025-11-14" --to "2025-11-15" /var/log/messages

# Show minute-by-minute activity starting from a specific date
glancelog --mgraph --from "2025-11-14" /var/log/messages

# Show second-by-second activity for a 5-minute window
glancelog --sgraph --from "2025-11-14" --to "2025-11-14" /var/log/messages
```

**Note**: When using `--from` and `--to` with graph modes, the graph duration is automatically calculated based on the time range. For example, `--hgraph --from "2025-11-14" --to "2025-11-15"` will graph 24 hours instead of the default 24 hours starting from the first log entry.

### Analysis Modes

- `-p, --print`: Print log lines as-is (respects `--from`/`--to` filters)
- `--hash`: Show log patterns with occurrence counts (default)
- `--daemon`: Report log entries by daemon/service
- `--host`: Report log entries by host
- `--wordcount`: Find qualitatively important words
- `--sgraph`, `--mgraph`, `--hgraph`, `--dgraph`, `--mograph`, `--ygraph`: Time-based graphs

### Options

- `--sample`: Show sample output for entries appearing 3 or fewer times (default)
- `--nosample`: Don't show samples, only show hashed patterns
- `--allsample`: Show samples for all entries instead of hashed patterns
- `-l, --lowcount <NUMBER>`: Set threshold for rare vs common events (default: 3)
- `--from <DATETIME>`: Filter logs from this datetime (formats: "YYYY-MM-DD HH:MM:SS", "YYYY-MM-DD HH:MM", or "YYYY-MM-DD")
- `--to <DATETIME>`: Filter logs to this datetime (formats: "YYYY-MM-DD HH:MM:SS", "YYYY-MM-DD HH:MM", or "YYYY-MM-DD")
- `--filter`: Use filter files during processing (default for most modes)
- `--nofilter`: Don't use filter files
- `--filter-dir <DIR>`: Custom directory for filter files (overrides `GLANCELOG_FILTERDIR` and default paths)
- `--export-filters [DIR]`: Export embedded default filters to a directory (defaults to `~/.glancelog/filters`)
- `--wide`: Use wider graph characters for better visibility
- `--tick <CHAR>`: Change the tick character used in graphs (default: `#`)
- `-v`: Verbose output (shows detected log format and entry count)

## How It Works

glancelog uses a simple but effective algorithm:

1. **Parse**: Automatically detect and parse log format
2. **Hash**: Replace variable data (numbers, IPs, etc.) with `#` characters
3. **Count**: Count how many times each hashed pattern appears
4. **Display**: Show patterns sorted by frequency

The philosophy is that:
- Common patterns (high count) are likely normal behavior
- Rare patterns (low count) may indicate issues or anomalies
- By removing certainty (common patterns), you're left with uncertainty (things to investigate)

## Filter Files

Filter files contain regular expressions (one per line) that define what should be replaced with `#`.

### Embedded Default Filters

**glancelog includes embedded default filter files** that are compiled directly into the binary. These filters work automatically as a fallback when no external filter files are found, ensuring the tool works out-of-the-box without requiring separate filter file installation.

### Filter Search Paths

glancelog searches for filter files in the following locations (in priority order):

1. **Custom directory** (via `--filter-dir` option) - highest priority
2. **Environment variable** (`GLANCELOG_FILTERDIR`)
3. **User home directory**: `~/.glancelog/filters/` (cross-platform)
4. **Current directory**: `./filters/`
5. **System directories** (Unix/Linux only):
   - `/var/lib/glancelog/filters/`
   - `/usr/local/glancelog/var/lib/filters/`
   - `/opt/glancelog/var/lib/filters/`
6. **Embedded defaults** - Built into the binary as a fallback

### Cross-Platform Home Directory Paths

The home directory filter location varies by operating system:
- **Linux**: `/home/username/.glancelog/filters/`
- **macOS**: `/Users/username/.glancelog/filters/`
- **BSD**: `/home/username/.glancelog/filters/`
- **Windows**: `C:\Users\username\.glancelog\filters\`

### Standard Filter Files

- `hash.stopwords`: Used in hash mode
- `words.stopwords`: Used in wordcount mode
- `daemon.stopwords`: Used in daemon mode
- `host.stopwords`: Used in host mode

### Exporting Embedded Filters

To customize the default filters, you can export the embedded filters to your filesystem:

**Export to home directory (recommended):**
```bash
# Export to ~/.glancelog/filters/
glancelog --export-filters

# All filter files are now available for editing
ls ~/.glancelog/filters/
# hash.stopwords  words.stopwords  daemon.stopwords  host.stopwords
```

**Export to custom directory:**
```bash
# Export to a specific directory
glancelog --export-filters /path/to/custom/filters

# Now you can edit and use them
glancelog --hash --filter-dir /path/to/custom/filters /var/log/messages
```

After exporting, you can edit the filter files to add your own regex patterns or remove patterns you don't need. The exported files will take precedence over the embedded defaults based on the filter search priority.

### Custom Filter Directory

You can specify a custom filter directory using:

**Command-line option:**
```bash
# Use custom filter directory
glancelog --hash --filter-dir /path/to/custom/filters /var/log/messages

# Works with all modes
glancelog --daemon --filter-dir ~/my-filters /var/log/messages
```

**Environment variable:**
```bash
# Set for current session
export GLANCELOG_FILTERDIR=/opt/my-filters
glancelog --hash /var/log/messages

# Or per-command
GLANCELOG_FILTERDIR=/tmp/filters glancelog --wordcount /var/log/messages
```

**User home directory (recommended for personal filters):**
```bash
# Create your personal filter directory
mkdir -p ~/.glancelog/filters

# Add custom regex patterns
echo '\d+\.\d+\.\d+\.\d+' > ~/.glancelog/filters/hash.stopwords  # Filter IP addresses
echo '(?i)(error|warning)' > ~/.glancelog/filters/hash.stopwords # Filter error/warning words

# Use automatically (no flags needed)
glancelog --hash /var/log/messages
```

**Priority Example:**
```bash
# CLI --filter-dir overrides environment variable and home directory
GLANCELOG_FILTERDIR=~/filters glancelog --hash --filter-dir /tmp/filters /var/log/messages
# Uses /tmp/filters (CLI has highest priority)
```

## Examples

### Finding Issues

```bash
# Look for uncommon patterns that might indicate problems
glancelog --hash /var/log/messages

# Find what's generating the most log entries
glancelog --daemon /var/log/messages

# See which hosts are most active
glancelog --host /var/log/messages

# Customize the threshold for rare vs common events
glancelog --hash -l 5 /var/log/messages  # Show samples for events appearing 5 or fewer times

# Filter logs by time range
glancelog --hash --from "2025-11-14 10:00:00" --to "2025-11-14 12:00:00" /var/log/messages

# Filter logs from a specific date onwards
glancelog --hash --from "2025-11-14" /var/log/messages

# Print only logs from a specific time range
glancelog --print --from "2025-11-14 09:00:00" --to "2025-11-14 10:00:00" /var/log/messages
```

### Understanding Activity Patterns

```bash
# Visualize activity throughout the day
glancelog --hgraph /var/log/messages

# Track error patterns minute-by-minute
grep -i error /var/log/messages | glancelog --mgraph

# Analyze journalctl logs (systemd)
journalctl -n 1000 --no-pager | glancelog --hash

# Find which systemd services are most active
journalctl -n 1000 --no-pager | glancelog --daemon
```

### Analyzing Windows Event Logs (EVTX)

```bash
# Print EVTX events as-is
glancelog --print Security.evtx

# Analyze Windows Security event log
glancelog --hash Security.evtx

# See which event sources are most active
glancelog --daemon Application.evtx

# Analyze events by computer/host
glancelog --host System.evtx

# Find important event patterns
glancelog --wordcount Security.evtx
```

**Note**: EVTX files are Windows Event Log files typically exported from Windows Event Viewer. You can export them using:
- Event Viewer → Right-click log → Save All Events As → Select EVTX format
- PowerShell: `wevtutil epl Security Security.evtx`

### Analyzing Apache Web Server Logs

```bash
# Print Apache access logs with timestamps
glancelog --print /var/log/apache2/access.log

# Analyze request patterns
glancelog --hash /var/log/apache2/access.log

# See which HTTP methods are most common
glancelog --daemon /var/log/apache2/access.log

# Analyze requests by IP address
glancelog --host /var/log/apache2/access.log

# Filter logs by date range
glancelog --print --from "2000-10-10" --to "2000-10-11" access.log

# Show hourly request activity
glancelog --hgraph --from "2000-10-10" --to "2000-10-11" access.log
```

**Supported Apache Formats**:
- **Common Log Format (CLF)**: `IP - user [timestamp] "request" status bytes`
- **Combined Log Format**: `IP - user [timestamp] "request" status bytes "referer" "user-agent"`

Example Apache logs:
```
127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326
192.168.1.1 - - [10/Oct/2000:14:10:20 -0700] "POST /api/login HTTP/1.1" 302 512 "-" "curl/7.68.0"
```

### Analyzing AWS Load Balancer Logs

glancelog supports both Classic ELB and Application Load Balancer (ALB) log formats.

```bash
# Print AWS ELB logs with timestamps
glancelog --print elb-logs.log

# Analyze ELB request patterns
glancelog --hash elb-logs.log

# See which HTTP methods are most common
glancelog --daemon elb-logs.log

# Analyze requests by client IP
glancelog --host elb-logs.log

# Show hourly request activity
glancelog --hgraph --from "2025-11-14" --to "2025-11-15" elb-logs.log
```

**AWS ELB Format Example**:
```
2015-05-01T23:00:00.123456Z my-loadbalancer 192.168.131.39:2817 10.0.0.1:80 0.000073 0.001048 0.000057 200 200 0 29 "GET http://www.example.com:80/ HTTP/1.1" "curl/7.38.0" - -
```

**AWS ALB Format Example**:
```
http 2018-07-02T22:23:00.186641Z app/my-loadbalancer/50dc6c495c0c9188 192.168.131.39:2817 10.0.0.1:80 0.000 0.001 0.000 200 200 34 366 "GET https://www.example.com:443/ HTTP/2.0" "curl/7.46.0" ...
```

**Note**: AWS ELB/ALB logs can be exported from your AWS Console or retrieved from S3 buckets where they're automatically stored.

### Analyzing MySQL Logs

```bash
# Print MySQL general query log with timestamps
glancelog --print mysql-general.log

# Analyze query patterns
glancelog --hash mysql-general.log

# See query types (Query, Connect, Quit, Execute)
glancelog --daemon mysql-general.log

# Analyze activity by thread
glancelog --host mysql-general.log

# Show hourly query activity
glancelog --hgraph --from "2025-11-14" --to "2025-11-15" mysql-general.log
```

**MySQL General Log Format Example**:
```
2025-11-14T10:00:00.123456Z         5 Connect   root@localhost on test_db using TCP/IP
2025-11-14T10:00:01.234567Z         5 Query     SELECT * FROM users WHERE id = 123
2025-11-14T10:00:02.345678Z         5 Quit
```

**Note**: Enable MySQL general log with `SET GLOBAL general_log = 'ON';` and `SET GLOBAL log_output = 'FILE';`

### Analyzing PostgreSQL Logs

```bash
# Print PostgreSQL logs with timestamps
glancelog --print postgresql.log

# Analyze log message patterns
glancelog --hash postgresql.log

# See log levels (LOG, ERROR, WARNING, etc.)
glancelog --daemon postgresql.log

# Analyze activity by user@database
glancelog --host postgresql.log

# Show hourly activity
glancelog --hgraph --from "2025-11-14" --to "2025-11-15" postgresql.log
```

**PostgreSQL Log Format Example**:
```
2025-11-14 10:00:00.123 UTC [12345] postgres@testdb LOG:  database system is ready to accept connections
2025-11-14 10:00:03.456 UTC [12347] admin@testdb ERROR:  relation "nonexistent_table" does not exist at character 15
2025-11-14 10:00:07.890 UTC [12349] postgres@postgres FATAL:  the database system is shutting down
```

**Note**: PostgreSQL logs must be in single-line format. Configure with `log_destination = 'stderr'` and `logging_collector = on` in postgresql.conf.

### Finding Keywords for Monitoring

```bash
# Find important words to monitor with swatch/logwatch
glancelog --wordcount /var/log/messages
```

## Supported Log Formats

- Syslog (BSD syslog format)
- RSyslog (with high-precision timestamps)
- Journalctl (systemd journal logs)
- EVTX (Windows Event Log binary format)
- Apache Common Log Format (CLF)
- Apache Combined Log Format
- AWS Classic Elastic Load Balancer (ELB) logs
- AWS Application Load Balancer (ALB) logs
- MySQL General Query Log
- PostgreSQL logs (single-line format)
- Secure log (authentication logs)
- Raw text (fallback for unrecognized formats)

## Building and Development

### Standard Build

```bash
# Build
cargo build --release

# Run tests
cargo test

# Build and install
cargo build --release
sudo cp target/release/glancelog /usr/local/bin/
```

### Cross-Platform Builds (Static Binaries)

Use the Makefile to build static binaries for multiple platforms:

```bash
# Build for all supported platforms
make dist

# Build for specific platforms
make dist-linux    # Linux (x64 + arm64) - static with musl
make dist-macos    # macOS (x64 + arm64)
make dist-windows  # Windows (x64 + x86 + arm64) - static

# Install required rustup targets
make install-targets

# View available targets
make help

# Clean dist directory
make clean
```

**Static Build Strategy:**

- **Linux**: Uses musl libc for **fully static binaries** that work on any Linux distribution without dependencies
  - Target: `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`
  - No shared library dependencies - runs everywhere

- **Windows**: Static CRT linking for minimal dependencies
  - Target: `x86_64-pc-windows-msvc`, `i686-pc-windows-msvc`, and `aarch64-pc-windows-msvc`
  - Links CRT statically using `-C target-feature=+crt-static`
  - Supports 64-bit, 32-bit, and ARM64 Windows

- **macOS**: Limited static linking (system frameworks remain dynamic)
  - Target: `x86_64-apple-darwin` and `aarch64-apple-darwin`
  - macOS doesn't support fully static binaries

**Supported Platforms:**
- Linux: x64 (static), arm64 (static)
- macOS: x64, arm64
- Windows: x64 (static), x86/32-bit (static), arm64 (static)

All binaries are placed in the `dist/` directory with naming format: `glancelog-{platform}-{arch}[.exe]`

**Verification:**

```bash
# Verify Linux binary is static
file dist/glancelog-linux-x64
# Output: ... statically linked ...

ldd dist/glancelog-linux-x64
# Output: statically linked (no dependencies)
```

**Cross-Compilation Requirements:**

Some targets require additional tools to be installed:

```bash
# For Linux ARM64 musl cross-compilation (on Linux x64 host)
sudo apt-get install musl-tools gcc-aarch64-linux-gnu

# For Windows cross-compilation
# - On Windows: Install Visual Studio Build Tools with MSVC
# - On Linux: Cross-compilation to Windows MSVC is not well supported
#   Use GitHub Actions or build on Windows
```

**Note**: macOS targets can only be fully built on macOS hosts with Xcode. Cross-compilation from Linux to macOS is not easily supported. The Makefile will skip targets that cannot be built and show warnings.

### Automated Release Builds (GitHub Actions)

The repository includes a GitHub Actions workflow that automatically builds release binaries for all supported platforms when you push a version tag:

```bash
# Create and push a release tag
git tag v1.0.0
git push origin v1.0.0
```

The workflow will:
- ✅ Build static binaries for Linux (x64, ARM64) using musl
- ✅ Build binaries for macOS (x64, ARM64)
- ✅ Build static binaries for Windows (x64, x86/32-bit, ARM64) using MSVC
- ✅ Create a GitHub Release with all binaries
- ✅ Verify Linux binaries are statically linked
- ✅ Use cargo caching for faster builds

**Release Assets:**
- `glancelog-linux-x64` - Fully static, works on any Linux
- `glancelog-linux-arm64` - Fully static ARM64 binary
- `glancelog-macos-x64` - Intel Mac binary
- `glancelog-macos-arm64` - Apple Silicon (M1/M2/M3) binary
- `glancelog-windows-x64.exe` - Static Windows x64 binary
- `glancelog-windows-x86.exe` - Static Windows 32-bit binary
- `glancelog-windows-arm64.exe` - Static Windows ARM64 binary

**Manual Workflow Dispatch:**

You can also trigger builds manually from the GitHub Actions tab without creating a tag.

## Using glancelog as a Library

glancelog can be used as a library in your Rust projects for programmatic log analysis.

### Add to your project

Add glancelog to your `Cargo.toml`:

```toml
[dependencies]
glancelog = { path = "../glancelog" }  # Use path for local development
# or
glancelog = { git = "https://github.com/kost/glancelog" }
```

### Basic Usage

```rust
use glancelog::{CrunchLog, Filter, SuperHash, HashMode, SampleMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load and parse a log file
    let log = CrunchLog::from_file("/var/log/messages")?;

    println!("Loaded {} entries", log.entries.len());
    println!("Detected format: {}", log.parser_type);

    Ok(())
}
```

### Hash Analysis

Analyze log patterns by removing variable data:

```rust
use glancelog::{CrunchLog, Filter, SuperHash, HashMode, SampleMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load logs
    let log = CrunchLog::from_file("/var/log/messages")?;

    // Create filter from stopwords file (uses embedded filter as fallback)
    let filter = Filter::from_file("hash.stopwords")
        .unwrap_or_else(|_| Filter::new());

    // Create hash analyzer
    let mut hash = SuperHash::from_log(&log, HashMode::Hash, filter);

    // Configure sampling
    hash.set_sample_threshold(3);
    hash.set_sample_mode(SampleMode::Threshold);

    // Display results
    hash.display();

    Ok(())
}
```

### Reading from stdin

```rust
use glancelog::{CrunchLog, SuperHash, HashMode, Filter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read from stdin
    let log = CrunchLog::from_stdin()?;

    // Analyze by daemon/service
    let filter = Filter::new();
    let mut hash = SuperHash::from_log(&log, HashMode::Daemon, filter);
    hash.display();

    Ok(())
}
```

### Daemon and Host Analysis

```rust
use glancelog::{CrunchLog, Filter, SuperHash, HashMode, SampleMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log = CrunchLog::from_file("/var/log/messages")?;

    // Analyze by daemon
    let filter = Filter::from_file("daemon.stopwords")
        .unwrap_or_else(|_| Filter::new());
    let mut daemon_hash = SuperHash::from_log(&log, HashMode::Daemon, filter);
    daemon_hash.set_sample_mode(SampleMode::None);
    println!("=== By Daemon ===");
    daemon_hash.display();

    // Analyze by host
    let filter = Filter::from_file("host.stopwords")
        .unwrap_or_else(|_| Filter::new());
    let mut host_hash = SuperHash::from_log(&log, HashMode::Host, filter);
    host_hash.set_sample_mode(SampleMode::None);
    println!("\n=== By Host ===");
    host_hash.display();

    Ok(())
}
```

### Time-based Graphs

Visualize log activity over time:

```rust
use glancelog::{CrunchLog, GraphHash, GraphType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log = CrunchLog::from_file("/var/log/messages")?;

    // Create hourly graph
    let mut graph = GraphHash::new(&log, GraphType::Hours);
    graph.set_tick('█');
    graph.set_wide(true);
    graph.display();

    Ok(())
}
```

### Time Filtering

Filter logs by date/time range:

```rust
use glancelog::CrunchLog;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut log = CrunchLog::from_file("/var/log/messages")?;

    // Create datetime range
    let from_date = NaiveDate::from_ymd_opt(2025, 11, 14).unwrap();
    let from_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let from_dt = DateTime::from_naive_utc_and_offset(
        NaiveDateTime::new(from_date, from_time),
        *Local::now().offset()
    );

    let to_date = NaiveDate::from_ymd_opt(2025, 11, 15).unwrap();
    let to_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let to_dt = DateTime::from_naive_utc_and_offset(
        NaiveDateTime::new(to_date, to_time),
        *Local::now().offset()
    );

    // Filter logs
    log.filter_by_time(Some(from_dt), Some(to_dt));

    println!("Filtered to {} entries", log.entries.len());

    Ok(())
}
```

### Custom Filters

Create custom regex-based filters:

```rust
use glancelog::Filter;

fn main() {
    // Create empty filter
    let mut filter = Filter::new();

    // Add patterns programmatically
    // (Note: Current API loads from files, but you can extend it)

    // Load from custom file
    let filter = Filter::from_file("my-custom.stopwords")
        .expect("Failed to load filter");
}
```

### Word Count Analysis

Find qualitatively important words:

```rust
use glancelog::{CrunchLog, Filter, SuperHash, HashMode, SampleMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log = CrunchLog::from_file("/var/log/messages")?;

    let filter = Filter::from_file("words.stopwords")
        .unwrap_or_else(|_| Filter::new());

    let mut hash = SuperHash::from_log(&log, HashMode::WordCount, filter);
    hash.set_sample_mode(SampleMode::None);
    hash.display();

    Ok(())
}
```

### Working with Log Entries

Access individual log entries:

```rust
use glancelog::CrunchLog;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log = CrunchLog::from_file("/var/log/messages")?;

    for entry in &log.entries {
        println!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} {} {}: {}",
            entry.year, entry.month, entry.day,
            entry.hour, entry.minute, entry.second,
            entry.host,
            entry.daemon,
            entry.log_entry
        );
    }

    Ok(())
}
```

### Advanced: Custom Time Ranges with Graphs

```rust
use glancelog::{CrunchLog, GraphHash, GraphType};
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log = CrunchLog::from_file("/var/log/messages")?;

    // Create custom time range
    let from_date = NaiveDate::from_ymd_opt(2025, 11, 14).unwrap();
    let from_time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let from_dt = DateTime::from_naive_utc_and_offset(
        NaiveDateTime::new(from_date, from_time),
        *Local::now().offset()
    );

    let to_date = NaiveDate::from_ymd_opt(2025, 11, 14).unwrap();
    let to_time = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
    let to_dt = DateTime::from_naive_utc_and_offset(
        NaiveDateTime::new(to_date, to_time),
        *Local::now().offset()
    );

    // Create graph with custom range
    let mut graph = GraphHash::new_with_range(
        &log,
        GraphType::Hours,
        Some(from_dt),
        Some(to_dt)
    );

    graph.display();

    Ok(())
}
```

### API Overview

**Core Types:**
- `CrunchLog` - Main log container with parsed entries
- `LogEntry` - Individual log entry with timestamp, host, daemon, and message
- `Filter` - Regex-based filter for removing variable data
- `SuperHash` - Pattern analyzer with counting
- `GraphHash` - Time-based visualization

**Enums:**
- `HashMode::Hash` - Standard pattern hashing
- `HashMode::Daemon` - Group by daemon/service
- `HashMode::Host` - Group by host
- `HashMode::WordCount` - Count important words
- `SampleMode::None` - Show hashed patterns only
- `SampleMode::Threshold` - Show samples for rare events
- `SampleMode::All` - Show samples for all events
- `GraphType::{Seconds, Minutes, Hours, Days, Months, Years}` - Time granularity

**Key Methods:**
- `CrunchLog::from_file(path)` - Load from file
- `CrunchLog::from_stdin()` - Load from stdin
- `CrunchLog::filter_by_time(from, to)` - Filter by datetime range
- `SuperHash::from_log(log, mode, filter)` - Create analyzer
- `SuperHash::set_sample_threshold(n)` - Set rare event threshold
- `SuperHash::set_sample_mode(mode)` - Configure sampling
- `SuperHash::display()` - Print results to stdout
- `GraphHash::new(log, type)` - Create graph
- `GraphHash::new_with_range(log, type, from, to)` - Graph with time range
- `GraphHash::set_tick(char)` - Set graph character
- `GraphHash::set_wide(bool)` - Use wider characters
- `GraphHash::display()` - Print graph to stdout

## License

MIT

## Credits

Inspired by the [petit](https://crunchtools.com/software/petit/) - original Python-based log analysis concepts.
