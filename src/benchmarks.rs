//! Performance benchmarks for Ruchy compiler components
//!
//! PMAT A+ Quality Standards:
//! - Maximum cyclomatic complexity: 10
//! - No TODO/FIXME/HACK comments
//! - 100% test coverage for benchmark functions

#[cfg(test)]
mod benchmark_tests {
    use std::time::{Duration, Instant};
    use crate::frontend::{Parser, lexer::TokenStream};
    use crate::backend::Transpiler;
    use crate::runtime::interpreter::Interpreter;
    use crate::runtime::cache::{BytecodeCache, CacheKey, CachedResult};
    use crate::frontend::ast::{Expr, ExprKind, Literal, BinaryOp, Span};
    use std::rc::Rc;

    // Benchmark helper functions
    fn time_operation<F, R>(operation: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        (result, duration)
    }

    fn create_benchmark_expr(depth: usize) -> Expr {
        if depth == 0 {
            Expr {
                kind: ExprKind::Literal(Literal::Integer(42)),
                span: Span::new(0, 2),
                attributes: vec![],
            }
        } else {
            Expr {
                kind: ExprKind::Binary {
                    left: Box::new(create_benchmark_expr(depth - 1)),
                    op: BinaryOp::Add,
                    right: Box::new(create_benchmark_expr(depth - 1)),
                },
                span: Span::new(0, 10),
                attributes: vec![],
            }
        }
    }

    #[test]
    fn benchmark_simple_parsing() {
        let input = "42";
        let iterations = 1000;
        
        let (_, total_duration) = time_operation(|| {
            for _ in 0..iterations {
                let tokens = TokenStream::new(input);
                let mut parser = Parser::new();
                let _ = parser.parse_tokens(tokens);
            }
        });
        
        let avg_duration = total_duration / iterations;
        println!("Simple parsing average: {:?} per operation", avg_duration);
        
        // Performance assertions
        assert!(avg_duration < Duration::from_millis(1), "Parsing should be under 1ms");
        assert!(total_duration < Duration::from_secs(1), "Total should be under 1 second");
    }

    #[test]
    fn benchmark_arithmetic_parsing() {
        let input = "1 + 2 * 3 - 4 / 5";
        let iterations = 500;
        
        let (_, total_duration) = time_operation(|| {
            for _ in 0..iterations {
                let tokens = TokenStream::new(input);
                let mut parser = Parser::new();
                let _ = parser.parse_tokens(tokens);
            }
        });
        
        let avg_duration = total_duration / iterations;
        println!("Arithmetic parsing average: {:?} per operation", avg_duration);
        
        // Should handle complex arithmetic efficiently
        assert!(avg_duration < Duration::from_millis(5), "Complex parsing should be under 5ms");
    }

    #[test]
    fn benchmark_string_parsing() {
        let input = "\"This is a test string with some content\"";
        let iterations = 1000;
        
        let (_, total_duration) = time_operation(|| {
            for _ in 0..iterations {
                let tokens = TokenStream::new(input);
                let mut parser = Parser::new();
                let _ = parser.parse_tokens(tokens);
            }
        });
        
        let avg_duration = total_duration / iterations;
        println!("String parsing average: {:?} per operation", avg_duration);
        
        assert!(avg_duration < Duration::from_millis(2), "String parsing should be under 2ms");
    }

