import { test, expect } from '@playwright/test';

/**
 * Matrix Test 01: Simple Arithmetic Operations (WASM Platform)
 *
 * This test verifies basic arithmetic works correctly in the WASM REPL.
 * Companion native test: tests/matrix/native/01_simple_arithmetic_native.rs
 *
 * Goal: Ensure identical behavior between WASM and native platforms
 */

test.describe('Matrix Test 01: Simple Arithmetic (WASM Platform)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for WASM to load
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });
  });

  test('should compute 10 + 20 = 30', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute arithmetic
    await input.fill('10 + 20');
    await input.press('Enter');

    // Verify output contains result
    await expect(output).toContainText('30');
  });

  test('should compute 100 - 42 = 58', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    await input.fill('100 - 42');
    await input.press('Enter');

    await expect(output).toContainText('58');
  });

  test('should compute 6 * 7 = 42', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    await input.fill('6 * 7');
    await input.press('Enter');

    await expect(output).toContainText('42');
  });

  test('should compute 100 / 4 = 25', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    await input.fill('100 / 4');
    await input.press('Enter');

    await expect(output).toContainText('25');
  });

  test('should handle operator precedence: 2 + 3 * 4 = 14', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    await input.fill('2 + 3 * 4');
    await input.press('Enter');

    // Should respect precedence: 2 + (3 * 4) = 14, not (2 + 3) * 4 = 20
    await expect(output).toContainText('14');
  });

  test('should handle parentheses: (2 + 3) * 4 = 20', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    await input.fill('(2 + 3) * 4');
    await input.press('Enter');

    await expect(output).toContainText('20');
  });

  test('should handle variables: let x = 10, x * 2 = 20', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Declare variable
    await input.fill('let x = 10');
    await input.press('Enter');

    // Use variable
    await input.fill('x * 2');
    await input.press('Enter');

    await expect(output).toContainText('20');
  });

  test('should handle multi-step computation', async ({ page }) => {
    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Step 1: Define variables
    await input.fill('let a = 5');
    await input.press('Enter');

    await input.fill('let b = 10');
    await input.press('Enter');

    await input.fill('let c = 15');
    await input.press('Enter');

    // Step 2: Compute result
    await input.fill('a + b + c');
    await input.press('Enter');

    // Should be 5 + 10 + 15 = 30
    await expect(output).toContainText('30');
  });
});
