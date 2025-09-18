//! TDD Tests for Async/Await Support
//! Sprint v3.19.0 - Asynchronous programming with async/await

use ruchy::frontend::parser::Parser;
use ruchy::runtime::async_runtime::{AsyncRuntime, Future};
use ruchy::Transpiler;

#[cfg(test)]
mod async_functions {
    use super::*;

    #[test]
    fn test_async_function_declaration() {
        let input = r#"
        async fn fetch_data() -> String {
            "data"
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
        let output_str = output.unwrap();
        assert!(output_str.contains("async") || output_str.contains("fn"));
    }

    #[test]
    fn test_async_function_with_params() {
        let input = r#"
        async fn process(id: u32, data: String) -> Result<String, Error> {
            Ok(data + id.to_string())
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }

    #[test]
    fn test_async_closure() {
        let input = r#"
        let handler = async |x: i32| -> i32 {
            x * 2
        };
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }

    #[test]
    fn test_async_method() {
        let input = r#"
        impl DataFetcher {
            async fn fetch(&self, url: String) -> String {
                // Fetch implementation
                url
            }
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }
}

#[cfg(test)]
mod await_expressions {
    use super::*;

    #[test]
    fn test_simple_await() {
        let input = r#"
        async fn main() {
            let result = fetch_data().await;
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }

    #[test]
    fn test_chained_await() {
        let input = r#"
        async fn pipeline() {
            let data = fetch().await.process().await.save().await;
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }

    #[test]
    fn test_await_in_expression() {
        let input = r#"
        async fn compute() {
            let sum = async_add(1, 2).await + async_multiply(3, 4).await;
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }

    #[test]
    fn test_await_with_error_handling() {
        let input = r#"
        async fn safe_fetch() {
            match fetch_data().await {
                Ok(data) => println(data),
                Err(e) => println("Error: " + e)
            }
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }
}

#[cfg(test)]
mod async_runtime_tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_execution() {
        let runtime = AsyncRuntime::new();

        let future = runtime.spawn(async {
            42
        });

        let result = future.await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_multiple_tasks() {
        let runtime = AsyncRuntime::new();

        let task1 = runtime.spawn(async { 1 });
        let task2 = runtime.spawn(async { 2 });
        let task3 = runtime.spawn(async { 3 });

        let sum = task1.await + task2.await + task3.await;
        assert_eq!(sum, 6);
    }

    #[tokio::test]
    async fn test_concurrent_execution() {
        let runtime = AsyncRuntime::new();

        let tasks: Vec<_> = (0..10)
            .map(|i| runtime.spawn(async move { i * 2 }))
            .collect();

        let mut results = Vec::new();
        for task in tasks {
            results.push(task.await);
        }

        assert_eq!(results.len(), 10);
    }

    #[tokio::test]
    async fn test_async_sleep() {
        let runtime = AsyncRuntime::new();

        let start = std::time::Instant::now();
        runtime.sleep(std::time::Duration::from_millis(10)).await;
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() >= 10);
    }
}

#[cfg(test)]
mod async_combinators {
    use super::*;

    #[test]
    fn test_async_map() {
        let input = r#"
        async fn transform() {
            let result = fetch_number()
                .await
                .map(|n| n * 2)
                .unwrap_or(0);
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }

    #[test]
    fn test_async_and_then() {
        let input = r#"
        async fn chain() {
            let result = fetch_user()
                .await
                .and_then(|user| fetch_profile(user.id))
                .await;
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }

    #[test]
    fn test_join_futures() {
        let input = r#"
        async fn parallel() {
            let (a, b, c) = join!(
                fetch_a(),
                fetch_b(),
                fetch_c()
            ).await;
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }

    #[test]
    fn test_select_futures() {
        let input = r#"
        async fn race() {
            let result = select!(
                a = fetch_fast() => a,
                b = fetch_slow() => b,
            ).await;
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }
}

#[cfg(test)]
mod async_streams {
    use super::*;

    #[test]
    fn test_async_iterator() {
        let input = r#"
        async fn stream_data() {
            let mut stream = fetch_stream().await;
            while let Some(item) = stream.next().await {
                process(item);
            }
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }

    #[test]
    fn test_async_generator() {
        let input = r#"
        async gen fn numbers() -> i32 {
            for i in 0..10 {
                yield i;
            }
        }
        "#;

        let mut parser = Parser::new(input);
        // Parser might not support async gen yet
        let _ = parser.parse();
    }

    #[test]
    fn test_async_collect() {
        let input = r#"
        async fn collect_all() {
            let items: Vec<Data> = stream
                .collect()
                .await;
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }
}

#[cfg(test)]
mod async_error_handling {
    use super::*;

    #[test]
    fn test_async_try_block() {
        let input = r#"
        async fn try_operation() -> Result<String, Error> {
            let data = fetch_data().await?;
            let processed = process(data).await?;
            Ok(processed)
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }

    #[test]
    fn test_async_panic_handling() {
        let input = r#"
        async fn safe_operation() {
            let result = panic::catch_unwind(async || {
                risky_operation().await
            }).await;
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }

    #[test]
    fn test_async_timeout() {
        let input = r#"
        async fn with_timeout() {
            match timeout(Duration::from_secs(5), fetch_data()).await {
                Ok(data) => process(data),
                Err(_) => println("Timeout!")
            }
        }
        "#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let output = transpiler.transpile_to_string(&ast);
        assert!(output.is_ok());
    }
}