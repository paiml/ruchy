# Sub-spec: System Operations — Libraries and Advanced Features

**Parent:** [systems-operations-io-spec.md](../systems-operations-io-spec.md) Sections 4-15

---

## 4. Path Operations

**Current Status**: ❌ Not Implemented  
**Priority**: HIGH

```ruchy
mod std::path {
    struct Path {
        fn new(s: String) -> Path
        fn parent() -> Option<Path>
        fn file_name() -> Option<String>
        fn file_stem() -> Option<String>
        fn extension() -> Option<String>
        fn join(path: Path) -> Path
        fn with_file_name(name: String) -> Path
        fn with_extension(ext: String) -> Path
        fn exists() -> bool
        fn is_absolute() -> bool
        fn is_relative() -> bool
        fn to_str() -> Option<String>
        fn canonicalize() -> Result<PathBuf, Error>
    }
    
    fn join(parts: [String]) -> PathBuf
    fn dirname(path: String) -> String
    fn basename(path: String) -> String
    fn resolve(path: String) -> String
    fn relative(from: String, to: String) -> String
    fn is_absolute(path: String) -> bool
}
```

## 5. Time and Date

**Current Status**: ❌ Not Implemented  
**Priority**: MEDIUM

```ruchy
mod std::time {
    struct SystemTime {
        fn now() -> SystemTime
        fn duration_since(earlier: SystemTime) -> Result<Duration, Error>
        fn elapsed() -> Result<Duration, Error>
    }
    
    struct Duration {
        fn from_secs(secs: u64) -> Duration
        fn from_millis(millis: u64) -> Duration
        fn from_micros(micros: u64) -> Duration
        fn from_nanos(nanos: u64) -> Duration
        fn as_secs() -> u64
        fn as_millis() -> u128
        fn as_micros() -> u128
        fn as_nanos() -> u128
    }
    
    struct Instant {
        fn now() -> Instant
        fn duration_since(earlier: Instant) -> Duration
        fn elapsed() -> Duration
    }
    
    // Date/time formatting
    fn format_rfc3339(time: SystemTime) -> String
    fn parse_rfc3339(s: String) -> Result<SystemTime, Error>
}
```

## 6. JSON and Serialization

**Current Status**: ❌ Not Implemented  
**Priority**: HIGH

```ruchy
mod std::json {
    fn parse(s: String) -> Result<JsonValue, Error>
    fn stringify(value: JsonValue) -> String
    fn stringify_pretty(value: JsonValue, indent: usize) -> String
    
    // Derive macros
    #[derive(Serialize, Deserialize)]
    struct Config {
        name: String,
        value: i32,
    }
    
    // Manual serialization
    fn to_json<T: Serialize>(value: T) -> Result<String, Error>
    fn from_json<T: Deserialize>(s: String) -> Result<T, Error>
}
```

## 7. Testing Framework

**Current Status**: ⚠️ Basic Implementation  
**Priority**: HIGH

```ruchy
mod std::test {
    // Test macros
    #[test]
    fn test_addition() {
        assert_eq!(2 + 2, 4)
    }
    
    #[test]
    #[should_panic]
    fn test_panic() {
        panic!("expected panic")
    }
    
    #[test]
    #[ignore]
    fn expensive_test() {
        // Skipped unless --ignored flag
    }
    
    // Assertions
    fn assert!(condition: bool, msg: String)
    fn assert_eq!(left: T, right: T, msg: String)
    fn assert_ne!(left: T, right: T, msg: String)
    fn assert_gt!(left: T, right: T, msg: String)
    fn assert_lt!(left: T, right: T, msg: String)
    fn assert_contains!(haystack: String, needle: String)
    fn assert_throws!(expr: () -> T, error_type: Type)
    
    // Property-based testing
    mod property {
        fn check<T>(name: String, gen: Generator<T>, prop: fn(T) -> bool)
        fn check_async<T>(name: String, gen: Generator<T>, prop: async fn(T) -> bool)
        
        // Generators
        fn arbitrary<T>() -> Generator<T>
        fn integer(min: i64, max: i64) -> Generator<i64>
        fn float(min: f64, max: f64) -> Generator<f64>
        fn string(min_len: usize, max_len: usize) -> Generator<String>
        fn array<T>(gen: Generator<T>, min_len: usize, max_len: usize) -> Generator<[T]>
        fn one_of<T>(values: [T]) -> Generator<T>
    }
    
    // Mocking
    mod mock {
        fn command(name: String) -> MockCommand
        fn file_system() -> MockFileSystem
        
        struct MockCommand {
            fn expect_args(args: [String]) -> MockCommand
            fn returns_output(stdout: String, stderr: String, code: i32) -> MockCommand
        }
    }
}
```

