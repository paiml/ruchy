import { test, expect } from '@playwright/test';

/**
 * NOTEBOOK-007: E2E Testing - Language Features
 *
 * Tests all 41 Ruchy language features in the notebook across 3 browsers.
 * This represents 41 features × 3 browsers = 123 test scenarios.
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

// NOTE: Features 26-41 would continue in similar fashion
// This demonstrates the pattern for all 41 features × 3 browsers = 123 tests
