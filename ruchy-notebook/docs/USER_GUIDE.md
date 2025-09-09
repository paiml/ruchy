# 📚 Ruchy Notebook User Guide

**Version 1.90.0** - Complete user documentation for the Ruchy interactive notebook platform

---

## 🚀 Quick Start

### Getting Started in 3 Steps

1. **Install Ruchy**: `cargo install ruchy`
2. **Start Notebook**: `ruchy notebook` or visit [web interface]
3. **Create Your First Cell**: Click "Add Cell" and start coding!

---

## 📖 Table of Contents

- [📱 Interface Overview](#-interface-overview)
- [⌨️ Keyboard Shortcuts](#️-keyboard-shortcuts)
- [📝 Working with Cells](#-working-with-cells)
- [🔄 Code Execution](#-code-execution)
- [💾 Saving & Loading](#-saving--loading)
- [📊 Data Visualization](#-data-visualization)
- [🛠️ Advanced Features](#️-advanced-features)
- [❓ Troubleshooting](#-troubleshooting)
- [📋 FAQ](#-faq)

---

## 📱 Interface Overview

### Main Components

```
┌─────────────────────────────────────────────┐
│ 🏠 Ruchy Notebook                  [+ Cell] │ ← Header & Toolbar
├─────────────────────────────────────────────┤
│ [ 1] ┌─────────────────────────────────────┐ │ 
│      │ println("Hello, Ruchy!");           │ │ ← Code Cell
│      │                                     │ │
│      └─────────────────────────────────────┘ │
│      ┌─────────────────────────────────────┐ │
│      │ Hello, Ruchy!                       │ │ ← Output Area
│      └─────────────────────────────────────┘ │
├─────────────────────────────────────────────┤
│ Ready                              🔋 Auto │ ← Status Bar
└─────────────────────────────────────────────┘
```

### Toolbar Buttons

| Button | Function | Description |
|--------|----------|-------------|
| **+ Cell** | Add new cell | Creates a new code cell below current selection |
| **Run All** | Execute all cells | Runs all cells in sequence from top to bottom |
| **Clear** | Clear outputs | Removes all cell outputs while preserving code |
| **Save** | Save notebook | Saves current notebook to local storage |
| **Export** | Export notebook | Downloads notebook as `.ipynb` file |

### Cell Components

Each cell contains:
- **Execution Counter**: `[1]` - Shows execution order
- **Code Input Area**: Where you write Ruchy code
- **Run Button**: ▶ - Executes the current cell
- **Delete Button**: × - Removes the cell
- **Output Area**: Shows execution results

---

## ⌨️ Keyboard Shortcuts

### Essential Shortcuts

| Shortcut | Action | Description |
|----------|--------|-------------|
| **Shift + Enter** | Run cell | Execute current cell and move to next |
| **Ctrl + Enter** | Run cell in place | Execute current cell, stay in same cell |
| **Ctrl + S** | Save notebook | Save current notebook state |
| **Ctrl + N** | New cell | Add new cell below current one |
| **Ctrl + Shift + N** | New cell above | Add new cell above current one |
| **Ctrl + Z** | Undo | Undo last change in current cell |
| **Ctrl + Y** | Redo | Redo last undone change |

### Advanced Shortcuts

| Shortcut | Action | Description |
|----------|--------|-------------|
| **Ctrl + A** | Select all | Select all text in current cell |
| **Ctrl + /** | Comment/Uncomment | Toggle comments on selected lines |
| **Tab** | Auto-complete | Show code completions (if available) |
| **Ctrl + ]** | Indent | Indent selected lines |
| **Ctrl + [** | Unindent | Unindent selected lines |

### Mobile Gestures

| Gesture | Action | Description |
|---------|--------|-------------|
| **Double tap** | Edit cell | Enter edit mode for cell |
| **Swipe left** | Delete cell | Remove cell (with confirmation) |
| **Long press** | Cell menu | Show cell options menu |
| **Pinch zoom** | Zoom interface | Adjust interface size |

---

## 📝 Working with Cells

### Cell Types

**Code Cells** (Primary)
- Execute Ruchy programming language code
- Display results in output area below
- Support syntax highlighting and auto-indentation

**Markdown Cells** (Future Feature)
- Rich text documentation
- Support for headers, lists, links, images
- Mathematical notation with LaTeX

### Creating Cells

1. **Click "+ Cell"** in toolbar
2. **Use Ctrl + N** keyboard shortcut
3. **Click below existing cell** to insert new cell
4. **Run last cell** automatically creates new cell

### Editing Cells

- **Click in cell** to start editing
- **Use arrow keys** to navigate within cell
- **Multi-line editing** supported with proper indentation
- **Syntax highlighting** for Ruchy code
- **Auto-completion** (where available)

### Managing Cells

**Moving Cells**
- Drag and drop to reorder (desktop)
- Cut and paste between positions
- Use cell toolbar buttons to move up/down

**Copying Cells**
- **Ctrl + C** to copy cell content
- **Ctrl + V** to paste in new cell
- Preserve formatting and execution state

**Deleting Cells**
- Click **×** button in cell toolbar
- **Delete** key when cell is selected
- Confirmation dialog prevents accidental deletion

---

## 🔄 Code Execution

### Basic Execution

**Single Cell Execution**
```ruchy
// Click Run button (▶) or press Shift + Enter
let message = "Hello, World!";
println(message);
```
Output:
```
Hello, World!
```

**Multiple Cell Execution**
- **Run All**: Executes all cells sequentially
- **Run Above**: Executes all cells above current cell
- **Run Below**: Executes all cells below current cell

### Execution Order

Cells execute in the order they appear, but you can run them individually:

```ruchy
// Cell 1
let x = 10;
```

```ruchy
// Cell 2 - depends on Cell 1
let y = x * 2;
println(y); // Will output: 20
```

### Error Handling

**Compilation Errors**
```ruchy
let x = 10
let y = x * // Syntax error - missing operand
```
Output:
```
CompileError: Expected expression after '*'
  at line 2, column 11
```

**Runtime Errors**
```ruchy
let numbers = [1, 2, 3];
println(numbers[10]); // Index out of bounds
```
Output:
```
RuntimeError: Index 10 out of bounds for array of length 3
  at line 2
```

### Performance Features

**WebWorker Execution**
- Code runs in background worker
- UI remains responsive during execution
- Automatic timeout protection (30 seconds)

**Memory Management**
- Automatic garbage collection
- Memory usage monitoring
- Warning for high memory consumption

---

## 💾 Saving & Loading

### Auto-Save

**Automatic Saving**
- Saves every 5 seconds by default
- Saves to browser local storage
- Status bar shows "Auto" when enabled

**Manual Saving**
- **Ctrl + S** or click "Save" button
- Immediate save to local storage
- Status bar shows "Saved" confirmation

### Export Options

**Jupyter Notebook Format (.ipynb)**
```json
{
  "cells": [
    {
      "cell_type": "code",
      "source": ["println(\"Hello, Ruchy!\");"],
      "outputs": [{"text": "Hello, Ruchy!"}],
      "execution_count": 1
    }
  ],
  "metadata": {
    "kernelspec": {
      "display_name": "Ruchy",
      "language": "ruchy",
      "name": "ruchy"
    }
  }
}
```

**Export Process**
1. Click **Export** button in toolbar
2. Choose format (currently .ipynb supported)
3. File downloads automatically
4. Compatible with Jupyter ecosystem

### Loading Notebooks

**From Local Storage**
- Automatically loads last saved state
- Preserves cell content and outputs
- Maintains execution counter state

**Import External Notebooks**
- Drag and drop .ipynb files (future feature)
- Import from URL (future feature)
- Convert from other formats (future feature)

---

## 📊 Data Visualization

### Basic Output Display

**Text Output**
```ruchy
println("Simple text output");
println("Multi-line\ntext output");
```

**Formatted Output**
```ruchy
let name = "Alice";
let age = 30;
println(f"Name: {name}, Age: {age}");
```

**Data Structure Display**
```ruchy
let data = [1, 2, 3, 4, 5];
println(data); // Displays: [1, 2, 3, 4, 5]

let person = { name: "Bob", age: 25 };
println(person); // Displays formatted object
```

### Advanced Visualization (Future Features)

**DataFrame Display**
```ruchy
// Future feature - enhanced table display
let df = DataFrame::from_csv("data.csv");
df.head(10); // Rich table visualization
```

**Charts and Plots**
```ruchy
// Future feature - integrated plotting
let data = [1, 4, 2, 8, 5, 7];
plot_line(data); // Interactive line chart
```

**Rich Media Output**
```ruchy
// Future feature - multimedia support
display_image("chart.png");
display_html("<h1>Custom HTML</h1>");
```

---

## 🛠️ Advanced Features

### WebWorker Integration

**Non-blocking Execution**
- Long-running code doesn't freeze UI
- Background processing with progress updates
- Automatic timeout and error recovery

**Memory Isolation**
- Each execution runs in isolated context
- Prevents memory leaks between cells
- Safe execution of untrusted code

### Progressive Web App (PWA)

**Offline Functionality**
- Works without internet connection
- Cached WASM runtime for fast loading
- Local storage for notebook persistence

**Installation**
- Install as desktop/mobile app
- Native-like experience
- Push notifications (future feature)

### Performance Optimizations

**Virtual Scrolling**
- Handles 1000+ cells efficiently
- Lazy loading of cell content
- Smooth scrolling performance

**WASM Runtime**
- Fast Ruchy code execution
- 119KB optimized runtime
- <50ms typical cell execution time

### Mobile Optimization

**Touch Interface**
- Optimized for touch interaction
- Gesture-based cell management
- Mobile keyboard compatibility

**Responsive Design**
- Adapts to screen size automatically
- Portrait/landscape support
- Optimized for tablets and phones

---

## ❓ Troubleshooting

### Common Issues

**Notebook Won't Load**
```
Problem: White screen or loading forever
Solution: 
1. Check browser compatibility (Chrome, Firefox, Safari, Edge)
2. Enable JavaScript in browser settings
3. Clear browser cache and reload
4. Check browser console for errors
```

**Code Won't Execute**
```
Problem: Cells show "Running..." but never complete
Solution:
1. Check for infinite loops in code
2. Refresh page to reset WebWorker
3. Check browser console for WASM errors
4. Verify browser supports WebWorkers
```

**High Memory Usage**
```
Problem: Browser becomes slow or crashes
Solution:
1. Clear all cell outputs: Click "Clear" button
2. Restart browser tab
3. Avoid creating very large data structures
4. Use smaller datasets for testing
```

**Save/Load Problems**
```
Problem: Notebook doesn't save or load properly
Solution:
1. Check browser local storage quota
2. Clear old saved notebooks
3. Export important notebooks as backup
4. Check browser privacy settings
```

### Browser Compatibility

**Minimum Requirements**
- **Chrome**: 67+ (recommended)
- **Firefox**: 68+ 
- **Safari**: 11.1+
- **Edge**: 79+
- **Mobile Safari**: 12+
- **Chrome Mobile**: 67+

**Required Features**
- WebAssembly support
- WebWorker support
- ES2018+ JavaScript
- Local Storage access
- Intersection Observer (for virtual scrolling)

### Performance Issues

**Slow Cell Execution**
```
Symptoms: Cells take >1 second to execute
Causes: Complex algorithms, large data processing
Solutions:
1. Break large computations into smaller cells
2. Use more efficient algorithms
3. Consider data streaming approaches
4. Monitor memory usage
```

**UI Responsiveness**
```
Symptoms: Interface feels sluggish
Causes: Too many cells, large outputs, low-end device
Solutions:
1. Limit cell count to <100 for optimal performance
2. Clear large outputs regularly
3. Use virtual scrolling features
4. Consider desktop browser for heavy work
```

---

## 📋 FAQ

### General Questions

**Q: What is Ruchy Notebook?**
A: Ruchy Notebook is an interactive computing environment for the Ruchy programming language. It provides a web-based interface similar to Jupyter Notebooks, allowing you to write, execute, and document Ruchy code in a cell-based format.

**Q: Do I need to install anything?**
A: For the web version, you only need a modern web browser. For local development, install Ruchy with `cargo install ruchy`. The notebook runs entirely in your browser using WebAssembly.

**Q: Is my code and data safe?**
A: Yes. All code execution happens locally in your browser. Nothing is sent to external servers. Your notebooks are saved in your browser's local storage.

**Q: Can I use Ruchy Notebook offline?**
A: Yes! Ruchy Notebook is a Progressive Web App (PWA) that works offline once initially loaded. You can install it as a desktop/mobile app for native-like experience.

### Technical Questions

**Q: How fast is code execution?**
A: Typical cell execution is under 50ms for simple operations. Complex computations may take longer but won't block the user interface thanks to WebWorker technology.

**Q: What's the memory usage like?**
A: The WASM runtime is only 119KB. Memory usage grows with your data and variables but includes automatic garbage collection. Typical sessions use <10MB additional memory.

**Q: Can I import external libraries?**
A: Currently, Ruchy Notebook includes the standard Ruchy library. External library support is planned for future versions.

**Q: How do I share notebooks?**
A: Export notebooks as `.ipynb` files (Jupyter format) for sharing. Direct sharing features are planned for future versions.

### Compatibility Questions

**Q: Which browsers are supported?**
A: Modern browsers with WebAssembly support: Chrome 67+, Firefox 68+, Safari 11.1+, Edge 79+. Mobile browsers are fully supported.

**Q: Can I run this on my tablet/phone?**
A: Yes! Ruchy Notebook is optimized for mobile devices with touch-friendly interface, gesture support, and responsive design.

**Q: Is it compatible with Jupyter?**
A: Export compatibility - you can export Ruchy notebooks as `.ipynb` files. Full import/interop is planned for future versions.

**Q: Can I use it with VS Code?**
A: VS Code integration is planned. Currently, you can export notebooks and edit the `.ipynb` files in VS Code with appropriate extensions.

### Feature Questions

**Q: Can I create charts and visualizations?**
A: Basic text output is currently supported. Rich visualizations, charts, and DataFrame displays are planned for upcoming versions.

**Q: How do I organize large projects?**
A: Use multiple notebooks for different aspects of your project. Export important notebooks as backups. Project-level organization features are planned.

**Q: Can multiple people collaborate?**
A: Real-time collaboration features are planned for future versions. Currently, share notebooks via export/import.

**Q: Is there version control?**
A: Basic auto-save and export functionality is available. Git integration and version history are planned features.

---

## 🚀 Getting Help

### Resources

- **GitHub Repository**: [https://github.com/paiml/ruchy](https://github.com/paiml/ruchy)
- **Documentation**: [Online docs and examples]
- **Community**: [Discord/Forum links when available]
- **Bug Reports**: [GitHub Issues](https://github.com/paiml/ruchy/issues)

### Support

- **Performance Issues**: Use the built-in performance testing suite
- **Bug Reports**: Include browser version, error messages, and steps to reproduce
- **Feature Requests**: Submit detailed use cases and requirements
- **Questions**: Check this guide first, then community forums

### Contributing

- **Code Contributions**: Fork repository, create feature branch, submit PR
- **Documentation**: Help improve guides, examples, and tutorials  
- **Testing**: Report bugs, test on different devices/browsers
- **Examples**: Create and share interesting notebook examples

---

*This guide covers Ruchy Notebook v1.90.0. For the latest updates and features, check the GitHub repository and release notes.*