## 8. Logging and Debugging

**Current Status**: ⚠️ Basic println only  
**Priority**: HIGH

```ruchy
mod std::log {
    enum Level {
        Trace,
        Debug,
        Info,
        Warn,
        Error,
    }
    
    fn trace(msg: String)
    fn debug(msg: String)
    fn info(msg: String)
    fn warn(msg: String)
    fn error(msg: String)
    
    struct Logger {
        fn new(name: String) -> Logger
        fn level(level: Level) -> Logger
        fn child(name: String) -> Logger
        fn with_context(key: String, value: Any) -> Logger
        
        fn trace(&self, msg: String)
        fn debug(&self, msg: String)
        fn info(&self, msg: String)
        fn warn(&self, msg: String)
        fn error(&self, msg: String)
    }
    
    // Debug printing
    fn dbg!(value: T) -> T  // Prints and returns value
}
```

## 9. Regular Expressions

**Current Status**: ❌ Not Implemented  
**Priority**: MEDIUM

```ruchy
mod std::regex {
    struct Regex {
        fn new(pattern: String) -> Result<Regex, Error>
        fn is_match(&self, text: String) -> bool
        fn find(&self, text: String) -> Option<Match>
        fn find_all(&self, text: String) -> [Match]
        fn captures(&self, text: String) -> Option<Captures>
        fn replace(&self, text: String, rep: String) -> String
        fn replace_all(&self, text: String, rep: String) -> String
        fn split(&self, text: String) -> [String]
    }
    
    struct Match {
        fn start() -> usize
        fn end() -> usize
        fn as_str() -> String
    }
    
    struct Captures {
        fn get(index: usize) -> Option<Match>
        fn name(name: String) -> Option<Match>
        fn len() -> usize
    }
}
```

## 10. Async/Await Runtime

**Current Status**: ⚠️ Parser support, no runtime  
**Priority**: HIGH

```ruchy
// Async functions
async fn fetch_data(url: String) -> Result<String, Error> {
    let response = await fetch(url)?;
    await response.text()
}

// Async runtime
mod std::async {
    fn block_on<F: Future>(future: F) -> F::Output
    fn spawn<F: Future>(future: F) -> JoinHandle<F::Output>
    fn sleep(duration: Duration) -> impl Future<Output = ()>
    
    struct JoinHandle<T> {
        fn await -> Result<T, Error>
        fn abort()
    }
    
    // Channels for async communication
    fn channel<T>(buffer: usize) -> (Sender<T>, Receiver<T>)
    
    struct Sender<T> {
        async fn send(value: T) -> Result<(), Error>
    }
    
    struct Receiver<T> {
        async fn recv() -> Option<T>
    }
}
```

## 11. Error Handling

**Current Status**: ⚠️ Basic Result type exists  
**Priority**: HIGH

```ruchy
// Result type with ? operator support
enum Result<T, E> {
    Ok(T),
    Err(E),
}

impl Result<T, E> {
    fn unwrap(self) -> T
    fn unwrap_or(self, default: T) -> T
    fn unwrap_or_else(self, f: fn() -> T) -> T
    fn expect(self, msg: String) -> T
    fn is_ok(&self) -> bool
    fn is_err(&self) -> bool
    fn map<U>(self, f: fn(T) -> U) -> Result<U, E>
    fn map_err<F>(self, f: fn(E) -> F) -> Result<T, F>
    fn and_then<U>(self, f: fn(T) -> Result<U, E>) -> Result<U, E>
}

// Option type
enum Option<T> {
    Some(T),
    None,
}

impl Option<T> {
    fn unwrap(self) -> T
    fn unwrap_or(self, default: T) -> T
    fn unwrap_or_else(self, f: fn() -> T) -> T
    fn expect(self, msg: String) -> T
    fn is_some(&self) -> bool
    fn is_none(&self) -> bool
    fn map<U>(self, f: fn(T) -> U) -> Option<U>
    fn and_then<U>(self, f: fn(T) -> Option<U>) -> Option<U>
    fn ok_or<E>(self, err: E) -> Result<T, E>
}

// Error trait
trait Error {
    fn message(&self) -> String
    fn source(&self) -> Option<&dyn Error>
}

// Panic handling
fn panic!(msg: String) -> !
fn unreachable!() -> !
fn todo!(msg: String) -> !
```

## 12. Collections

**Current Status**: ⚠️ Basic array support  
**Priority**: HIGH

