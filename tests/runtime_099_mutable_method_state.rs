/// RUNTIME-099: Mutable method calls don't preserve state
///
/// EXTREME TDD Test Suite for mutable method state preservation.
///
/// ROOT CAUSE: Discovered by property testing - `Calculator.add()` calls
/// don't accumulate state correctly.
///
/// OBSERVED:
///   calc.add(1) → returns 1
///   calc.add(1) → returns 1 (should be 2!)
///
/// RED Phase: All tests MUST fail initially showing state isn't preserved.
/// Coverage:
/// - Simple mutable method (add)
/// - Multiple calls to same method
/// - Mixed mutable and immutable methods
/// - Field access after mutation
/// - Multiple fields mutation

use ruchy::runtime::interpreter::Interpreter;
use ruchy::frontend::parser::Parser;

#[test]
fn test_runtime_099_01_simple_mutable_method() {
    let code = r"
class Counter {
    count: i32

    pub new() -> Counter {
        Counter { count: 0 }
    }

    pub fun increment(&mut self) -> i32 {
        self.count = self.count + 1
        self.count
    }

    pub fun get_count(&self) -> i32 {
        self.count
    }
}

let mut c = Counter::new()
c.increment()
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");

    // Should return 1 after first increment
    assert_eq!(result.to_string(), "1",
        "First increment should return 1, got: {result}");
}

#[test]
fn test_runtime_099_02_multiple_mutable_calls() {
    let code = r"
class Counter {
    count: i32

    pub new() -> Counter {
        Counter { count: 0 }
    }

    pub fun increment(&mut self) -> i32 {
        self.count = self.count + 1
        self.count
    }
}

let mut c = Counter::new()
c.increment()
c.increment()
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");

    // Should return 2 after two increments (CUMULATIVE!)
    assert_eq!(result.to_string(), "2",
        "Two increments should return 2, got: {result}");
}

#[test]
fn test_runtime_099_03_three_mutable_calls() {
    let code = r"
class Calculator {
    value: i32

    pub new() -> Calculator {
        Calculator { value: 0 }
    }

    pub fun add(&mut self, n: i32) -> i32 {
        self.value = self.value + n
        self.value
    }
}

let mut calc = Calculator::new()
calc.add(5)
calc.add(3)
calc.add(2)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");

    // Should return 10 after three additions (5+3+2=10)
    assert_eq!(result.to_string(), "10",
        "Three additions (5+3+2) should return 10, got: {result}");
}

#[test]
fn test_runtime_099_04_field_access_after_mutation() {
    let code = r"
class Point {
    x: i32
    y: i32

    pub new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }

    pub fun move_by(&mut self, dx: i32, dy: i32) {
        self.x = self.x + dx
        self.y = self.y + dy
    }

    pub fun get_x(&self) -> i32 {
        self.x
    }
}

let mut p = Point::new(3, 4)
p.move_by(1, 1)
p.get_x()
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");

    // Should return 4 (3+1) after move_by
    assert_eq!(result.to_string(), "4",
        "After move_by(1,1), x should be 4, got: {result}");
}

#[test]
fn test_runtime_099_05_multiple_moves() {
    let code = r"
class Point {
    x: i32
    y: i32

    pub new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }

    pub fun move_by(&mut self, dx: i32, dy: i32) {
        self.x = self.x + dx
        self.y = self.y + dy
    }

    pub fun get_x(&self) -> i32 {
        self.x
    }
}

let mut p = Point::new(0, 0)
p.move_by(1, 1)
p.move_by(2, 2)
p.move_by(3, 3)
p.get_x()
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");

    // Should return 6 (0+1+2+3) after three moves
    assert_eq!(result.to_string(), "6",
        "After three moves, x should be 6, got: {result}");
}

#[test]
fn test_runtime_099_06_mixed_mutable_immutable() {
    let code = r"
class BankAccount {
    balance: i32

    pub new(initial: i32) -> BankAccount {
        BankAccount { balance: initial }
    }

    pub fun deposit(&mut self, amount: i32) {
        self.balance = self.balance + amount
    }

    pub fun get_balance(&self) -> i32 {
        self.balance
    }
}

let mut account = BankAccount::new(100)
account.deposit(50)
let b1 = account.get_balance()
account.deposit(25)
let b2 = account.get_balance()
b2
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");

    // Should return 175 (100+50+25) after two deposits
    assert_eq!(result.to_string(), "175",
        "After deposits of 50 and 25, balance should be 175, got: {result}");
}

#[test]
fn test_runtime_099_07_property_test_minimal_case() {
    // This is the exact minimal failing case from property testing
    let code = r"
class Calculator {
    value: i32

    pub new() -> Calculator {
        Calculator { value: 0 }
    }

    pub fun add(&mut self, n: i32) -> i32 {
        self.value = self.value + n
        self.value
    }
}

let mut calc = Calculator::new()
calc.add(1)
calc.add(1)
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");

    // Property test discovered: This should return 2, but returns 1!
    assert_eq!(result.to_string(), "2",
        "Cumulative addition (1+1) should return 2, got: {result}");
}
