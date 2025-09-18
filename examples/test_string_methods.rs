use ruchy::runtime::Repl;

fn main() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    println!("Testing string methods:\n");
    
    let tests = vec![
        ("\"hello\"", "String literal"),
        ("\"hello\".length()", "String length"),
        ("\"hello\".toUpperCase()", "To uppercase"),
        ("\"HELLO\".toLowerCase()", "To lowercase"),
        ("\"  trim  \".trim()", "Trim"),
        ("\"hello world\".split(\" \")", "Split"),
    ];
    
    for (expr, desc) in tests {
        println!("{desc}: > {expr}");
        match repl.eval(expr) {
            Ok(v) => println!("   OK: {v}"),
            Err(e) => println!("   ERROR: {e}"),
        }
        println!();
    }
}