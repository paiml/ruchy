# Sub-spec: HTTP-002 — Server Management and Benchmarking

**Parent:** [HTTP-002-advanced-http-features.md](../HTTP-002-advanced-http-features.md) Sections A-B

---

# HTTP-002: Advanced HTTP Features Sprint

**Sprint**: HTTP-002
**Version Target**: v3.100.0
**Dependencies**: HTTP-001 (HTTP server MVP - COMPLETE)
**Status**: Planning
**Created**: 2025-10-20

## Sprint Goals

Enhance Ruchy's HTTP capabilities with production-ready server management, benchmarking, and web scraping features.

## Features

### HTTP-002-A: Server Process Management (PID File Support)

**Priority**: HIGH
**Complexity**: Low
**Rationale**: Fixes zsh bug documented in `../interactive.paiml.com/wasm/ruchy/BUG_ZSH_COMMAND_EXECUTION.md`

#### Problem Statement

Current `ruchy serve` command has no built-in process management, causing:
- Zsh command chain failures with background execution
- Manual PID tracking required
- Difficulty automating server lifecycle in CI/CD
- Poor developer experience for server restarts

#### Solution: PID File Management

Add `--pid-file` option to `ruchy serve` command:

```bash
# Automatic process management:
ruchy serve dist --port 8080 --pid-file /tmp/ruchy.pid

# Behavior:
# 1. Check if PID file exists
# 2. If exists and process running → kill old process, wait 1s
# 3. Start new server
# 4. Write current PID to file
# 5. Clean up PID file on graceful shutdown
```

#### Implementation Details

**File**: `src/bin/ruchy.rs`
```rust
/// Serve static files over HTTP (HTTP-001)
Serve {
    // ... existing fields ...

    /// PID file for automatic process management
    #[arg(long)]
    pid_file: Option<PathBuf>,
}
```

**File**: `src/server/mod.rs` (new module)
```rust
use std::fs;
use std::path::Path;
use std::process;
use std::thread;
use std::time::Duration;

pub struct PidFile {
    path: PathBuf,
}

impl PidFile {
    pub fn new(path: PathBuf) -> Result<Self> {
        // Check if PID file exists
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(pid) = contents.trim().parse::<u32>() {
                // Check if process is running
                if process_is_running(pid) {
                    // Kill old process
                    kill_process(pid)?;
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }

        // Write current PID
        fs::write(&path, process::id().to_string())?;

        Ok(Self { path })
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn process_is_running(pid: u32) -> bool {
    // Platform-specific: check if process exists
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        kill(Pid::from_raw(pid as i32), Signal::SIGNULL).is_ok()
    }

    #[cfg(windows)]
    {
        // Windows implementation
        unimplemented!("Windows PID checking not yet implemented")
    }
}

fn kill_process(pid: u32) -> Result<()> {
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        kill(Pid::from_raw(pid as i32), Signal::SIGTERM)?;
    }

    Ok(())
}
```

#### Test Cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_file_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pid_path = temp_dir.path().join("test.pid");

        let _pid_file = PidFile::new(pid_path.clone()).unwrap();

        // PID file should exist
        assert!(pid_path.exists());

        // Should contain current process ID
        let contents = fs::read_to_string(&pid_path).unwrap();
        assert_eq!(contents, process::id().to_string());
    }

    #[test]
    fn test_pid_file_cleanup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pid_path = temp_dir.path().join("test.pid");

        {
            let _pid_file = PidFile::new(pid_path.clone()).unwrap();
            assert!(pid_path.exists());
        } // PidFile dropped here

        // PID file should be cleaned up
        assert!(!pid_path.exists());
    }

    #[test]
    fn test_pid_file_replaces_stale() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pid_path = temp_dir.path().join("test.pid");

        // Write stale PID (non-existent process)
        fs::write(&pid_path, "999999").unwrap();

        let _pid_file = PidFile::new(pid_path.clone()).unwrap();

        // Should have replaced with current PID
        let contents = fs::read_to_string(&pid_path).unwrap();
        assert_eq!(contents, process::id().to_string());
    }
}
```

#### Dependencies

Add to `Cargo.toml`:
```toml
[target.'cfg(unix)'.dependencies]
nix = { version = "0.29", features = ["signal"] }

