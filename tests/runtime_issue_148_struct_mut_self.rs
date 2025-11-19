/// RUNTIME-ISSUE-148: &mut self mutations don't persist in interpreter
///
/// ROOT CAUSE: To be determined via Five Whys
///
/// EVIDENCE:
/// - Compiled binary works correctly (outputs 15)
/// - Interpreter shows mutations happening (logs show 5, 10)
/// - But final `get()` returns 0 instead of 10
///
/// EXTREME TDD Test Suite
/// Coverage:
/// - Single &mut self mutation
/// - Multiple &mut self mutations
/// - Mixed &self and &mut self calls
/// - Mutation followed by read
use ruchy::runtime::repl::Repl;
use std::path::PathBuf;

#[test]
fn test_issue_148_01_single_mut_self_mutation() {
    let code = r"
struct Counter {
    value: i32,

    pub fun new() -> Counter {
        Counter { value: 0 }
    }

    pub fun increment(&mut self) {
        self.value = self.value + 1
    }

    pub fun get(&self) -> i32 {
        self.value
    }
}

let mut counter = Counter::new()
counter.increment()
counter.get()
";

    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
    let result = repl.eval(code).expect("Should execute");

    // RED: This will fail - interpreter returns 0 instead of 1
    assert_eq!(result, "1", "After increment(), get() should return 1");
}

#[test]
fn test_issue_148_02_multiple_mut_self_mutations() {
    let code = r"
struct Calculator {
    value: i32,

    pub fun new() -> Calculator {
        Calculator { value: 0 }
    }

    pub fun add(&mut self, amount: i32) {
        self.value = self.value + amount
    }

    pub fun get(&self) -> i32 {
        self.value
    }
}

let mut calc = Calculator::new()
calc.add(5)
calc.add(10)
calc.get()
";

    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
    let result = repl.eval(code).expect("Should execute");

    // RED: This will fail - interpreter returns 0 instead of 15
    assert_eq!(
        result, "15",
        "After add(5) and add(10), get() should return 15"
    );
}

#[test]
fn test_issue_148_03_mutation_persists_across_method_calls() {
    let code = r"
struct Point {
    x: i32,
    y: i32,

    pub fun new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }

    pub fun move_x(&mut self, dx: i32) {
        self.x = self.x + dx
    }

    pub fun get_x(&self) -> i32 {
        self.x
    }
}

let mut p = Point::new(0, 0)
p.move_x(3)
p.move_x(7)
p.get_x()
";

    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
    let result = repl.eval(code).expect("Should execute");

    // RED: This will fail - should return 10 (0 + 3 + 7)
    assert_eq!(
        result, "10",
        "After move_x(3) and move_x(7), get_x() should return 10"
    );
}

#[test]
fn test_issue_148_04_mixed_self_mut_self_calls() {
    let code = r"
struct Account {
    balance: i32,

    pub fun new(initial: i32) -> Account {
        Account { balance: initial }
    }

    pub fun deposit(&mut self, amount: i32) {
        self.balance = self.balance + amount
    }

    pub fun get_balance(&self) -> i32 {
        self.balance
    }

    pub fun withdraw(&mut self, amount: i32) {
        self.balance = self.balance - amount
    }
}

let mut acc = Account::new(100)
let initial = acc.get_balance()
acc.deposit(50)
let after_deposit = acc.get_balance()
acc.withdraw(30)
acc.get_balance()
";

    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
    let result = repl.eval(code).expect("Should execute");

    // RED: Should return 120 (100 + 50 - 30)
    assert_eq!(result, "120", "Final balance should be 120");
}

#[test]
fn test_issue_148_05_self_mutation_via_assignment() {
    let code = r"
struct Temperature {
    celsius: i32,

    pub fun new() -> Temperature {
        Temperature { celsius: 0 }
    }

    pub fun set(&mut self, value: i32) {
        self.celsius = value
    }

    pub fun get(&self) -> i32 {
        self.celsius
    }
}

let mut temp = Temperature::new()
temp.set(25)
temp.get()
";

    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
    let result = repl.eval(code).expect("Should execute");

    // RED: Should return 25
    assert_eq!(result, "25", "After set(25), get() should return 25");
}

#[test]
fn test_issue_148_06_multiple_instances_isolated() {
    let code = r"
struct Counter {
    count: i32,

    pub fun new() -> Counter {
        Counter { count: 0 }
    }

    pub fun inc(&mut self) {
        self.count = self.count + 1
    }

    pub fun get(&self) -> i32 {
        self.count
    }
}

let mut c1 = Counter::new()
let mut c2 = Counter::new()
c1.inc()
c1.inc()
c2.inc()
c1.get()
";

    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
    let result = repl.eval(code).expect("Should execute");

    // RED: c1 should be 2, c2 should be 1
    assert_eq!(result, "2", "c1 should be 2 after two inc() calls");
}
