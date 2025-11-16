# Ruchy Notebook User Guide

## Complete Walkthrough for New Users

This guide shows you how to use Ruchy's Jupyter-style notebook with markdown and code cells.

---

## Step 1: Start the Notebook Server

```bash
# Navigate to the ruchy directory
cd /path/to/ruchy

# Start the notebook server
cargo run --bin ruchy notebook
```

You'll see:
```
🚀 Starting Ruchy Notebook server...
   Host: 127.0.0.1:8080
🚀 Notebook server running at http://127.0.0.1:8080
```

**Keep this terminal open** - the server needs to stay running.

---

## Step 2: Open the Notebook in Your Browser

Open your web browser and navigate to:
```
http://localhost:8080
```

**Important**: If you've used the notebook before, press `Ctrl+Shift+R` (hard refresh) to see the latest features.

---

## Step 3: Understanding the Interface

You'll see:

```
┌─────────────────────────────────────────────────────────┐
│ Ruchy Notebook                                     [+]  │
│ [Code ▼]  [Markdown]  [Raw]                            │
│  ^                                                       │
│  └─ Cell Type Selector                                  │
├─────────────────────────────────────────────────────────┤
│ 📝 Cell 1 (Welcome Cell - Code)                    [🗑]│
│ ┌─────────────────────────────────────────────────┐   │
│ │ // Welcome to Ruchy Notebook                     │   │
│ │ println("Hello, Ruchy!")                        │   │
│ │ 2 + 3                                           │   │
│ └─────────────────────────────────────────────────┘   │
│ Output: "Hello, Ruchy!"                                │
│         5                                              │
└─────────────────────────────────────────────────────────┘
```

**Key Elements**:
- **[+ Cell]** button: Adds a new cell
- **Cell Type Selector**: Choose Code, Markdown, or Raw
- **Cell toolbar**: Delete (🗑), Move Up (↑), Move Down (↓)
- **Cell content**: The actual code or markdown

---

## Step 4: Your First Code Cell

**Try this:**

1. **Click inside the welcome cell** (or create a new code cell)
2. **Type some Ruchy code**:
   ```ruchy
   x = 42
   y = 10
   x + y
   ```
3. **Press `Shift+Enter`** to execute
4. **See the output**: `52`

**Keyboard Shortcuts**:
- `Shift+Enter`: Execute cell and move to next
- `Ctrl+Enter`: Execute cell and stay in place
- `Esc`: Stop editing cell

---

## Step 5: Create Your First Markdown Cell

This is the **new NOTEBOOK-009 feature**!

### 5a. Create a Markdown Cell

1. **Select "Markdown"** from the cell type dropdown (top of page)
2. **Click the "+ Cell" button**
3. A new cell appears with text: "Double-click to edit"

### 5b. Edit the Markdown

1. **Double-click the markdown cell**
2. **Type your markdown**:
   ```markdown
   # My First Notebook

   This notebook demonstrates **Ruchy** features:
   - Variables
   - Functions
   - Control flow

   ## Code Example

   Here's some `inline code`.
   ```

3. **Press `Esc`** or **click outside** the cell

### 5c. See the Rendered Markdown

The markdown is now beautifully rendered as HTML:
- Headings are large and styled
- **Bold** and *italic* work
- Lists are formatted
- Code blocks are highlighted

**To edit again**: Double-click the rendered markdown

---

## Step 6: Build a Complete Notebook

Let's create a notebook that teaches Ruchy variables:

### Cell 1: Title (Markdown)
```markdown
# Learning Ruchy Variables

Variables in Ruchy are immutable by default.
```

### Cell 2: Example (Code)
```ruchy
x = 42
println("x = " + x.to_string())
```
**Output**: `x = 42`

### Cell 3: Explanation (Markdown)
```markdown
## Immutability

Once assigned, `x` cannot be changed. This prevents bugs!
```

### Cell 4: Another Example (Code)
```ruchy
name = "Alice"
greeting = "Hello, " + name
greeting
```
**Output**: `Hello, Alice`

---

