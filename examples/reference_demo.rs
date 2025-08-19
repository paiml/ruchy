//! Demo of reference operator functionality

#![allow(clippy::print_stdout)]
#![allow(clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

use ruchy::runtime::Repl;

fn run_demo(repl: &mut Repl, desc: &str, code: &str) {
    print!("{:<20} : {} => ", desc, code);
    match repl.eval(code) {
        Ok(val) => println!("{:?}", val),
        Err(e) => println!("ERROR: {}", e),
    }
}

fn main() {
    println!("Reference Operator Demo");
    println!("=======================\n");
    
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    run_demo(&mut repl, "Basic reference", "&42");
    run_demo(&mut repl, "Reference to sum", "&(10 + 20)");
    run_demo(&mut repl, "Reference to bool", "&true");
    run_demo(&mut repl, "Bitwise AND", "5 & 3");
    run_demo(&mut repl, "Reference vs bitwise", "&7 & 1");
}