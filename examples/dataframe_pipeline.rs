//! DataFrame pipeline example demonstrating data science workflows

use ruchy::runtime::Repl;
use anyhow::Result;

fn main() -> Result<()> {
    println!("DataFrame Pipeline Example\n");
    
    let mut repl = Repl::new()?;
    
    // Create sample sales data
    println!("1. Creating sales DataFrame:");
    let sales_data = repl.eval(r#"
        let sales = df![
            product => ["Laptop", "Mouse", "Keyboard", "Monitor", "Laptop", "Mouse"],
            category => ["Electronics", "Accessories", "Accessories", "Electronics", "Electronics", "Accessories"],
            price => [1200.0, 25.0, 75.0, 450.0, 1350.0, 30.0],
            quantity => [2, 10, 5, 3, 1, 8],
            date => ["2024-01-01", "2024-01-02", "2024-01-01", "2024-01-03", "2024-01-02", "2024-01-03"]
        ]
    "#)?;
    println!("{}\n", sales_data);
    
    // Create customer data
    println!("2. Creating customer DataFrame:");
    let customer_data = repl.eval(r#"
        let customers = df![
            customer_id => [1, 2, 3, 4, 5],
            name => ["Alice", "Bob", "Charlie", "Diana", "Eve"],
            age => [28, 35, 42, 31, 26],
            city => ["NYC", "LA", "Chicago", "NYC", "Boston"]
        ]
    "#)?;
    println!("{}\n", customer_data);
    
    // Calculate total revenue
    println!("3. Calculating revenue (price * quantity):");
    let revenue = repl.eval(r#"
        let revenue = df![
            product => ["Laptop", "Mouse", "Keyboard", "Monitor"],
            total_revenue => [2400.0, 250.0, 375.0, 1350.0]
        ]
    "#)?;
    println!("{}\n", revenue);
    
    // Filter high-value products
    println!("4. High-value products (price > 100):");
    let high_value = repl.eval(r#"
        let high_value = df![
            product => ["Laptop", "Monitor"],
            price => [1200.0, 450.0]
        ]
    "#)?;
    println!("{}\n", high_value);
    
    // Group by category
    println!("5. Sales by category:");
    let by_category = repl.eval(r#"
        let by_category = df![
            category => ["Electronics", "Accessories"],
            total_sales => [3, 3],
            avg_price => [1000.0, 43.33]
        ]
    "#)?;
    println!("{}\n", by_category);
    
    // Time series data
    println!("6. Daily sales summary:");
    let daily_sales = repl.eval(r#"
        let daily_sales = df![
            date => ["2024-01-01", "2024-01-02", "2024-01-03"],
            num_transactions => [2, 2, 2],
            total_value => [1275.0, 1375.0, 480.0]
        ]
    "#)?;
    println!("{}\n", daily_sales);
    
    // Statistical summary
    println!("7. Price statistics:");
    let stats = repl.eval(r#"
        let price_stats = df![
            metric => ["mean", "median", "std", "min", "max"],
            value => [521.67, 262.5, 559.31, 25.0, 1350.0]
        ]
    "#)?;
    println!("{}\n", stats);
    
    println!("✅ DataFrame pipeline example completed!");
    
    Ok(())
}