## Step 7: Working with Multiple Cells

### Add Cells
- Click **"+ Cell"** to add at the end
- Or use **"Insert Below"** on any cell

### Reorder Cells
- Use **↑** (Move Up) button
- Use **↓** (Move Down) button

### Delete Cells
- Click **🗑** (Delete) button
- Confirm deletion

### Execute All Cells
1. Start from the top cell
2. Press `Shift+Enter` repeatedly
3. Or execute each manually

---

## Step 8: Save Your Notebook

### Method 1: Browser Local Storage (Automatic)
- Notebook state is saved automatically in your browser
- Reloading the page restores your work

### Method 2: Save to File (.rnb format)

**Via API** (advanced users):
```bash
curl -X POST http://localhost:8080/api/notebook/save \
  -H "Content-Type: application/json" \
  -d '{
    "path": "my-notebook.rnb",
    "notebook": {
      "cells": [...],
      "metadata": {...}
    }
  }'
```

**Via UI** (coming soon):
- Click "💾 Save" button
- Choose filename
- Saves as `.rnb` (Ruchy Notebook) file

---

## Step 9: Load a Saved Notebook

### Load Sample Notebooks

We've created 4 sample notebooks from the Ruchy documentation:

1. **01-literals.rnb** - Basic literal values (21 cells)
2. **01-variables.rnb** - Variable usage (37 cells)
3. **02-arithmetic.rnb** - Math operations (55 cells)
4. **03-if-else.rnb** - Control flow (55 cells)

**To load** (via API):
```bash
curl -X POST http://localhost:8080/api/notebook/load \
  -H "Content-Type: application/json" \
  -d '{"path": "notebooks/01-literals.rnb"}' | python3 -m json.tool
```

**Via UI** (coming soon):
- Click "📂 Open" button
- Select `.rnb` file
- Notebook loads with all cells

---

## Step 10: Advanced Features

### Markdown Features Supported

All standard markdown works:

**Headers**:
```markdown
# H1
## H2
### H3
```

**Emphasis**:
```markdown
**bold** *italic* ~~strikethrough~~
```

**Lists**:
```markdown
- Item 1
- Item 2
  - Nested item

1. First
2. Second
```

**Code**:
```markdown
Inline: `code here`

Block:
\`\`\`
code block
\`\`\`
```

**Links**:
```markdown
[Link text](https://example.com)
```

**Tables**:
```markdown
| Name | Age |
|------|-----|
| Alice| 30  |
| Bob  | 25  |
```

### XSS Protection

All markdown is sanitized before rendering - your notebook is safe from malicious content!

---

## Step 11: Cell Execution Order

**Important**: Cells execute independently but share state.

**Example**:

**Cell 1**:
```ruchy
x = 10
```

**Cell 2**:
```ruchy
y = 20
```

**Cell 3**:
```ruchy
x + y  // Uses x and y from above
```

**Output**: `30`

**If you execute Cell 3 first**, it will error because `x` and `y` don't exist yet!

**Best Practice**: Execute cells in order from top to bottom.

---

## Step 12: Common Workflows

### Workflow 1: Data Analysis Notebook

```markdown
# Data Analysis

## Load Data
```
```ruchy
data = [1, 2, 3, 4, 5]
```
```markdown
## Calculate Statistics
```
```ruchy
sum = data.reduce(0, |acc, x| acc + x)
mean = sum / data.len()
mean
```

### Workflow 2: Tutorial Notebook

```markdown
# Ruchy Tutorial

Learn Ruchy step by step!

## Lesson 1: Variables
```
```ruchy
x = 42
```
```markdown
Try changing the value of `x`!

## Lesson 2: Functions
```
```ruchy
fn double(x) {
  x * 2
}
double(21)
```

### Workflow 3: Documentation Notebook

```markdown
# API Documentation

## Function: `calculate_total`

**Purpose**: Calculates total with tax

**Example**:
```
```ruchy
fn calculate_total(subtotal, tax_rate) {
  subtotal * (1.0 + tax_rate)
}

calculate_total(100.0, 0.08)
```
```markdown
**Output**: 108.0
```

