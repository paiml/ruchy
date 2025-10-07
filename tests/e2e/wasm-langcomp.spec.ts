import { test, expect } from '@playwright/test';

/**
 * WASM LANG-COMP E2E Tests
 *
 * Tests that compiled WASM from LANG-COMP examples executes correctly
 * in browser environments.
 *
 * Strategy: Load WASM module, call exported functions, verify results
 */

test.describe('WASM LANG-COMP Execution Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });
  });

  test('should execute simple arithmetic from compiled WASM', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Load and execute arithmetic example
    await input.fill('10 + 20');
    await input.press('Enter');

    // Should show result
    await expect(output).toContainText('30');
  });

  test('should execute variables from compiled WASM', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute variable declaration and usage
    await input.fill('let x = 42');
    await input.press('Enter');

    await input.fill('x * 2');
    await input.press('Enter');

    // Should show result
    await expect(output).toContainText('84');
  });

  test('should execute if expressions from compiled WASM', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute if expression
    await input.fill('if 10 > 5 then 100 else 200');
    await input.press('Enter');

    // Should show true branch result
    await expect(output).toContainText('100');
  });

  test('should execute match expressions from compiled WASM', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute match expression
    await input.fill('match 2 { 1 => 10, 2 => 20, _ => 30 }');
    await input.press('Enter');

    // Should match second arm
    await expect(output).toContainText('20');
  });

  test('should execute user-defined functions from compiled WASM', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Define function
    await input.fill('fn double(x) { return x * 2 }');
    await input.press('Enter');

    // Call function
    await input.fill('double(21)');
    await input.press('Enter');

    // Should show result
    await expect(output).toContainText('42');
  });

  test('should execute closures from compiled WASM', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Define closure
    await input.fill('let triple = |x| x * 3');
    await input.press('Enter');

    // Call closure
    await input.fill('triple(14)');
    await input.press('Enter');

    // Should show result
    await expect(output).toContainText('42');
  });

  test('should execute f-string expressions from compiled WASM', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // F-string with expression
    await input.fill('let x = 10');
    await input.press('Enter');

    await input.fill('let y = 20');
    await input.press('Enter');

    // F-string evaluates expression
    await input.fill('println(f"{x + y}")');
    await input.press('Enter');

    // Should show computed value (30)
    await expect(output).toContainText('30');
  });

  test('should execute while loops from compiled WASM', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // While loop that counts
    await input.fill('let i = 0');
    await input.press('Enter');

    await input.fill('while i < 3 { i = i + 1 }');
    await input.press('Enter');

    await input.fill('i');
    await input.press('Enter');

    // Should show final value
    await expect(output).toContainText('3');
  });

  test('should handle complex expressions from compiled WASM', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Complex expression with precedence
    await input.fill('2 + 3 * 4');
    await input.press('Enter');

    // Should respect precedence: 2 + (3 * 4) = 14
    await expect(output).toContainText('14');
  });

  test('should execute comparison operations from compiled WASM', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Comparison
    await input.fill('10 > 5');
    await input.press('Enter');

    // Should show true (1 in WASM i32)
    await expect(output).toContainText('1');

    await input.fill('10 < 5');
    await input.press('Enter');

    // Should show false (0 in WASM i32)
    await expect(output).toContainText('0');
  });

  test('should execute logical operations from compiled WASM', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Logical AND
    await input.fill('1 && 1');
    await input.press('Enter');

    await expect(output).toContainText('1');

    // Logical OR
    await input.fill('0 || 1');
    await input.press('Enter');

    await expect(output).toContainText('1');
  });
});
