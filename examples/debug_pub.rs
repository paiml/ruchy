use ruchy::frontend::parser::Parser;
use ruchy::frontend::ast::ExprKind;

fn main() {
    let code = r#"
mod math {
    pub fun add(a: i32, b: i32) -> i32 {
        a + b
    }
}
"#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    // Find the module
    println!("AST kind: {:?}", std::mem::discriminant(&ast.kind));
    
    match &ast.kind {
        ExprKind::Module { name, body } => {
            println!("Direct Module: {}", name);
            if let ExprKind::Block(funcs) = &body.kind {
                for func in funcs {
                    if let ExprKind::Function { name, is_pub, .. } = &func.kind {
                        println!("  Function: {} (is_pub: {})", name, is_pub);
                    }
                }
            }
        }
        ExprKind::Block(exprs) => {
            println!("Block with {} expressions", exprs.len());
            for (i, expr) in exprs.iter().enumerate() {
                println!("  Expr {}: {:?}", i, std::mem::discriminant(&expr.kind));
                if let ExprKind::Module { name, body } = &expr.kind {
                    println!("  Module: {}", name);
                    if let ExprKind::Block(funcs) = &body.kind {
                        for func in funcs {
                            if let ExprKind::Function { name, is_pub, .. } = &func.kind {
                                println!("    Function: {} (is_pub: {})", name, is_pub);
                            }
                        }
                    }
                }
            }
        }
        _ => println!("Other kind")
    }
}