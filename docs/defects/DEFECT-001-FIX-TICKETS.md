# DEFECT-001 Fix Tickets

**Parent**: CRITICAL-DEFECT-001-UI-EXECUTION-BROKEN.md
**Status**: 🚨 ACTIVE - P0 CRITICAL
**Created**: 2025-10-12

---

## Ticket Structure

All tickets follow Extreme TDD:
1. **RED**: Write failing test first
2. **GREEN**: Fix to make test pass
3. **REFACTOR**: Clean up, ensure ≤10 complexity

---

## DEFECT-001-FIX: Fix UI Cell Execution

**Priority**: P0 - CRITICAL
**Estimate**: 4 hours
**Dependencies**: None

### Problem Statement
UI cell execution completely broken. E2E tests prove:
- Code cells return empty output (`""`)
- Shared state doesn't work (variables undefined across cells)
- Backend API works, frontend broken

### Acceptance Criteria
- ✅ E2E smoke test passes (00-smoke-test.spec.ts)
- ✅ Cell execution shows output
- ✅ Shared state works across cells
- ✅ All 21 E2E tests passing (currently 10/21 failing)

### Implementation Plan

#### RED Phase (Write Failing Tests)
```typescript
// Already exists: tests/e2e/notebook/00-smoke-test.spec.ts
test('should execute code cell with Shift+Enter', async ({ page }) => {
  const codeMirror = page.locator('.CodeMirror').first();
  await codeMirror.click();
  await page.keyboard.press('Control+A');
  await page.keyboard.type('42');
  await page.keyboard.press('Shift+Enter');

  await page.waitForSelector('.cell-output:visible', { timeout: 5000 });

  const output = await page.locator('.cell-output').first().textContent();
  expect(output).toContain('42');  // Currently fails - empty string
});
```

#### GREEN Phase (Fix Implementation)
1. **Check JavaScript execution flow**:
   ```bash
   grep -n "Shift.*Enter" static/notebook.html
   ```

2. **Verify CodeMirror key binding**:
   - Check if Shift+Enter event is bound
   - Verify it calls execution function
   - Check if output is displayed

3. **Fix execution logic**:
   ```javascript
   // In static/notebook.html
   codeMirror.on('keydown', function(cm, event) {
     if (event.shiftKey && event.key === 'Enter') {
       event.preventDefault();
       executeCell(cellId);  // Ensure this is called
     }
   });
   ```

4. **Verify API call**:
   ```javascript
   async function executeCell(cellId) {
     const code = getCodeFromCell(cellId);

     const response = await fetch('/api/execute', {
       method: 'POST',
       headers: { 'Content-Type': 'application/json' },
       body: JSON.stringify({ source: code })  // Correct field name
     });

     const result = await response.json();
     displayOutput(cellId, result);  // Ensure this updates DOM
   }
   ```

#### REFACTOR Phase
- Extract `executeCell()` to separate function
- Add error handling
- Ensure complexity ≤10
- Add JSDoc comments

### Testing Strategy
1. Run E2E smoke test after each change
2. Verify in all 3 browsers (Chrome/Firefox/Safari)
3. Test keyboard shortcuts work
4. Test button execution works
5. Test shared state across cells

---

## DEFECT-001-E2E: Fix All E2E Tests

**Priority**: P0 - CRITICAL
**Estimate**: 2 hours
**Dependencies**: DEFECT-001-FIX (execution must work first)

### Problem Statement
E2E tests use wrong selectors, test phantom UI that doesn't exist.

### Acceptance Criteria
- ✅ All E2E tests use correct selectors
- ✅ Selector validation helper added
- ✅ 21/21 tests passing (currently 10/21)

### Implementation Plan

#### Already Fixed
- ✅ Updated `00-smoke-test.spec.ts` to use `#notebook-cells` (not `#notebook-container`)

#### Remaining Work
1. **Update old E2E tests**:
   ```bash
   # Find all tests using wrong selectors
   grep -r "#notebook-container" tests/e2e/
   grep -r ".cell-input" tests/e2e/
   grep -r ".execute-button" tests/e2e/
   ```

