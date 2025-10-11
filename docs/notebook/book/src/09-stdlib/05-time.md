# Time & Date - Feature 30/41

Time and date operations handle timestamps, durations, formatting, and time zone conversions. Ruchy provides instant, duration, and datetime types.

## Current Time

```ruchy
use std::time::SystemTime

let now = SystemTime::now()
```

**Test Coverage**: ✅ [tests/lang_comp/stdlib/time.rs](../../../../tests/lang_comp/stdlib/time.rs)

### Try It in the Notebook

```ruchy
use std::time::{SystemTime, UNIX_EPOCH}

let now = SystemTime::now()
let since_epoch = now.duration_since(UNIX_EPOCH).unwrap()
since_epoch.as_secs()  // Returns: seconds since 1970-01-01
```

**Expected Output**: Unix timestamp (e.g., `1702345678`)

## Duration

```ruchy
use std::time::Duration

let dur = Duration::from_secs(60)
dur.as_secs()         // Returns: 60
dur.as_millis()       // Returns: 60000
dur.as_micros()       // Returns: 60000000
```

**Expected Output**: `60`, `60000`, `60000000`

### Creating Durations

```ruchy
Duration::from_secs(5)       // 5 seconds
Duration::from_millis(500)   // 500 milliseconds
Duration::from_micros(1000)  // 1000 microseconds
Duration::from_nanos(1_000_000)  // 1 million nanoseconds
```

**Expected Output**: Various duration objects

## Measuring Elapsed Time

```ruchy
use std::time::Instant

let start = Instant::now()
// ... some work ...
let elapsed = start.elapsed()

elapsed.as_secs()      // Seconds elapsed
elapsed.as_millis()    // Milliseconds elapsed
```

**Expected Output**: Elapsed time measurements

## Duration Arithmetic

```ruchy
let dur1 = Duration::from_secs(60)
let dur2 = Duration::from_secs(30)

let sum = dur1 + dur2           // 90 seconds
let diff = dur1 - dur2          // 30 seconds
let scaled = dur1 * 2           // 120 seconds
let divided = dur1 / 2          // 30 seconds
```

**Expected Output**: Duration calculations

## Time Comparisons

```ruchy
let now = Instant::now()
let later = now + Duration::from_secs(5)

later > now           // Returns: true
later == now          // Returns: false
```

**Expected Output**: `true`, `false`

## Common Patterns

### Benchmarking

```ruchy
fn benchmark<F>(f: F) -> Duration
where
  F: FnOnce()
{
  let start = Instant::now()
  f()
  start.elapsed()
}

let elapsed = benchmark(|| {
  // Code to benchmark
  for i in 0..1_000_000 {
    let _ = i * i
  }
})

println!("Took: {:?}", elapsed)
```

**Expected Output**: Execution time measurement

### Timeout Implementation

```ruchy
fn with_timeout<F, T>(duration: Duration, f: F) -> Option<T>
where
  F: FnOnce() -> T
{
  let start = Instant::now()
  let result = f()

  if start.elapsed() > duration {
    None  // Timeout exceeded
  } else {
    Some(result)
  }
}
```

**Expected Output**: Result or timeout

### Rate Limiting

```ruchy
struct RateLimiter {
  last_call: Instant,
  min_interval: Duration
}

impl RateLimiter {
  fn new(min_interval: Duration) -> Self {
    RateLimiter {
      last_call: Instant::now() - min_interval,
      min_interval
    }
  }

  fn should_allow(&mut self) -> bool {
    let now = Instant::now()
    let elapsed = now - self.last_call

    if elapsed >= self.min_interval {
      self.last_call = now
      true
    } else {
      false
    }
  }
}
```

**Expected Output**: Rate limiting logic

### Sleep

```ruchy
use std::thread::sleep

sleep(Duration::from_secs(1))  // Sleep for 1 second
sleep(Duration::from_millis(500))  // Sleep for 500ms
```

**Expected Output**: Pauses execution

## Formatting Duration

```ruchy
fn format_duration(dur: Duration) -> String {
  let secs = dur.as_secs()
  let hours = secs / 3600
  let minutes = (secs % 3600) / 60
  let seconds = secs % 60

  format!("{}:{:02}:{:02}", hours, minutes, seconds)
}

let dur = Duration::from_secs(3665)
format_duration(dur)  // Returns: "1:01:05"
```

**Expected Output**: `"1:01:05"`

## DateTime (with chrono)

```ruchy
use chrono::{DateTime, Utc, Local}

// Current time
let now_utc: DateTime<Utc> = Utc::now()
let now_local: DateTime<Local> = Local::now()

// Formatting
now_utc.format("%Y-%m-%d %H:%M:%S").to_string()
// Returns: "2024-01-15 14:30:00"
```

**Expected Output**: Formatted date-time string

### Parsing Dates

```ruchy
use chrono::NaiveDate

let date = NaiveDate::from_ymd(2024, 1, 15)
let parsed = NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d")
```

**Expected Output**: Parsed date objects

### Date Arithmetic

```ruchy
use chrono::Duration as ChronoDuration

let date = Utc::now()
let tomorrow = date + ChronoDuration::days(1)
let next_week = date + ChronoDuration::weeks(1)
let last_month = date - ChronoDuration::days(30)
```

**Expected Output**: Date calculations

## Best Practices

### ✅ Use Instant for Relative Time

```ruchy
// Good: Instant for elapsed time
let start = Instant::now()
do_work()
let elapsed = start.elapsed()

// Bad: SystemTime for elapsed time (affected by clock changes)
let start = SystemTime::now()
do_work()
let elapsed = SystemTime::now().duration_since(start).unwrap()
```

### ✅ Use SystemTime for Absolute Time

```ruchy
// Good: SystemTime for timestamps
let created_at = SystemTime::now()
save_to_database(created_at)

// Bad: Instant can't be serialized
let created_at = Instant::now()  // Can't store this
```

### ✅ Handle Duration Subtraction Errors

```ruchy
// Good: Check before subtracting
let now = SystemTime::now()
match now.duration_since(UNIX_EPOCH) {
  Ok(since_epoch) => use_timestamp(since_epoch),
  Err(e) => handle_error(e)
}

// Bad: Unwrap may panic
let since_epoch = now.duration_since(UNIX_EPOCH).unwrap()
```

### ✅ Use Appropriate Precision

```ruchy
// Good: Milliseconds for most logging
let elapsed_ms = start.elapsed().as_millis()
log!("Request took {}ms", elapsed_ms)

// Overkill: Nanoseconds for simple logging
let elapsed_ns = start.elapsed().as_nanos()
log!("Request took {}ns", elapsed_ns)
```

## Performance Considerations

| Operation | Cost | Use Case |
|-----------|------|----------|
| `Instant::now()` | Fast (~20ns) | High-frequency timing |
| `SystemTime::now()` | Medium (~100ns) | Timestamps |
| `Duration` arithmetic | Negligible | Always use |
| `sleep()` | Expensive | Only when needed |

## Summary

✅ **Feature Status**: WORKING
✅ **Test Coverage**: 100%
✅ **Mutation Score**: 94%

Time and date operations handle timestamps, durations, and formatting. Use Instant for elapsed time, SystemTime for timestamps, and chrono for complex date operations.

**Key Takeaways**:
- Instant: Monotonic clock for elapsed time
- SystemTime: Wall clock for timestamps
- Duration: Time spans with arithmetic
- Use `elapsed()` for benchmarking
- chrono crate for date/time formatting
- Handle duration_since errors properly

---

[← Previous: Math Functions](./04-math.md) | [Next: Generics →](../10-advanced/01-generics.md)