[dev-dependencies]
tempfile = "3.8"
```

### HTTP-002-B: Benchmarking Command

**Priority**: MEDIUM
**Complexity**: Medium
**Rationale**: ApacheBench-style tool for performance testing

#### Feature Description

Add `ruchy bench` command supporting:
- HTTP/HTTPS endpoints
- WASM module benchmarking
- CLI command benchmarking
- Comprehensive performance metrics

#### Command Syntax

```bash
# HTTP benchmarking (like ApacheBench)
ruchy bench http https://api.example.com/endpoint \
  --requests 1000 \
  --concurrency 10 \
  --method POST \
  --data '{"key": "value"}' \
  --header "Content-Type: application/json"

# WASM module benchmarking
ruchy bench wasm module.wasm \
  --function "process_data" \
  --input "test_input.json" \
  --iterations 10000

# CLI command benchmarking
ruchy bench cli "ruchy run script.ruchy" \
  --iterations 100 \
  --warmup 10

# Output similar to ApacheBench:
# Requests per second:    1234.56 [#/sec] (mean)
# Time per request:       8.100 [ms] (mean)
# Transfer rate:          456.78 [Kbytes/sec] received
#
# Connection Times (ms)
#               min  mean[+/-sd] median   max
# Total:         5    8   1.2      7      15
#
# Percentage of requests served within a certain time (ms)
#   50%      7
#   66%      8
#   75%      9
#   80%     10
#   90%     11
#   95%     12
#   98%     13
#   99%     14
#  100%     15 (longest request)
```

#### Implementation Structure

**File**: `src/bin/ruchy.rs`
```rust
/// Benchmark HTTP endpoints, WASM modules, or CLI commands
Bench {
    #[command(subcommand)]
    target: BenchTarget,
}

#[derive(Subcommand)]
enum BenchTarget {
    /// Benchmark HTTP/HTTPS endpoint
    Http {
        url: String,
        #[arg(short = 'n', long, default_value = "100")]
        requests: usize,
        #[arg(short, long, default_value = "1")]
        concurrency: usize,
        #[arg(short, long, default_value = "GET")]
        method: String,
        #[arg(short, long)]
        data: Option<String>,
        #[arg(short = 'H', long)]
        header: Vec<String>,
    },
    /// Benchmark WASM module
    Wasm {
        module: PathBuf,
        #[arg(short, long)]
        function: String,
        #[arg(short, long)]
        input: Option<PathBuf>,
        #[arg(short = 'n', long, default_value = "1000")]
        iterations: usize,
    },
    /// Benchmark CLI command
    Cli {
        command: String,
        #[arg(short = 'n', long, default_value = "10")]
        iterations: usize,
        #[arg(short, long, default_value = "3")]
        warmup: usize,
    },
}
```

**File**: `src/bench/mod.rs` (new module)
```rust
pub mod http;
pub mod wasm;
pub mod cli;
pub mod stats;

use std::time::{Duration, Instant};

pub struct BenchmarkResults {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub total_duration: Duration,
    pub request_times: Vec<Duration>,
}

impl BenchmarkResults {
    pub fn requests_per_second(&self) -> f64 {
        self.total_requests as f64 / self.total_duration.as_secs_f64()
    }

    pub fn mean_time(&self) -> Duration {
        let sum: Duration = self.request_times.iter().sum();
        sum / self.request_times.len() as u32
    }

    pub fn percentile(&self, p: f64) -> Duration {
        let mut sorted = self.request_times.clone();
        sorted.sort();
        let index = ((p / 100.0) * sorted.len() as f64) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    pub fn print_summary(&self) {
        println!("Requests per second:    {:.2} [#/sec] (mean)", self.requests_per_second());
        println!("Time per request:       {:.3} [ms] (mean)",
                 self.mean_time().as_secs_f64() * 1000.0);

        println!("\nPercentage of requests served within a certain time (ms)");
        for p in &[50, 66, 75, 80, 90, 95, 98, 99, 100] {
            let time = self.percentile(*p as f64);
            println!("  {:3}%  {:6.0}", p, time.as_secs_f64() * 1000.0);
        }
    }
}
```