2. **Add selector validator**:
   ```typescript
   // tests/e2e/helpers/selector-validator.ts
   export async function validateSelectors(page, required: string[]) {
     for (const selector of required) {
       const exists = await page.$(selector);
       if (!exists) {
         throw new Error(`PHANTOM UI: ${selector} doesn't exist`);
       }
     }
   }
   ```

3. **Use in all tests**:
   ```typescript
   test.beforeEach(async ({ page }) => {
     await page.goto('http://localhost:8080');
     await validateSelectors(page, [
       '#notebook-cells',
       '#cell-type-selector',
       '.CodeMirror'
     ]);
   });
   ```

---

## DEFECT-001-LINT: Add Frontend Linting

**Priority**: P1 - HIGH
**Estimate**: 3 hours
**Dependencies**: None

### Problem Statement
No linting for HTML/CSS/JavaScript. Quality issues go undetected.

### Acceptance Criteria
- ✅ ESLint configured for JavaScript
- ✅ Stylelint configured for CSS
- ✅ HTMLHint configured for HTML
- ✅ Pre-commit hooks run linters
- ✅ All linting warnings fixed

### Implementation Plan

#### Install Tools
```bash
make install-frontend-tools
```

#### Configure ESLint
```json
// .eslintrc.json
{
  "env": {
    "browser": true,
    "es2021": true
  },
  "extends": "eslint:recommended",
  "rules": {
    "no-unused-vars": "error",
    "no-undef": "error",
    "prefer-const": "error"
  }
}
```

#### Configure Stylelint
```json
// .stylelintrc.json
{
  "extends": "stylelint-config-standard",
  "rules": {
    "color-hex-length": "short",
    "declaration-block-no-duplicate-properties": true
  }
}
```

#### Configure HTMLHint
```json
// .htmlhintrc
{
  "tagname-lowercase": true,
  "attr-lowercase": true,
  "attr-value-double-quotes": true,
  "id-unique": true
}
```

#### Fix Linting Issues
```bash
make lint-frontend
# Fix all warnings/errors
```

---

## DEFECT-001-CI: Add E2E to CI/CD

**Priority**: P0 - CRITICAL
**Estimate**: 2 hours
**Dependencies**: DEFECT-001-FIX (tests must pass first)

### Problem Statement
E2E tests not in CI/CD pipeline. Broken UI can be deployed.

### Acceptance Criteria
- ✅ GitHub Actions runs E2E tests
- ✅ Coverage reports generated
- ✅ E2E failure blocks merge
- ✅ Playwright HTML report uploaded

### Implementation Plan

#### Update GitHub Actions
```yaml
# .github/workflows/ci.yml
- name: Install Playwright
  run: make e2e-install

- name: Run E2E Tests
  run: make test-e2e-smoke

- name: Upload E2E Report
  if: always()
  uses: actions/upload-artifact@v3
  with:
    name: playwright-report
    path: playwright-report/

- name: Check E2E Coverage
  run: |
    PASS_COUNT=$(grep -oP '\d+(?= passed)' e2e-results.txt)
    TOTAL=21
    COVERAGE=$(echo "scale=2; $PASS_COUNT / $TOTAL * 100" | bc)
    if (( $(echo "$COVERAGE < 80" | bc -l) )); then
      echo "E2E coverage $COVERAGE% below 80% threshold"
      exit 1
    fi
```

---

## DEFECT-001-COVERAGE: Frontend Coverage Reports

**Priority**: P1 - HIGH
**Estimate**: 3 hours
**Dependencies**: None

### Problem Statement
No coverage metrics for frontend JavaScript.

### Acceptance Criteria
- ✅ Istanbul/NYC configured
- ✅ Coverage reports generated
- ✅ Coverage ≥80% enforced
- ✅ Reports in CI/CD

### Implementation Plan

#### Install Tools
```bash
npm install --save-dev nyc @playwright/test-coverage
```

#### Configure NYC
```json
// .nycrc
{
  "all": true,
  "include": ["static/**/*.js"],
  "exclude": ["static/lib/**"],
  "reporter": ["html", "text", "lcov"],
  "check-coverage": true,
  "lines": 80,
  "functions": 80,
  "branches": 80
}
```

#### Run with Coverage
```bash
make coverage-frontend
```

---

## Timeline

| Ticket | Priority | Estimate | Start | End |
|--------|----------|----------|-------|-----|
| DEFECT-001-FIX | P0 | 4h | Now | +4h |
| DEFECT-001-E2E | P0 | 2h | +4h | +6h |
| DEFECT-001-CI | P0 | 2h | +6h | +8h |
| DEFECT-001-LINT | P1 | 3h | +8h | +11h |
| DEFECT-001-COVERAGE | P1 | 3h | +11h | +14h |

**Total**: 14 hours (P0: 8 hours, P1: 6 hours)

---

## Success Criteria (Definition of Done)

### P0 (Blocking)
- ✅ UI cell execution works (E2E test proves it)
- ✅ All E2E tests passing (21/21)
- ✅ E2E in CI/CD pipeline
- ✅ Pre-commit hooks enforce E2E

### P1 (High)
- ✅ Frontend linting configured
- ✅ All linting warnings fixed
- ✅ Frontend coverage ≥80%
- ✅ Coverage in CI/CD

### P2 (Nice to Have)
- ✅ Visual regression tests
- ✅ Performance testing (Lighthouse)
- ✅ Accessibility testing (axe-core)

---

**Created**: 2025-10-12
**Status**: 🚨 READY TO START
**Owner**: Development team

**This is our top priority. Nothing else matters until this is fixed.**
