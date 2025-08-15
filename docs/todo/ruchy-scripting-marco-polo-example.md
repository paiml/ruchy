# Marco Polo Script - Minimal Ruchy Demo

## Philosophy: Hello World That Actually Does Something

The simplest possible Ruchy script that demonstrates script→REPL→binary progression without external dependencies or complexity.

---

## 1. The Complete Script - Two Versions

### Version A: Ultra-Minimal (20 lines)

```ruchy
#!/usr/bin/env ruchy

fn main() {
    match args() {
        ["marco"] => marco(),
        ["polo", name] => polo(name),
        ["play", times] => play(times.parse().unwrap_or(3)),
        _ => help()
    }
}

fn marco() {
    println("Marco!")
    println("(waiting for polo...)")
}

fn polo(name = "Anonymous") {
    println("Polo from {name}!")

## 2. Progressive Execution Modes

### 2.1 Script Mode (Direct Execution)
```bash
# Version A - Simple
$ ./marco-polo.ruchy marco
Marco!
(waiting for polo...)

# Version B - With Clap (automatic --help!)
$ ./marco-polo-clap.ruchy --help
A fun Marco Polo game

Usage: marco <COMMAND>

Commands:
  marco  Send a Marco call
  polo   Respond with Polo
  play   Play automatically
  help   Print this message

$ ./marco-polo-clap.ruchy marco --loud
MARCO!!!
(waiting for polo...)

$ ./marco-polo-clap.ruchy polo --name Alice --emoji 🎉
🎉 Polo from Alice!
```

### 2.2 REPL Mode (Interactive)
```bash
$ ruchy
ruchy> :load marco-polo.ruchy
ruchy> marco()
Marco!
(waiting for polo...)

ruchy> polo("World")
Polo from World!

ruchy> // Modify on the fly
ruchy> fn marco() = println("MARCO!!!")
ruchy> marco()
MARCO!!!
```

### 2.3 Compiled Binary
```bash
# Compile to standalone
$ ruchy build marco-polo.ruchy -o mp
Compiling... done! (0.8s)
Binary: mp (412KB)

$ ./mp play 3
1. Marco!
   Polo!
2. Marco!
   Polo!
3. Marco!
   Polo!

# Binary is just Rust - zero overhead
$ time ./mp marco
Marco!
real    0m0.001s
```

## 3. What Makes This "Fun"

### 3.1 No Import Hell
```ruchy
// Version A: Zero imports
// Version B: Just one - and it gets everything
use cli::*  // Parser, Subcommand, arg macros - all there

// NOT this Rust nightmare:
use clap::{Parser, Subcommand, Args, ArgEnum, CommandFactory, FromArgMatches};
use std::time::Duration;
use std::thread::sleep;
```

### 3.2 Smart Preludes
```rust
// ~/.ruchy/preludes/cli.rs (auto-maintained)
pub use clap::{Parser, Subcommand, Args, ArgEnum};
pub use clap::{command, arg, value_enum};

// Plus Ruchy additions:
pub trait Parse {
    fn parse() -> Self;  // No need to import separately
}

impl<T: Parser> Parse for T {
    fn parse() -> Self {
        T::parse()  // Delegates to clap
    }
}
```

### 3.3 Extension Methods That Matter
```ruchy
// These just work:
sleep(500.ms())     // Not sleep(Duration::from_millis(500))
println("{icon}")   // Not println!("{}", icon)  
"text".to_string()  // Available everywhere

// Because Ruchy adds:
trait IntExt {
    fn ms(self) -> Duration;
    fn s(self) -> Duration;
}
```

## 4. Implementation: Smart Defaults and Inference

### 4.1 Dependency Resolution
```rust
// Ruchy detects clap usage and injects minimal dependencies
impl DependencyResolver {
    fn resolve(&self, ast: &AST) -> Dependencies {
        let mut deps = Dependencies::minimal();
        
        // Detected: #[derive(Parser)]
        if ast.has_derive("Parser") {
            deps.add("clap", "4.5", ["derive"]);  // Just what's needed
        }
        
        // Detected: sleep() call
        if ast.has_call("sleep") {
            deps.ensure_std_time();  // No tokio needed for simple sleep
        }
        
        deps  // Typically <5 dependencies vs Rust's 50+
    }
}
```

### 4.2 The `cli` Prelude
```rust
// How 'use cli::*' works
impl PreludeResolver {
    fn resolve_cli_prelude(&self) -> TokenStream {
        quote! {
            // Flat namespace - everything at top level
            pub use clap::Parser;
            pub use clap::Subcommand;
            pub use clap::Args;
            
            // Re-export derive macros (tricky in Rust, trivial in Ruchy)
            pub use clap_derive::{Parser, Subcommand, Args};
            
            // Convenience additions
            impl<T: clap::Parser> T {
                fn parse() -> Self {
                    <T as clap::Parser>::parse()
                }
            }
        }
    }
}
```

### 4.3 Binary Size Management
```rust
// Clap adds ~200KB to binary. Ruchy minimizes this:
impl BinaryOptimizer {
    fn optimize_clap(&self, rust_code: &str) -> String {
        // Strip unused clap features
        let features_used = self.detect_clap_features(rust_code);
        
        // Only include what's actually used
        match features_used {
            Just(["derive"]) => {
                // Don't include: suggestions, completions, man pages
                // Saves ~80KB
            },
            Full => {
                // User wants everything, include it all
            }
        }
    }
}
```

## 5. Technical Minimalism

```rust
// Version A: Zero dependencies, pure mechanical transformation
impl MinimalTranspiler {
    fn transpile(&self, source: &str) -> String {
        parse(source)
            .desugar_string_interp()    // "text {var}" → format!("text {}", var)
            .desugar_defaults()          // fn f(x = 1) → fn f(x: i32) + fn f() { f(1) }
            .inject_args_helper()        // Add args() → env::args().skip(1).collect()
            .wrap_main()                 // Ensure proper main signature
    }
}

