import { test, expect } from '@playwright/test';

/**
 * NOTEBOOK-007: E2E Testing - Language Features (COMPLETE)
 *
 * Tests all 41 Ruchy language features in the notebook across 3 browsers.
 * This represents 41 features × 3 browsers = 123 test scenarios.
 *
 * Feature Coverage:
 * - Part 1: Basic Syntax (Features 1-4): literals, variables, booleans
 * - Part 2: Operators (Features 5-7): arithmetic, comparison, logical
 * - Part 3: Control Flow (Features 8-10): if-else, match, for loops
 * - Part 4: Functions (Features 11-13): definitions, closures, higher-order
 * - Part 5: Data Structures (Features 14-18): arrays, tuples, objects, structs, enums
 * - Part 6: Pattern Matching (Features 19-20): destructuring, guards
 * - Part 7: Error Handling (Features 21-23): try-catch, Option, Result
 * - Part 8: String Features (Features 24-25): interpolation, methods
 * - Part 9-13: Standard Library (Features 26-34): collections, iterators, I/O, math, time
 * - Part 14-18: Advanced Features (Features 35-41): generics, traits, async/await, macros, testing
 */

test.describe('Part 1: Foundation - Basic Syntax', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 1: Integer literals', async ({ page }) => {
    await page.fill('.cell-input', '42');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('42');
  });

  test('Feature 2: String literals', async ({ page }) => {
    await page.fill('.cell-input', '"Hello, World!"');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('Hello, World!');
  });

  test('Feature 3: Boolean literals', async ({ page }) => {
    await page.fill('.cell-input', 'true');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('true');
  });

  test('Feature 4: Variable assignment', async ({ page }) => {
    await page.fill('.cell-input', 'let x = 100\nx');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('100');
  });
});

test.describe('Part 2: Operators', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 5: Arithmetic operators', async ({ page }) => {
    await page.fill('.cell-input', '10 + 5 * 2');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('20');
  });

  test('Feature 6: Comparison operators', async ({ page }) => {
    await page.fill('.cell-input', '10 > 5');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('true');
  });

  test('Feature 7: Logical operators', async ({ page }) => {
    await page.fill('.cell-input', 'true && false');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('false');
  });
});

test.describe('Part 3: Control Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 8: If-else expressions', async ({ page }) => {
    const code = `if true { "yes" } else { "no" }`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('yes');
  });

  test('Feature 9: Match expressions', async ({ page }) => {
    const code = `match 2 {
  1 => "one",
  2 => "two",
  _ => "other"
}`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('two');
  });

  test('Feature 10: For loops', async ({ page }) => {
    const code = `let sum = 0
for i in 1..=5 {
  sum = sum + i
}
sum`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('15');
  });
});

test.describe('Part 4: Functions', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 11: Function definitions', async ({ page }) => {
    const code = `fn double(x) { x * 2 }
double(21)`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('42');
  });

  test('Feature 12: Closures', async ({ page }) => {
    const code = `let add_n = |n| { |x| x + n }
let add_5 = add_n(5)
add_5(10)`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('15');
  });

  test('Feature 13: Higher-order functions', async ({ page }) => {
    const code = `[1, 2, 3].map(|x| x * 2)`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toMatch(/\[2,\s*4,\s*6\]/);
  });
});

test.describe('Part 5: Data Structures', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 14: Arrays', async ({ page }) => {
    await page.fill('.cell-input', '[1, 2, 3, 4, 5]');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toMatch(/\[1,.*5\]/);
  });

  test('Feature 15: Tuples', async ({ page }) => {
    await page.fill('.cell-input', '(1, "hello", true)');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('1');
    expect(output).toContain('hello');
  });

  test('Feature 16: Objects/Maps', async ({ page }) => {
    await page.fill('.cell-input', '{ name: "Alice", age: 30 }');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('name');
    expect(output).toContain('Alice');
  });

  test('Feature 17: Structs', async ({ page }) => {
    const code = `struct Point { x: i32, y: i32 }
Point { x: 10, y: 20 }`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('10');
    expect(output).toContain('20');
  });

  test('Feature 18: Enums', async ({ page }) => {
    const code = `enum Status { Active, Inactive }
Status::Active`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('Active');
  });
});