---

## Step 13: Keyboard Shortcuts Reference

| Shortcut | Action |
|----------|--------|
| `Shift+Enter` | Execute cell and move to next |
| `Ctrl+Enter` | Execute cell and stay |
| `Esc` | Stop editing / Save markdown cell |
| `Tab` | Indent (in code cells) |
| `Shift+Tab` | Unindent |

**Markdown Cell Specific**:
- Double-click: Start editing
- `Esc` or click outside: Render markdown

---

## Step 14: Troubleshooting

### Problem: "I don't see the markdown cell option"

**Solution**: Hard refresh the browser
- Chrome/Firefox: `Ctrl+Shift+R`
- Safari: `Cmd+Option+R`

### Problem: "Markdown shows as plain text"

**Solution**: Click outside the cell or press `Esc` to render

### Problem: "Cell execution fails"

**Check**:
1. Are previous cells executed? (shared state)
2. Is the syntax correct?
3. Check the error message in the output

### Problem: "Server not responding"

**Solution**:
1. Check the terminal - is the server still running?
2. Restart: `Ctrl+C` then `cargo run --bin ruchy notebook`

---

## Step 15: Example Complete Notebook

Here's a full example of a well-structured notebook:

### Cell 1 (Markdown):
```markdown
# Fibonacci Sequence Generator

This notebook demonstrates generating Fibonacci numbers in Ruchy.
```

### Cell 2 (Markdown):
```markdown
## Theory

The Fibonacci sequence: `0, 1, 1, 2, 3, 5, 8, 13, ...`

Each number is the sum of the two preceding ones.
```

### Cell 3 (Code):
```ruchy
fn fibonacci(n) {
  if n <= 1 {
    n
  } else {
    fibonacci(n - 1) + fibonacci(n - 2)
  }
}
```

### Cell 4 (Markdown):
```markdown
## Generate First 10 Numbers
```

### Cell 5 (Code):
```ruchy
for i in 0..10 {
  println(fibonacci(i))
}
```

### Cell 6 (Markdown):
```markdown
## Result

We've successfully generated the first 10 Fibonacci numbers!

**Complexity**: O(2^n) - exponential (not optimal!)

### Optimization

In a real application, use memoization or iteration.
```

---

## Next Steps

1. **Experiment**: Try creating cells with different content
2. **Learn Ruchy**: Use notebooks to learn the language interactively
3. **Share**: Export notebooks as `.rnb` files
4. **Validate**: Run `cargo test --test notebook_validation` to check examples

---

## Quick Reference Card

```
┌─────────────────────────────────────────────┐
│ RUCHY NOTEBOOK QUICK REFERENCE              │
├─────────────────────────────────────────────┤
│ Start Server:                               │
│   cargo run --bin ruchy notebook            │
│                                             │
│ Open Browser:                               │
│   http://localhost:8080                     │
│                                             │
│ Create Cell:                                │
│   Select type → Click [+ Cell]              │
│                                             │
│ Execute Code:                               │
│   Shift+Enter                               │
│                                             │
│ Edit Markdown:                              │
│   Double-click → Edit → Esc                 │
│                                             │
│ Cell Types:                                 │
│   • Code: Execute Ruchy code                │
│   • Markdown: Formatted documentation       │
│   • Raw: Plain text (no rendering)          │
│                                             │
│ Features:                                   │
│   ✅ Jupyter-style notebooks                │
│   ✅ Markdown + Code cells                  │
│   ✅ Live code execution                    │
│   ✅ File persistence (.rnb format)         │
│   ✅ XSS-safe rendering                     │
│   ✅ 90.2% validation success               │
└─────────────────────────────────────────────┘
```

---

**Need Help?**
- Check `NOTEBOOK-009-DEMO.md` for troubleshooting
- Visit `http://localhost:8080/test-markdown.html` for API tests
- Run validation tests: `cargo test --test notebook_validation -- --nocapture`

**Happy Notebook Computing! 📓✨**
