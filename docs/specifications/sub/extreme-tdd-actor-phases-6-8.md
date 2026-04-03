# Sub-spec: EXTREME TDD Actor — Phases 6-8: Property Tests, Mutation Testing, Benchmarks & Quality Gates

**Parent:** [EXTREME-TDD-ACTOR-SPEC.md](../EXTREME-TDD-ACTOR-SPEC.md) Phases 6-8

---

### Phase 6: Property-Based Tests

```rust
// tests/property/actor_properties.rs

use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_message_order_preserved(
        messages in prop::collection::vec(any::<String>(), 1..100)
    ) {
        let code = r#"
            actor Recorder {
                messages: Vec<String>,
                receive record(msg: String) {
                    self.messages.push(msg)
                }
                receive get_all() -> Vec<String> {
                    self.messages.clone()
                }
            }
        "#;
        
        let runtime = tokio_test::block_on(compile_and_load(code));
        let actor = tokio_test::block_on(
            runtime.spawn_actor("Recorder", json!({"messages": []}))
        );
        
        // Send all messages
        for msg in &messages {
            tokio_test::block_on(actor.send("record", json!(msg)));
        }
        
        // Verify order preserved
        let received: Vec<String> = tokio_test::block_on(
            actor.ask("get_all", json!({}))
        );
        
        prop_assert_eq!(messages, received);
    }
    
    #[test]
    fn prop_supervision_never_loses_messages(
        message_count in 10..100usize,
        failure_probability in 0.0..0.3f64,
    ) {
        let code = r#"
            actor Unreliable {
                processed: Vec<i32>,
                
                receive process(n: i32) -> Result<(), Error> {
                    if random() < $FAILURE_PROB {
                        panic!("Random failure");
                    }
                    self.processed.push(n);
                    Ok(())
                }
                
                receive get_processed() -> Vec<i32> {
                    self.processed.clone()
                }
                
                on_restart() {
                    // Preserve processed list
                }
            }
        "#.replace("$FAILURE_PROB", &failure_probability.to_string());
        
        let runtime = tokio_test::block_on(compile_and_load(&code));
        let supervisor = tokio_test::block_on(
            runtime.create_supervisor(SupervisorStrategy::OneForOne, 1000, Duration::from_secs(60))
        );
        let actor = tokio_test::block_on(
            supervisor.spawn_child("Unreliable", json!({"processed": []}))
        );
        
        // Send messages
        let messages: Vec<i32> = (0..message_count as i32).collect();
        for msg in &messages {
            tokio_test::block_on(
                actor.send_reliable("process", json!(msg))
            );
        }
        
        // Wait for processing
        tokio_test::block_on(tokio::time::sleep(Duration::from_millis(100)));
        
        // Verify all processed despite failures
        let processed: Vec<i32> = tokio_test::block_on(
            actor.ask("get_processed", json!({}))
        );
        
        prop_assert_eq!(messages.len(), processed.len());
        prop_assert_eq!(messages.into_iter().collect::<HashSet<_>>(), 
                       processed.into_iter().collect::<HashSet<_>>());
    }
    
    #[test]
    fn prop_actor_isolation(
        actor_count in 2..20usize,
        operations_per_actor in 10..100usize,
    ) {
        let code = r#"
            actor Isolated {
                state: i32,
                receive add(n: i32) { self.state += n }
                receive get() -> i32 { self.state }
            }
        "#;
        
        let runtime = tokio_test::block_on(compile_and_load(code));
        
        // Spawn actors
        let actors: Vec<_> = (0..actor_count)
            .map(|_| tokio_test::block_on(
                runtime.spawn_actor("Isolated", json!({"state": 0}))
            ))
            .collect();
        
        // Generate operations for each actor
        let operations: Vec<Vec<i32>> = (0..actor_count)
            .map(|_| (0..operations_per_actor).map(|_| rand::random::<i32>() % 100).collect())
            .collect();
        
        // Apply operations concurrently
        let handles: Vec<_> = actors.iter().zip(&operations)
            .map(|(actor, ops)| {
                let actor = actor.clone();
                let ops = ops.clone();
                tokio_test::task::spawn(async move {
                    for op in ops.iter() {
                        actor.send("add", json!(op)).await;
                    }
                    actor.ask("get", json!({})).await
                })
            })
            .collect();
        
        let results: Vec<i32> = tokio_test::block_on(
            futures::future::join_all(handles)
        ).into_iter().map(|r| r.unwrap()).collect();
        
        // Verify isolation - each actor should have sum of its operations
        for (i, result) in results.iter().enumerate() {
            let expected: i32 = operations[i].iter().sum();
            prop_assert_eq!(*result, expected);
        }
    }
}
```