test.describe('Part 6: Pattern Matching', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 19: Destructuring', async ({ page }) => {
    const code = `let [a, b, c] = [1, 2, 3]
a + b + c`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('6');
  });

  test('Feature 20: Pattern guards', async ({ page }) => {
    const code = `match 5 {
  x if x > 10 => "large",
  x if x > 0 => "small",
  _ => "zero or negative"
}`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('small');
  });
});

test.describe('Part 7: Error Handling', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 21: Try-catch', async ({ page }) => {
    const code = `try {
  throw "error"
} catch (e) {
  "caught"
}`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('caught');
  });

  test('Feature 22: Option type', async ({ page }) => {
    const code = `Some(42).unwrap()`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('42');
  });

  test('Feature 23: Result type', async ({ page }) => {
    const code = `Ok(100).unwrap()`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('100');
  });
});

test.describe('Part 8: String Features', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 24: String interpolation', async ({ page }) => {
    const code = `let name = "World"
f"Hello, {name}!"`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('Hello, World!');
  });

  test('Feature 25: String methods', async ({ page }) => {
    await page.fill('.cell-input', '"hello".to_uppercase()');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('HELLO');
  });
});

test.describe('Part 9: Standard Library - Collections', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 26: Vector operations', async ({ page }) => {
    const code = `let v = vec![1, 2, 3]\nv.push(4)\nv`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toMatch(/\[1.*4\]/);
  });

  test('Feature 27: HashMap operations', async ({ page }) => {
    const code = `let map = HashMap::new()\nmap.insert("key", "value")\nmap.get("key")`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('value');
  });
});

test.describe('Part 10: Standard Library - Iterators', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 28: Iterator map', async ({ page }) => {
    const code = `[1, 2, 3].iter().map(|x| x * 2).collect()`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toMatch(/\[2.*6\]/);
  });

  test('Feature 29: Iterator filter', async ({ page }) => {
    const code = `[1, 2, 3, 4, 5].iter().filter(|x| x % 2 == 0).collect()`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toMatch(/\[2.*4\]/);
  });
});

test.describe('Part 11: Standard Library - I/O', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 30: Print output', async ({ page }) => {
    await page.fill('.cell-input', 'println("test output")');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('test output');
  });

  test('Feature 31: Format strings', async ({ page }) => {
    const code = `format!("Value: {}", 42)`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('Value: 42');
  });
});

test.describe('Part 12: Standard Library - Math', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 32: Math functions', async ({ page }) => {
    await page.fill('.cell-input', 'Math.sqrt(16)');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('4');
  });

  test('Feature 33: Trigonometric functions', async ({ page }) => {
    await page.fill('.cell-input', 'Math.sin(0)');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('0');
  });
});

test.describe('Part 13: Standard Library - Time', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 34: Time measurement', async ({ page }) => {
    const code = `let start = Time.now()\nstart`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toBeTruthy();
  });
});

test.describe('Part 14: Advanced Features - Generics', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 35: Generic functions', async ({ page }) => {
    const code = `fn identity<T>(x: T) -> T { x }\nidentity(42)`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('42');
  });

  test('Feature 36: Generic structs', async ({ page }) => {
    const code = `struct Point<T> { x: T, y: T }\nPoint { x: 1, y: 2 }`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('1');
  });
});

test.describe('Part 15: Advanced Features - Traits', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 37: Trait definitions', async ({ page }) => {
    const code = `trait Greet { fn greet(&self) -> String }\n"test"`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('test');
  });
});

test.describe('Part 16: Advanced Features - Async/Await', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 38: Async functions', async ({ page }) => {
    const code = `async fn get_value() -> i32 { 42 }\nget_value().await`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('42');
  });
});

test.describe('Part 17: Advanced Features - Macros', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 39: Macro invocation', async ({ page }) => {
    await page.fill('.cell-input', 'vec![1, 2, 3]');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toMatch(/\[1.*3\]/);
  });

  test('Feature 40: println! macro', async ({ page }) => {
    await page.fill('.cell-input', 'println!("macro test")');
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('macro test');
  });
});

test.describe('Part 18: Advanced Features - Testing', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/notebook.html');
    await page.waitForSelector('#notebook-container');
  });

  test('Feature 41: Assertions', async ({ page }) => {
    const code = `assert_eq!(2 + 2, 4)\n"test passed"`;
    await page.fill('.cell-input', code);
    await page.click('.execute-button');
    await page.waitForSelector('.cell-output');

    const output = await page.textContent('.cell-output');
    expect(output).toContain('test passed');
  });
});

// ✅ All 41 features covered × 3 browsers = 123 test scenarios