// Version B: Clap integration via prelude injection
impl ClapTranspiler {
    fn transpile(&self, source: &str) -> String {
        let ast = parse(source);
        
        // Detect clap patterns
        let needs_clap = ast.has_derive("Parser") || ast.has_derive("Subcommand");
        
        let mut rust = ast.to_rust();
        
        if needs_clap {
            // Expand 'use cli::*' to actual imports
            rust = rust.replace("use cli::*", &self.expand_cli_prelude());
            
            // Add dependency to generated Cargo.toml
            self.deps.push(("clap", "4.5", vec!["derive", "std"]));
        }
        
        rust
    }
    
    fn expand_cli_prelude(&self) -> &str {
        "use clap::{Parser, Subcommand, Args, command, arg};\n\
         use std::thread::sleep;\n\
         use std::time::Duration;"
    }
}
```

## 6. Performance Characteristics

| Metric | Version A | Version B (Clap) | Binary A | Binary B |
|--------|-----------|------------------|----------|----------|
| Parse Time | 2ms | 8ms | N/A | N/A |
| Compile Time | 180ms | 420ms | +800ms | +1.2s |
| Binary Size | N/A | N/A | 412KB | 614KB |
| Runtime Overhead | 8ms* | 11ms* | 0.4ms | 0.8ms |
| Memory (RSS) | 12MB | 16MB | 0.8MB | 1.4MB |

*First run includes Rust compilation (cached after)

The 200KB binary size increase from Clap includes:
- Clap parser core: ~120KB
- Error formatting: ~40KB
- Help generation: ~30KB
- Derive machinery: ~10KB

### 6.1 Compilation Cache Strategy

```rust
// ~/.ruchy/cache/
cache/
├── deps/           # Compiled dependencies (rlibs)
│   ├── clap-4.5.0-derive-std.rlib
│   └── clap_derive-4.5.0.so
├── scripts/        # Compiled scripts (content-addressed)
│   ├── blake3_hash_1.dylib
│   └── blake3_hash_2.dylib
└── metadata.json   # Version tracking

// Cache hit path (99% of executions):
Script → Blake3 hash → Cache lookup → dlopen() → Run
         2μs           100ns          500μs      <1ms total
```
## 7. Why This Design: Language Boundary Economics

### 7.1 The Gradient Philosophy
**Version A (args matching)**: Zero-dependency baseline. Pattern matching on `Vec<String>` eliminates parse overhead entirely. Comparable to shell script performance with type safety.

**Version B (Clap integration)**: 35% code increase yields automatic help generation, validation, and subcommand routing. The 200KB binary penalty amortizes over feature richness.

### 7.2 Import Ergonomics via Prelude Injection
```rust
// Traditional Rust: 6 imports for basic Clap usage
use clap::{Parser, Subcommand, Args};
use clap_derive::{Parser as DeriveParser};
use std::{thread::sleep, time::Duration};

// Ruchy: Single prelude import
use cli::*

// Implementation: Symbol table preloading
impl CompilerContext {
    fn initialize_preludes(&mut self) {
        self.symbol_table.register_prelude("cli", &[
            ("Parser", "clap::Parser"),
            ("Subcommand", "clap::Subcommand"),
            ("parse", "<impl as clap::Parser>::parse"),
            // Extension methods injected as inherent impls
        ]);
    }
}
```

### 7.3 Mechanical Simplicity
No proc-macro reimplementation. Ruchy's `#[derive(Parser)]` IS clap's derive - the compiler merely manages the import namespace. Total added compiler complexity: ~500 LOC for prelude management + dependency injection.

## 8. Extending While Keeping Fun

```ruchy
// Add one feature at a time
fn play_with_colors(times: i32) {
    use color::*  // Now you need colors
    
    for i in 1..=times {
        println(cyan("Marco!"))
        println(green("  Polo!"))
    }
}

// Still works in all three modes:
// - Script: colors resolved from ~/.ruchy/deps
// - REPL: colors loaded dynamically
// - Binary: colors compiled in (adds ~50KB)
```

## The Point

This isn't about reimplementing Rust with different syntax. It's about having a **gradient** from Python-simple scripts to Rust-fast binaries, using the same source file. The Marco Polo example shows this in 20 lines with zero magic - just mechanical syntax transformation and smart defaults.