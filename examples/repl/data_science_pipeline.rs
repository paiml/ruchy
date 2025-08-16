//! Data Science Pipeline Example for Ruchy REPL
//!
//! This example demonstrates interactive data analysis workflows using the Ruchy REPL.
//! Run with: cargo run --example data_science_pipeline

use anyhow::Result;
use ruchy::runtime::repl::Repl;

fn main() -> Result<()> {
    println!("🔬 Ruchy REPL: Data Science Pipeline Example");
    println!("============================================");
    
    let mut repl = Repl::new()?;
    
    println!("\n📊 Step 1: Create and manipulate data");
    demonstrate_step(&mut repl, "let data = [1, 2, 3, 4, 5]")?;
    demonstrate_step(&mut repl, "let doubled = [x * 2 for x in data]")?;
    demonstrate_step(&mut repl, "doubled")?;
    
    println!("\n🔢 Step 2: Define analysis functions");
    demonstrate_step(&mut repl, "fun mean(numbers: List<i32>) -> f64 { numbers.sum() / numbers.len() }")?;
    demonstrate_step(&mut repl, "fun variance(numbers: List<i32>) -> f64 { /* simplified */ 0.0 }")?;
    
    println!("\n📈 Step 3: Apply statistical operations");
    demonstrate_step(&mut repl, "let avg = mean(data)")?;
    demonstrate_step(&mut repl, "let filtered = [x for x in data if x > 2]")?;
    
    println!("\n📋 Step 4: Create summary DataFrame");
    demonstrate_step(&mut repl, r#"
        let summary = df![
            "metric" => ["count", "mean", "max"];
            "value" => [data.len(), avg, data.max()]
        ]
    "#)?;
    
    println!("\n📜 Step 5: Review session history");
    println!("History:");
    println!("{}", repl.show_history());
    
    println!("\n✅ Data science pipeline completed!");
    Ok(())
}

fn demonstrate_step(repl: &mut Repl, code: &str) -> Result<()> {
    println!("🟢 Executing: {}", code);
    match repl.eval(code) {
        Ok(result) => {
            if !result.trim().is_empty() {
                println!("   Result: {}", result);
            }
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }
    Ok(())
}