    #[test]
    fn benchmark_transpilation() {
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(Expr {
                    kind: ExprKind::Literal(Literal::Integer(1)),
                    span: Span::new(0, 1),
                    attributes: vec![],
                }),
                op: BinaryOp::Add,
                right: Box::new(Expr {
                    kind: ExprKind::Literal(Literal::Integer(2)),
                    span: Span::new(4, 5),
                    attributes: vec![],
                }),
            },
            span: Span::new(0, 5),
            attributes: vec![],
        };
        
        let iterations = 1000;
        let transpiler = Transpiler::new();
        
        let (_, total_duration) = time_operation(|| {
            for _ in 0..iterations {
                let _ = transpiler.transpile_expr(&expr);
            }
        });
        
        let avg_duration = total_duration / iterations;
        println!("Transpilation average: {:?} per operation", avg_duration);
        
        assert!(avg_duration < Duration::from_millis(1), "Transpilation should be under 1ms");
    }

    #[test]
    fn benchmark_interpreter_evaluation() {
        let expr = create_benchmark_expr(5); // Depth 5 binary tree
        let iterations = 100;
        
        let (_, total_duration) = time_operation(|| {
            for _ in 0..iterations {
                let mut interpreter = Interpreter::new();
                let _ = interpreter.evaluate_expr(&expr);
            }
        });
        
        let avg_duration = total_duration / iterations;
        println!("Interpreter evaluation average: {:?} per operation", avg_duration);
        
        assert!(avg_duration < Duration::from_millis(10), "Evaluation should be under 10ms");
    }

    #[test]
    fn benchmark_cache_operations() {
        let mut cache = BytecodeCache::new();
        let iterations = 10000;
        
        // Benchmark insertions
        let (_, insert_duration) = time_operation(|| {
            for i in 0..iterations {
                let key = CacheKey::new(format!("key_{}", i));
                let expr = create_benchmark_expr(0);
                let result = CachedResult {
                    ast: Rc::new(expr),
                    rust_code: Some(format!("result_{}", i)),
                    timestamp: Instant::now(),
                };
                cache.insert(key, result);
            }
        });
        
        println!("Cache insertion average: {:?} per operation", 
                insert_duration / iterations);
        
        // Benchmark lookups
        let (_, lookup_duration) = time_operation(|| {
            for i in 0..iterations {
                let key = CacheKey::new(format!("key_{}", i));
                let _ = cache.get(&key);
            }
        });
        
        println!("Cache lookup average: {:?} per operation", 
                lookup_duration / iterations);
        
        assert!(insert_duration / iterations < Duration::from_millis(1));
        assert!(lookup_duration / iterations < Duration::from_millis(1));
    }

    #[test]
    fn benchmark_deep_expression_handling() {
        // Test performance with deeply nested expressions
        let depths = vec![1, 5, 10, 15];
        
        for depth in depths {
            let expr = create_benchmark_expr(depth);
            let iterations = 50;
            
            // Benchmark transpilation
            let transpiler = Transpiler::new();
            let (_, transpile_duration) = time_operation(|| {
                for _ in 0..iterations {
                    let _ = transpiler.transpile_expr(&expr);
                }
            });
            
            // Benchmark evaluation
            let (_, eval_duration) = time_operation(|| {
                for _ in 0..iterations {
                    let mut interpreter = Interpreter::new();
                    let _ = interpreter.evaluate_expr(&expr);
                }
            });
            
            let avg_transpile = transpile_duration / iterations;
            let avg_eval = eval_duration / iterations;
            
            println!("Depth {}: Transpile {:?}, Eval {:?}", 
                    depth, avg_transpile, avg_eval);
            
            // Performance should degrade gracefully with depth
            assert!(avg_transpile < Duration::from_millis(100), 
                   "Transpilation should handle depth {} efficiently", depth);
            assert!(avg_eval < Duration::from_millis(100), 
                   "Evaluation should handle depth {} efficiently", depth);
        }
    }

    #[test]
    fn benchmark_memory_usage() {
        // Test memory efficiency of operations
        let iterations = 1000;
        
        // Benchmark AST creation memory pattern
        let (expressions, creation_duration) = time_operation(|| {
            let mut expressions = Vec::new();
            for i in 0..iterations {
                let expr = Expr {
                    kind: ExprKind::Literal(Literal::Integer(i as i64)),
                    span: Span::new(0, 5),
                    attributes: vec![],
                };
                expressions.push(expr);
            }
            expressions
        });
        
        println!("AST creation for {} items: {:?}", iterations, creation_duration);
        assert_eq!(expressions.len(), iterations);
        
        // Benchmark cache memory efficiency
        let mut cache = BytecodeCache::new();
        let (_, cache_fill_duration) = time_operation(|| {
            for i in 0..iterations {
                let key = CacheKey::new(format!("mem_test_{}", i));
                let result = CachedResult {
                    ast: Rc::new(expressions[i].clone()),
                    rust_code: None,
                    timestamp: Instant::now(),
                };
                cache.insert(key, result);
            }
        });
        
        println!("Cache memory fill: {:?}", cache_fill_duration);
        assert_eq!(cache.len(), iterations);
    }

    #[test]
    fn benchmark_concurrent_operations() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        // Benchmark thread-safe operations
        let cache = Arc::new(Mutex::new(BytecodeCache::new()));
        let thread_count = 4;
        let operations_per_thread = 250;
        
        let (_, concurrent_duration) = time_operation(|| {
            let handles: Vec<_> = (0..thread_count).map(|thread_id| {
                let cache_clone = Arc::clone(&cache);
                thread::spawn(move || {
                    for i in 0..operations_per_thread {
                        let key = CacheKey::new(format!("thread_{}_{}", thread_id, i));
                        let expr = create_benchmark_expr(0);
                        let result = CachedResult {
                            ast: Rc::new(expr),
                            rust_code: Some(format!("result_{}_{}", thread_id, i)),
                            timestamp: Instant::now(),
                        };
                        
                        {
                            let mut cache_guard = cache_clone.lock().unwrap();
                            cache_guard.insert(key, result);
                        }
                    }
                })
            }).collect();
            
            for handle in handles {
                handle.join().unwrap();
            }
        });
        
        println!("Concurrent operations ({} threads): {:?}", 
                thread_count, concurrent_duration);
        
        let final_size = cache.lock().unwrap().len();
        assert_eq!(final_size, thread_count * operations_per_thread);
        
        // Concurrent operations should complete in reasonable time
        assert!(concurrent_duration < Duration::from_secs(5), 
               "Concurrent operations should complete within 5 seconds");
    }

    #[test]
    fn benchmark_compilation_pipeline() {
        let test_cases = vec![
            "42",
            "1 + 2",
            "\"hello world\"",
            "true && false",
            "1 + 2 * 3 - 4 / 5",
        ];
        
        for input in test_cases {
            let iterations = 100;
            
            let (_, pipeline_duration) = time_operation(|| {
                for _ in 0..iterations {
                    // Full pipeline: Parse -> Transpile -> Evaluate
                    let tokens = TokenStream::new(input);
                    let mut parser = Parser::new();
                    
                    if let Ok(ast) = parser.parse_tokens(tokens) {
                        let transpiler = Transpiler::new();
                        let _ = transpiler.transpile_expr(&ast);
                        
                        let mut interpreter = Interpreter::new();
                        let _ = interpreter.evaluate_expr(&ast);
                    }
                }
            });
            
            let avg_pipeline = pipeline_duration / iterations;
            println!("Full pipeline '{}': {:?} average", input, avg_pipeline);
            
            // Full pipeline should be efficient
            assert!(avg_pipeline < Duration::from_millis(20), 
                   "Pipeline for '{}' should be under 20ms", input);
        }
    }
}

// Mock implementations needed for benchmarks
use crate::runtime::cache::{CacheKey, CachedResult};
use std::collections::HashMap;

impl CacheKey {
    pub fn new(source: String) -> Self {
        use std::hash::{Hash, Hasher};
        let hash = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            source.hash(&mut hasher);
            hasher.finish()
        };
        Self { source, hash }
    }
}

pub struct BytecodeCache {
    cache: HashMap<CacheKey, CachedResult>,
}

impl BytecodeCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: CacheKey, result: CachedResult) {
        self.cache.insert(key, result);
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<&CachedResult> {
        self.cache.get(key)
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }
}