```ruchy
mod std::collections {
    // HashMap
    struct HashMap<K, V> {
        fn new() -> HashMap<K, V>
        fn insert(&mut self, key: K, value: V) -> Option<V>
        fn get(&self, key: &K) -> Option<&V>
        fn get_mut(&mut self, key: &K) -> Option<&mut V>
        fn remove(&mut self, key: &K) -> Option<V>
        fn contains_key(&self, key: &K) -> bool
        fn len(&self) -> usize
        fn is_empty(&self) -> bool
        fn keys(&self) -> Iterator<&K>
        fn values(&self) -> Iterator<&V>
        fn iter(&self) -> Iterator<(&K, &V)>
    }
    
    // Vec (dynamic array)
    struct Vec<T> {
        fn new() -> Vec<T>
        fn with_capacity(capacity: usize) -> Vec<T>
        fn push(&mut self, value: T)
        fn pop(&mut self) -> Option<T>
        fn insert(&mut self, index: usize, value: T)
        fn remove(&mut self, index: usize) -> T
        fn get(&self, index: usize) -> Option<&T>
        fn len(&self) -> usize
        fn is_empty(&self) -> bool
        fn clear(&mut self)
        fn sort(&mut self)
        fn sort_by(&mut self, f: fn(&T, &T) -> Ordering)
        fn iter(&self) -> Iterator<&T>
        fn map<U>(&self, f: fn(T) -> U) -> Vec<U>
        fn filter(&self, f: fn(&T) -> bool) -> Vec<T>
        fn reduce<U>(&self, init: U, f: fn(U, T) -> U) -> U
    }
    
    // HashSet
    struct HashSet<T> {
        fn new() -> HashSet<T>
        fn insert(&mut self, value: T) -> bool
        fn remove(&mut self, value: &T) -> bool
        fn contains(&self, value: &T) -> bool
        fn len(&self) -> usize
        fn is_empty(&self) -> bool
        fn union(&self, other: &HashSet<T>) -> HashSet<T>
        fn intersection(&self, other: &HashSet<T>) -> HashSet<T>
        fn difference(&self, other: &HashSet<T>) -> HashSet<T>
    }
}
```

## 13. String Operations

**Current Status**: ⚠️ Basic string concatenation  
**Priority**: HIGH

```ruchy
impl String {
    fn len(&self) -> usize
    fn is_empty(&self) -> bool
    fn chars(&self) -> Iterator<char>
    fn bytes(&self) -> Iterator<u8>
    fn contains(&self, pattern: &str) -> bool
    fn starts_with(&self, pattern: &str) -> bool
    fn ends_with(&self, pattern: &str) -> bool
    fn find(&self, pattern: &str) -> Option<usize>
    fn rfind(&self, pattern: &str) -> Option<usize>
    fn split(&self, delimiter: &str) -> [String]
    fn split_once(&self, delimiter: &str) -> Option<(String, String)>
    fn trim(&self) -> String
    fn trim_start(&self) -> String
    fn trim_end(&self) -> String
    fn to_lowercase(&self) -> String
    fn to_uppercase(&self) -> String
    fn replace(&self, from: &str, to: &str) -> String
    fn repeat(&self, n: usize) -> String
    fn parse<T>(&self) -> Result<T, Error>
    
    // String interpolation (already exists)
    f"Hello, {name}!"
}
```

## 14. Compilation and Deployment

**Current Status**: ❌ Not Implemented  
**Priority**: CRITICAL

```ruchy
// CLI command
ruchy compile script.ruchy \
    --output binary \
    --target x86_64-unknown-linux-gnu \
    --release \
    --strip \
    --permissions read,write,net,env,run

// Programmatic API
mod std::compile {
    struct CompileOptions {
        entry: String,
        output: String,
        target: Target,
        release: bool,
        strip: bool,
        permissions: [Permission],
        icon: Option<String>,
        embed_assets: [String],
    }
    
    enum Target {
        Linux_x64,
        Linux_arm64,
        Windows_x64,
        MacOS_x64,
        MacOS_arm64,
    }
    
    fn compile(options: CompileOptions) -> Result<(), Error>
}
```

## 15. Package Management

**Current Status**: ❌ Not Implemented  
**Priority**: MEDIUM

```ruchy
// Package manifest (ruchy.toml)
[package]
name = "ubuntu-config"
version = "1.0.0"
authors = ["developer@example.com"]

[dependencies]
http = "0.2"
json = "1.0"
regex = "1.5"

[dev-dependencies]
test-utils = "0.1"

// CLI commands
ruchy add http@0.2
ruchy remove json
ruchy update
ruchy publish

// Import from packages
import http::client::fetch
import json::{parse, stringify}
```