### Phase 7: Mutation Testing

```toml
# .mutants.toml
[mutants]
timeout = 30
test_tool = "nextest"
exclude_patterns = [
    "tests/**",  # Don't mutate tests
]

[[mutants.rules]]
path = "src/actors/**"
minimum_kill_rate = 0.95  # 95% of mutants must be caught

[[mutants.rules]]
path = "src/transpiler/actors.rs"
minimum_kill_rate = 1.0  # 100% for critical transpilation
```

```rust
// tests/mutation/actor_mutation_test.rs

#[test]
fn verify_mutation_coverage() {
    let report = run_mutants(&["src/actors"]);
    
    assert!(report.kill_rate >= 0.95, 
            "Mutation kill rate {:.2}% is below 95% threshold", 
            report.kill_rate * 100.0);
    
    // Check specific mutations that must be caught
    let critical_mutations = [
        "operator_replacement: ! to ?",  // Must catch send vs ask
        "constant_replacement: 0 to 1",  // Must catch off-by-one
        "condition_flip: < to >=",       // Must catch boundary conditions
        "return_value: Ok to Err",       // Must catch error handling
    ];
    
    for mutation in &critical_mutations {
        assert!(report.caught_mutations.contains(mutation),
                "Critical mutation '{}' was not caught by tests", mutation);
    }
}
```

### Phase 8: Benchmark Tests

```rust
// tests/bench/actor_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_actor_spawn(c: &mut Criterion) {
    let runtime = setup_runtime();
    
    c.bench_function("actor_spawn", |b| {
        b.iter(|| {
            runtime.spawn_actor("Empty", json!({}))
        });
    });
}

fn bench_message_throughput(c: &mut Criterion) {
    let runtime = setup_runtime();
    let actor = runtime.spawn_actor("Counter", json!({"value": 0}));
    
    c.bench_function("message_send", |b| {
        b.iter(|| {
            black_box(actor.send("increment", json!({})))
        });
    });
}

fn bench_ask_latency(c: &mut Criterion) {
    let runtime = setup_runtime();
    let actor = runtime.spawn_actor("Echo", json!({}));
    
    c.bench_function("ask_latency", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
         .iter(|| async {
            black_box(actor.ask("echo", json!("test")).await)
         });
    });
}

fn bench_supervision_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("supervision");
    
    for supervisor_strategy in &[
        SupervisorStrategy::OneForOne,
        SupervisorStrategy::AllForOne,
        SupervisorStrategy::RestForOne,
    ] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", supervisor_strategy)),
            supervisor_strategy,
            |b, strategy| {
                let runtime = setup_runtime();
                let supervisor = runtime.create_supervisor(strategy.clone(), 10, Duration::from_secs(60));
                let actor = supervisor.spawn_child("Failing", json!({}));
                
                b.iter(|| {
                    // Force restart
                    black_box(actor.send("fail", json!({})));
                    black_box(actor.send("recover", json!({})));
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, 
    bench_actor_spawn, 
    bench_message_throughput,
    bench_ask_latency,
    bench_supervision_overhead
);
criterion_main!(benches);
```

### Quality Gates Configuration

```yaml
# .github/quality-gates.yml
name: EXTREME-TDD Quality Gates

gates:
  coverage:
    overall: 95
    new_code: 100  # Every new line must be tested
    per_file:
      src/actors/**/*.rs: 100
      src/transpiler/actors.rs: 100
      src/runtime/supervision.rs: 95
    
  complexity:
    cyclomatic: 5
    cognitive: 8
    nesting: 3
    
  performance:
    actor_spawn_p99: 100µs
    message_send_p99: 1µs
    ask_latency_p99: 10µs
    supervision_restart_p99: 500µs
    
  testing:
    test_ratio: 3.0  # 3 test lines per implementation line
    property_tests: required
    mutation_score: 0.95
    benchmarks: required
    
  documentation:
    public_items: 100%
    examples: required
    
failure_policy: block_merge
```

### Continuous Quality Monitoring

