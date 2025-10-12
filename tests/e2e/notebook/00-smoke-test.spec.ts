import { test, expect } from '@playwright/test';

/**
 * NOTEBOOK-009-DEFECT: Smoke Test - Cell Execution Reality Check
 *
 * Purpose: Test the ACTUAL notebook UI (not phantom UI from old tests)
 * Created: 2025-10-12 (Response to user bug report)
 *
 * Five Whys Root Cause:
 * 1. Why is cell execution broken? → E2E tests disconnected from actual UI
 * 2. Why are E2E tests disconnected? → Tests written for old UI structure
 * 3. Why doesn't UI match tests? → UI updated without updating tests
 * 4. Why weren't tests updated? → No requirement to run E2E before deployment
 * 5. Why no E2E requirement? → E2E tests not in CI/CD, only unit tests run
 *
 * This test validates the REAL notebook at http://localhost:8080
 */

test.describe('Notebook Smoke Test - Reality Check', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to ACTUAL notebook URL
    await page.goto('http://localhost:8080');

    // Wait for notebook to load (FIXED: actual ID is #notebook-cells, not #notebook-container)
    await page.waitForSelector('#notebook-cells', { timeout: 10000 });
  });

  test('should load actual notebook interface', async ({ page }) => {
    // Verify REAL UI elements exist (FIXED: using actual IDs from HTML)
    await expect(page.locator('#notebook-cells')).toBeVisible();
    await expect(page.locator('#cell-type-selector')).toBeVisible();
    await expect(page.locator('#btn-add-cell')).toBeVisible();

    // Check for CodeMirror (actual editor)
    await expect(page.locator('.CodeMirror')).toBeVisible();
  });

  test('should execute code cell with Shift+Enter', async ({ page }) => {
    // Find the first CodeMirror instance
    const codeMirror = page.locator('.CodeMirror').first();
    await codeMirror.click();

    // Clear existing code and type new code
    await page.keyboard.press('Control+A');
    await page.keyboard.type('42');

    // Execute with Shift+Enter
    await page.keyboard.press('Shift+Enter');

    // Wait for output to appear
    await page.waitForSelector('.cell-output:visible', { timeout: 5000 });

    // Verify output contains result
    const output = await page.locator('.cell-output').first().textContent();
    expect(output).toContain('42');
  });

  test('should execute via API (backend verification)', async ({ page, request }) => {
    // Direct API test to prove backend works
    const response = await request.post('http://localhost:8080/api/execute', {
      data: {
        source: '2 + 2'
      }
    });

    const result = await response.json();
    expect(result.success).toBe(true);
    expect(result.output).toBe('4');
  });

  test('should create markdown cell', async ({ page }) => {
    // Select Markdown from dropdown
    await page.selectOption('#cell-type-selector', 'markdown');

    // Add cell
    await page.click('#btn-add-cell');

    // Verify markdown cell was created
    const markdownCells = await page.locator('.cell-type-markdown').count();
    expect(markdownCells).toBeGreaterThan(0);
  });

  test('should render markdown cell', async ({ page }) => {
    // Select Markdown
    await page.selectOption('#cell-type-selector', 'markdown');
    await page.click('#btn-add-cell');

    // Wait for markdown cell to be added
    await page.waitForTimeout(500);

    // Find the markdown cell's preview area
    const markdownPreview = page.locator('.markdown-preview').last();

    // Double-click to edit
    await markdownPreview.dblclick();

    // Wait for editor to appear
    await page.waitForSelector('.markdown-edit', { state: 'visible' });

    // Type markdown
    const markdownEditor = page.locator('.markdown-edit').last();
    await markdownEditor.fill('# Test Heading\n\nThis is **bold**');

    // Press Esc to render
    await page.keyboard.press('Escape');

    // Wait for rendering to complete
    await page.waitForTimeout(500);

    // Verify rendered output
    const rendered = await markdownPreview.innerHTML();
    expect(rendered).toContain('<h1>Test Heading</h1>');
    expect(rendered).toContain('<strong>bold</strong>');
  });
});

test.describe('Bug Reproduction - User Report', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('#notebook-cells');  // FIXED: actual ID
  });

  test('CRITICAL: Basic cell execution must work', async ({ page }) => {
    // This is the EXACT scenario the user reported broken

    // 1. Find first cell (welcome cell)
    const firstCell = page.locator('.cell').first();

    // 2. Click into CodeMirror editor
    await firstCell.locator('.CodeMirror').click();

    // 3. Clear and type simple code
    await page.keyboard.press('Control+A');
    await page.keyboard.type('println("Hello, World!")');

    // 4. Execute with Shift+Enter
    await page.keyboard.press('Shift+Enter');

    // 5. MUST see output within 5 seconds
    await page.waitForSelector('.cell-output:visible', { timeout: 5000 });

    // 6. Output MUST contain expected result
    const output = await firstCell.locator('.cell-output').textContent();
    expect(output).toContain('Hello, World!');
  });

  test('CRITICAL: Multiple cell execution must work', async ({ page }) => {
    // Execute cell 1
    const firstCell = page.locator('.cell').first();
    await firstCell.locator('.CodeMirror').click();
    await page.keyboard.press('Control+A');
    await page.keyboard.type('x = 10');
    await page.keyboard.press('Shift+Enter');

    // Wait for first cell execution to complete
    await page.waitForSelector('.cell-output:visible', { timeout: 5000 });
    await page.waitForTimeout(500);

    // Add cell 2
    await page.click('#btn-add-cell');

    // Wait for second cell to be created and ready
    await page.waitForTimeout(500);

    // Execute cell 2 using cell 1's variable
    const secondCell = page.locator('.cell').nth(1);
    await secondCell.locator('.CodeMirror').click();
    await page.keyboard.type('x * 2');
    await page.keyboard.press('Shift+Enter');

    // Wait for second cell output to appear
    await page.waitForTimeout(1000);

    // Verify shared state works
    const output = await secondCell.locator('.cell-output').textContent();
    expect(output).toContain('20');
  });
});
