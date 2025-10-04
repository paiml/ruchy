import { test, expect } from '@playwright/test';

test.describe('Ruchy REPL WASM E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Clear localStorage before each test
    await page.goto('/');
    await page.evaluate(() => localStorage.clear());
    await page.reload();
  });

  test('should load WASM and show ready status', async ({ page }) => {
    await page.goto('/');

    // Wait for WASM to load (status changes from loading to ready)
    const status = page.locator('#status');
    await expect(status).toHaveClass(/status-ready/, { timeout: 10000 });
    await expect(status).toHaveText('Ready');

    // Input should be enabled
    const input = page.locator('#repl-input');
    await expect(input).toBeEnabled();

    // Should show welcome message
    await expect(page.locator('#output')).toContainText('Welcome to Ruchy REPL');
  });

  test('should execute :help command and display help', async ({ page }) => {
    await page.goto('/');

    // Wait for ready
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute :help command
    await input.fill(':help');
    await input.press('Enter');

    // Should show command and help output
    await expect(output).toContainText(':help');
    await expect(output).toContainText('Available commands');
  });

  test('should execute :clear command and clear output', async ({ page }) => {
    await page.goto('/');

    // Wait for ready
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute some command first
    await input.fill(':help');
    await input.press('Enter');

    // Verify help is displayed
    await expect(output).toContainText('Available commands');

    // Now clear
    await input.fill(':clear');
    await input.press('Enter');

    // Output should be cleared (only welcome message)
    const outputText = await output.textContent();
    expect(outputText).toContain('Welcome to Ruchy REPL');
    expect(outputText).not.toContain('Available commands');
  });

  test('should persist REPL history in localStorage', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');

    // Execute multiple commands
    await input.fill(':help');
    await input.press('Enter');
    await input.fill(':clear');
    await input.press('Enter');

    // Reload page
    await page.reload();
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    // History should be restored
    const output = page.locator('#output');
    await expect(output).toContainText(':help');
    await expect(output).toContainText(':clear');
    await expect(output).toContainText('(from history)');
  });

  test('should navigate command history with arrow keys', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');

    // Execute commands
    await input.fill('first command');
    await input.press('Enter');
    await input.fill('second command');
    await input.press('Enter');
    await input.fill('third command');
    await input.press('Enter');

    // Navigate history with ArrowUp
    await input.press('ArrowUp');
    await expect(input).toHaveValue('third command');

    await input.press('ArrowUp');
    await expect(input).toHaveValue('second command');

    await input.press('ArrowUp');
    await expect(input).toHaveValue('first command');

    // Navigate down
    await input.press('ArrowDown');
    await expect(input).toHaveValue('second command');

    await input.press('ArrowDown');
    await expect(input).toHaveValue('third command');

    await input.press('ArrowDown');
    await expect(input).toHaveValue('');
  });

  test('should clear history with Clear History button', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute commands
    await input.fill('test command');
    await input.press('Enter');

    // Clear history
    await page.locator('#clear-history').click();

    // Output should be cleared and show welcome
    await expect(output).toContainText('Welcome to Ruchy REPL');
    await expect(output).toContainText('History cleared');
    await expect(output).not.toContainText('test command');

    // History should be cleared from localStorage
    const historyData = await page.evaluate(() => localStorage.getItem('ruchy-repl-history'));
    expect(historyData).toBeNull();
  });

  test('should reset environment with Reset Environment button', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute command
    await input.fill('test command');
    await input.press('Enter');

    // Reset environment
    await page.locator('#reset-env').click();

    // Output should be cleared and show welcome
    await expect(output).toContainText('Welcome to Ruchy REPL');
    await expect(output).toContainText('Environment reset');
  });

  test('should work offline after initial load', async ({ page, context }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute command to verify it works
    await input.fill(':help');
    await input.press('Enter');
    await expect(output).toContainText('Available commands');

    // Go offline
    await context.setOffline(true);

    // Execute commands while offline - should still work
    await input.fill(':clear');
    await input.press('Enter');
    await expect(output).toContainText('Output cleared');

    // Go back online
    await context.setOffline(false);

    // Should still work
    await input.fill(':help');
    await input.press('Enter');
    await expect(output).toContainText('Available commands');
  });

  test('should handle rapid command execution without race conditions', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute multiple commands rapidly
    for (let i = 1; i <= 10; i++) {
      await input.fill(`command${i}`);
      await input.press('Enter');
    }

    // All commands should be executed and displayed
    for (let i = 1; i <= 10; i++) {
      await expect(output).toContainText(`command${i}`);
    }
  });

  test('should parse simple expressions', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Test arithmetic expression
    await input.fill('2 + 2');
    await input.press('Enter');

    // Should show the expression and parsed AST
    await expect(output).toContainText('2 + 2');
    // Should show Binary operation in AST (current WASM REPL behavior)
    await expect(output).toContainText('Binary', { timeout: 2000 });
    await expect(output).toContainText('Integer(2)');
  });

  test('should parse variable declarations', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Declare a variable
    await input.fill('let x = 42');
    await input.press('Enter');

    // Should show the declaration and parsed AST
    await expect(output).toContainText('let x = 42');
    // Should show parsed structure (current WASM REPL behavior)
    await expect(output).toContainText('Let');
    await expect(output).toContainText('name: "x"');
    await expect(output).toContainText('Integer(42)');
  });

  test('should parse function definitions', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Define a function
    await input.fill('fun double(n) { n * 2 }');
    await input.press('Enter');

    // Should show the function definition and parsed AST
    await expect(output).toContainText('fun double');
    // Should show Function in parsed structure (current WASM REPL behavior)
    await expect(output).toContainText('Function');
    await expect(output).toContainText('name: "double"');
  });

  test('should handle parse errors gracefully', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#status')).toHaveClass(/status-ready/, { timeout: 10000 });

    const input = page.locator('#repl-input');
    const output = page.locator('#output');

    // Execute invalid syntax
    await input.fill('let x = ');
    await input.press('Enter');

    // The REPL should show the input and an error
    await expect(output).toContainText('let x = ');
    // Should show some form of error indication
    await expect(output).toContainText('Error', { timeout: 2000 });
  });
});