```rust
// tools/quality_monitor.rs

fn main() {
    let metrics = QualityMetrics::collect(".");
    
    // Generate report
    println!("=== EXTREME-TDD Quality Report ===");
    println!("Coverage: {:.1}%", metrics.coverage);
    println!("Mutation Score: {:.1}%", metrics.mutation_score * 100.0);
    println!("Test Ratio: {:.1}:1", metrics.test_ratio);
    println!("Max Complexity: {}", metrics.max_complexity);
    println!("Property Tests: {}", metrics.property_test_count);
    println!("Benchmarks: {}", metrics.benchmark_count);
    
    // Check gates
    let violations = metrics.check_gates();
    if !violations.is_empty() {
        eprintln!("\n❌ Quality Gate Violations:");
        for v in violations {
            eprintln!("  - {}", v);
        }
        std::process::exit(1);
    }
    
    println!("\n✅ All quality gates passed!");
}
```

## Enforcement Checklist

### Pre-Commit Hooks
```bash
#!/bin/bash
# .git/hooks/pre-commit

# No untested code
untested=$(git diff --cached --name-only | xargs grep -l "^impl\|^fn" | while read f; do
    test_file="tests/${f%.rs}_test.rs"
    if [ ! -f "$test_file" ]; then
        echo "$f"
    fi
done)

if [ ! -z "$untested" ]; then
    echo "❌ Files without tests:"
    echo "$untested"
    exit 1
fi

# Run tests
cargo test --all || exit 1

# Check coverage
cargo llvm-cov --html
coverage=$(cargo llvm-cov --summary-only | grep -oP '\d+\.\d+%' | head -1 | tr -d '%')
if (( $(echo "$coverage < 95" | bc -l) )); then
    echo "❌ Coverage $coverage% is below 95% threshold"
    exit 1
fi

echo "✅ All pre-commit checks passed"
```

### CI Pipeline
```yaml
# .github/workflows/extreme-tdd.yml
name: EXTREME-TDD Pipeline

on: [push, pull_request]

jobs:
  test-first:
    runs-on: ubuntu-latest
    steps:
      - name: Ensure Tests Exist
        run: |
          for src in $(find src -name "*.rs"); do
            test_file="tests/${src%.rs}_test.rs"
            if [ ! -f "$test_file" ]; then
              echo "::error::Missing test for $src"
              exit 1
            fi
          done
      
      - name: Test-to-Code Ratio
        run: |
          test_lines=$(find tests -name "*.rs" | xargs wc -l | tail -1 | awk '{print $1}')
          src_lines=$(find src -name "*.rs" | xargs wc -l | tail -1 | awk '{print $1}')
          ratio=$(echo "scale=1; $test_lines / $src_lines" | bc)
          if (( $(echo "$ratio < 3.0" | bc -l) )); then
            echo "::error::Test ratio $ratio:1 is below 3:1 threshold"
            exit 1
          fi
      
      - name: Run All Tests
        run: cargo nextest run --all
      
      - name: Coverage Check
        run: |
          cargo llvm-cov --all --html
          cargo llvm-cov --summary-only --fail-under 95
      
      - name: Mutation Testing
        run: cargo mutants --minimum-kill-rate 0.95
      
      - name: Property Tests
        run: cargo test --features proptest
      
      - name: Benchmarks
        run: cargo bench --no-fail-fast
```

## The EXTREME-TDD Manifesto for Ruchy Actors

1. **Tests are the specification** - The test suite IS the documentation
2. **Coverage is not a metric, it's a requirement** - <95% = broken build
3. **Every bug is a missing test** - Bugs prove our tests were incomplete
4. **Mutation testing proves test quality** - If mutants survive, tests are weak
5. **Property tests prove correctness** - Examples test cases, properties prove laws
6. **Benchmarks prevent regression** - Performance is a feature
7. **TDD is not a practice, it's the only way** - Implementation without test is technical debt

## Timeline

- **Day 1**: Write ALL tests (no implementation)
- **Day 2-3**: Make parser tests pass
- **Day 4-5**: Make type tests pass  
- **Day 6-7**: Make transpiler tests pass
- **Day 8-9**: Make runtime tests pass
- **Day 10**: Make property tests pass
- **Day 11**: Make benchmarks pass
- **Day 12**: Demo ready

Total: 12 days from zero to demo with 100% test coverage.
