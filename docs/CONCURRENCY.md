# Ruchy Concurrency Model

## Design Philosophy

**Core Principle**: Ruchy uses **exactly the same concurrency primitives as Rust** with **zero abstractions**.

- ✅ **1:1 Mapping**: Ruchy syntax transpiles directly to Rust concurrency APIs
- ✅ **Zero Overhead**: No runtime wrappers or indirection
- ✅ **Memory Safe**: All generated code is thread-safe and data-race-free
- ✅ **Zero Unsafe**: Transpiler NEVER generates `unsafe` blocks (GitHub Issue #132)

## ZERO UNSAFE POLICY

**SACRED RULE**: The Ruchy transpiler MUST NEVER generate unsafe Rust code.

### Forbidden Patterns

❌ **NEVER Generate**:
```rust
// ❌ FORBIDDEN: static mut
static mut COUNTER: i32 = 0;

// ❌ FORBIDDEN: unsafe blocks
unsafe {
    COUNTER += 1;
}

// ❌ FORBIDDEN: raw pointers without wrappers
let ptr: *mut i32 = &mut value;
*ptr = 42;
```

### Required Safe Patterns

✅ **ALWAYS Generate**:
```rust
// ✅ CORRECT: LazyLock<Mutex<T>> for globals
static COUNTER: LazyLock<Mutex<i32>> = LazyLock::new(|| Mutex::new(0));
*COUNTER.lock().unwrap() += 1;

// ✅ CORRECT: Arc<Mutex<T>> for shared ownership
let counter = Arc::new(Mutex::new(0));
let counter_clone = Arc::clone(&counter);

// ✅ CORRECT: Channel-based message passing
let (tx, rx) = mpsc::channel();
tx.send(value).unwrap();
```

## Concurrency Primitives

### 1. Threads (`std::thread`)

**Ruchy Syntax** (same as Rust):
```rust
// Spawn a thread
let handle = std::thread::spawn(|| {
    println!("Hello from thread!");
    42
});

// Wait for thread to complete
let result = handle.join().unwrap();  // Returns 42
```

**Transpiled Output** (identical):
```rust
let handle = std::thread::spawn(|| {
    println!("Hello from thread!");
    42
});
let result = handle.join().unwrap();
```

**Thread-Safe Shared State**:
```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    let handle = std::thread::spawn(move || {
        let mut num = counter.lock().unwrap();
        *num += 1;
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

println!("Result: {}", *counter.lock().unwrap());  // Prints: 10
```

### 2. Async/Await (Tokio)

**Async Functions**:
```rust
async fn fetch_url(url: String) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    response.text().await
}
```

**Async Main**:
```rust
#[tokio::main]
async fn main() {
    let body = fetch_url("https://www.rust-lang.org").await;
    println!("Body: {:?}", body);
}
```

**Concurrent Async Tasks**:
```rust
use tokio::task;

let tasks: Vec<_> = urls.into_iter()
    .map(|url| task::spawn(fetch_url(url)))
    .collect();

for task in tasks {
    match task.await {
        Ok(Ok(body)) => println!("Got: {}", body),
        Ok(Err(e)) => eprintln!("Error: {}", e),
        Err(e) => eprintln!("Task error: {}", e),
    }
}
```

### 3. Message Passing (Channels)

**MPSC (Multi-Producer Single-Consumer)**:
```rust
use std::sync::mpsc;

// Create channel
let (tx, rx) = mpsc::channel();

// Spawn producer thread
std::thread::spawn(move || {
    tx.send("Hello from thread").unwrap();
});

// Receive message
let msg = rx.recv().unwrap();
println!("{}", msg);
```

**Async Channels (Tokio)**:
```rust
use tokio::sync::mpsc;

// Create async channel with buffer
let (tx, mut rx) = mpsc::channel(100);

// Spawn producer task
tokio::spawn(async move {
    for i in 0..10 {
        tx.send(i).await.unwrap();
    }
});

// Receive messages
while let Some(value) = rx.recv().await {
    println!("Got: {}", value);
}
```

**Broadcast Channels**:
```rust
use tokio::sync::broadcast;

let (tx, mut rx1) = broadcast::channel(16);
let mut rx2 = tx.subscribe();

tokio::spawn(async move {
    assert_eq!(rx1.recv().await.unwrap(), 10);
    assert_eq!(rx1.recv().await.unwrap(), 20);
});

tokio::spawn(async move {
    assert_eq!(rx2.recv().await.unwrap(), 10);
    assert_eq!(rx2.recv().await.unwrap(), 20);
});

tx.send(10).unwrap();
tx.send(20).unwrap();
```

### 4. Atomics (Lock-Free)

**Atomic Counter**:
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

let counter = Arc::new(AtomicUsize::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    let handle = std::thread::spawn(move || {
        for _ in 0..1000 {
            counter.fetch_add(1, Ordering::SeqCst);
        }
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

println!("Result: {}", counter.load(Ordering::SeqCst));  // 10000
```

**Atomic Bool (Flag)**:
```rust
use std::sync::atomic::{AtomicBool, Ordering};

let stop_flag = Arc::new(AtomicBool::new(false));
let flag_clone = Arc::clone(&stop_flag);

std::thread::spawn(move || {
    while !flag_clone.load(Ordering::Relaxed) {
        // Do work...
        std::thread::sleep(Duration::from_millis(100));
    }
});

// Later: signal thread to stop
stop_flag.store(true, Ordering::Relaxed);
```

### 5. Synchronization Primitives

**Mutex (Exclusive Access)**:
```rust
use std::sync::Mutex;

let data = Mutex::new(vec![1, 2, 3]);

{
    let mut d = data.lock().unwrap();
    d.push(4);
}  // Lock automatically released

// Read the data
let d = data.lock().unwrap();
assert_eq!(*d, vec![1, 2, 3, 4]);
```

**RwLock (Read-Heavy Workloads)**:
```rust
use std::sync::RwLock;

let config = RwLock::new(HashMap::new());

// Multiple readers can read simultaneously
{
    let r1 = config.read().unwrap();
    let r2 = config.read().unwrap();
    // Both can read at same time
}

// Single writer has exclusive access
{
    let mut w = config.write().unwrap();
    w.insert("key", "value");
}
```

**Barrier (Synchronization Point)**:
```rust
use std::sync::{Arc, Barrier};

let barrier = Arc::new(Barrier::new(3));
let mut handles = vec![];

for i in 0..3 {
    let c = Arc::clone(&barrier);
    handles.push(std::thread::spawn(move || {
        println!("Thread {} before barrier", i);
        c.wait();  // Wait for all threads
        println!("Thread {} after barrier", i);
    }));
}

for handle in handles {
    handle.join().unwrap();
}
```

**Condvar (Condition Variable)**:
```rust
use std::sync::{Arc, Mutex, Condvar};

let pair = Arc::new((Mutex::new(false), Condvar::new()));
let pair_clone = Arc::clone(&pair);

std::thread::spawn(move || {
    let (lock, cvar) = &*pair_clone;
    let mut ready = lock.lock().unwrap();
    *ready = true;
    cvar.notify_one();
});

let (lock, cvar) = &*pair;
let mut ready = lock.lock().unwrap();
while !*ready {
    ready = cvar.wait(ready).unwrap();
}
```

## Global Mutable State

**Problem**: How to share mutable state across functions?

### ❌ Wrong Solution (Unsafe)
```rust
// ❌ FORBIDDEN: This generates data-race-prone code
static mut COUNTER: i32 = 0;

fn increment() {
    unsafe {
        COUNTER += 1;  // Data race if called from multiple threads!
    }
}
```

### ✅ Correct Solution (Thread-Safe)

**Ruchy Code**:
```rust
let mut global_counter = 0;

fn increment() {
    global_counter = global_counter + 1;
}

fn main() {
    increment();
    println!("{}", global_counter);
}
```

**Transpiled Code** (GitHub Issue #132 fix):
```rust
use std::sync::{LazyLock, Mutex};

static GLOBAL_COUNTER: LazyLock<Mutex<i32>> =
    LazyLock::new(|| Mutex::new(0));

fn increment() {
    *GLOBAL_COUNTER.lock().unwrap() += 1;  // ✅ Thread-safe!
}

fn main() {
    increment();
    println!("{}", *GLOBAL_COUNTER.lock().unwrap());
}
```

**Why This Works**:
- ✅ `LazyLock`: Lazy initialization (no startup cost)
- ✅ `Mutex`: Exclusive access (no data races)
- ✅ Thread-safe: Works correctly with `std::thread::spawn`
- ✅ Zero unsafe: All memory-safe guarantees preserved

## Performance Considerations

### Lock Contention

**Problem**: High lock contention with `Mutex<T>`

**Solutions**:
1. **RwLock**: Use for read-heavy workloads
   ```rust
   static CONFIG: LazyLock<RwLock<HashMap<String, String>>> =
       LazyLock::new(|| RwLock::new(HashMap::new()));

   // Many readers OK
   let config = CONFIG.read().unwrap();
   ```

2. **Thread-Local Storage**: Per-thread state
   ```rust
   use std::cell::RefCell;
   thread_local! {
       static COUNTER: RefCell<i32> = RefCell::new(0);
   }

   COUNTER.with(|c| *c.borrow_mut() += 1);
   ```

3. **Atomics**: Lock-free for simple operations
   ```rust
   static COUNTER: AtomicUsize = AtomicUsize::new(0);
   COUNTER.fetch_add(1, Ordering::Relaxed);
   ```

### Deadlock Avoidance

**Rule**: Always acquire locks in the same order

```rust
// ✅ CORRECT: Always acquire lock1 before lock2
let data1 = LOCK1.lock().unwrap();
let data2 = LOCK2.lock().unwrap();

// ❌ WRONG: Different order in different places
// Thread 1:
let a = LOCK1.lock().unwrap();
let b = LOCK2.lock().unwrap();  // Deadlock!

// Thread 2:
let b = LOCK2.lock().unwrap();
let a = LOCK1.lock().unwrap();  // Deadlock!
```

## Testing Concurrent Code

### Race Detection (Loom)

```rust
#[cfg(test)]
mod tests {
    use loom::sync::Arc;
    use loom::sync::atomic::{AtomicUsize, Ordering};
    use loom::thread;

    #[test]
    fn test_concurrent_increment() {
        loom::model(|| {
            let counter = Arc::new(AtomicUsize::new(0));

            let threads: Vec<_> = (0..2).map(|_| {
                let counter = Arc::clone(&counter);
                thread::spawn(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                })
            }).collect();

            for t in threads {
                t.join().unwrap();
            }

            assert_eq!(counter.load(Ordering::SeqCst), 2);
        });
    }
}
```

### Stress Testing

```rust
#[test]
fn test_stress_concurrent_access() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..100 {
        let counter = Arc::clone(&counter);
        handles.push(std::thread::spawn(move || {
            for _ in 0..1000 {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 100_000);
}
```

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| Thread-safe globals | ✅ COMPLETE | v3.194.0, LazyLock<Mutex<T>> |
| `std::thread` syntax | ⚠️ PARTIAL | Parser needs work for `spawn` |
| `async`/`await` keywords | ⚠️ PARTIAL | Runtime exists, syntax incomplete |
| Channels (`mpsc`) | ❌ NOT IMPLEMENTED | Use Rust stdlib directly |
| Atomics | ❌ NOT IMPLEMENTED | Use Rust stdlib directly |
| Arc/Mutex patterns | ✅ COMPLETE | Full support, transparent |

## References

- **GitHub Issue #132**: [CRITICAL] Transpiler must use safe abstractions
- **Rust Book - Concurrency**: https://doc.rust-lang.org/book/ch16-00-concurrency.html
- **Tokio Documentation**: https://tokio.rs/
- **LazyLock RFC**: https://github.com/rust-lang/rust/issues/109736
- **CHANGELOG v3.194.0**: TRANSPILER-SCOPE thread-safe implementation

## Future Work

1. **Parser Support**: Add syntax for `spawn`, `async`, `await`
2. **Ownership Analysis**: Detect `move` closures automatically
3. **Deadlock Detection**: Static analysis for lock ordering
4. **Performance Profiling**: Identify lock contention hotspots
5. **Actor Model**: Explore Actix/Bastion integration

---

**Last Updated**: 2025-11-04 (v3.194.0)
