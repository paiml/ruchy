# VS Code Ruchy Extension Installation Guide

## Quick Install (Local Development)

```bash
# From the repository root
cd ide/vscode-ruchy
npm install
npm run compile
code --install-extension ruchy-0.1.0.vsix
```

## Manual Installation

1. **Download the Extension Package**
   - File: `ruchy-0.1.0.vsix` 
   - Located in: `ide/vscode-ruchy/`

2. **Install via VS Code Command Palette**
   - Open VS Code
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS)
   - Type "Install from VSIX"
   - Select the `ruchy-0.1.0.vsix` file
   - Restart VS Code

3. **Alternative: Command Line Installation**
   ```bash
   code --install-extension /path/to/ruchy-0.1.0.vsix
   ```

## Verify Installation

1. Open any `.ruchy` file
2. Check for:
   - Syntax highlighting (keywords in color)
   - Quality score in status bar (bottom right)
   - Commands in palette: "Ruchy: Run Tests", "Ruchy: Lint File", etc.

## Features

### Syntax Highlighting
- Keywords: `fn`, `let`, `const`, `if`, `else`, `for`, `while`, `match`
- Types: `i32`, `f64`, `String`, `Vec`, `bool`
- String interpolation: `f"Hello {name}"`
- Comments: `//` and `/* */`

### Quality Tools Integration
- **Status Bar**: Live quality score (A+ to F grades)
- **Commands** (Ctrl+Shift+P):
  - `Ruchy: Run Tests` - Execute test suite
  - `Ruchy: Lint File` - Lint with auto-fix
  - `Ruchy: Show Quality Score` - Detailed analysis
  - `Ruchy: Verify Proofs` - Check proofs

### Auto Features
- Lint on save (configurable)
- Quality score updates on save
- Inline diagnostics from linter

## Configuration

Access settings: File → Preferences → Settings → Search "Ruchy"

- `ruchy.binaryPath`: Path to ruchy executable (default: "ruchy")
- `ruchy.enableLinting`: Enable auto-linting (default: true)
- `ruchy.showQualityScore`: Show quality in status bar (default: true)
- `ruchy.lintOnSave`: Lint when saving files (default: true)

## Requirements

- VS Code 1.74.0 or later
- Ruchy v1.20.0 or later installed and in PATH

## Troubleshooting

### Extension Not Activating
- Ensure file has `.ruchy` extension
- Check Output panel (View → Output → Ruchy)
- Verify ruchy binary is in PATH: `which ruchy`

### Quality Score Not Showing
- Save the file to trigger analysis
- Check ruchy is installed: `ruchy --version`
- Enable in settings: `ruchy.showQualityScore`

### Linting Not Working
- Verify setting: `ruchy.enableLinting` is true
- Check ruchy lint works: `ruchy lint yourfile.ruchy`
- Look for errors in Output panel

## Development

To modify and rebuild:
```bash
cd ide/vscode-ruchy
npm install
# Make changes to src/extension.ts
npm run compile
npx vsce package
```

## Uninstall

Via VS Code:
1. Open Extensions panel (Ctrl+Shift+X)
2. Search for "Ruchy"
3. Click Uninstall

Via Command Line:
```bash
code --uninstall-extension ruchy-lang.ruchy
```