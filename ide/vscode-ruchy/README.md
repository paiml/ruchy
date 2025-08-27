# Ruchy Language Support for VS Code

Official Visual Studio Code extension for the Ruchy programming language with integrated quality tools.

## Features

### 🎨 Syntax Highlighting
- Full syntax highlighting for `.ruchy` files
- Support for all Ruchy keywords and constructs
- String interpolation highlighting
- Comment highlighting

### 🔧 Quality Tools Integration
- **Test Runner**: Run `ruchy test` from the command palette
- **Linting**: Automatic linting with inline diagnostics
- **Quality Score**: Real-time quality score in status bar
- **Proof Verification**: Run `ruchy prove` on your code

### 📊 Status Bar Integration
- Live quality score display (A+ to F grades)
- Click to see detailed quality analysis
- Updates on file save

### ⚡ Commands
- `Ruchy: Run Tests` - Execute tests in workspace
- `Ruchy: Lint File` - Lint and auto-fix current file
- `Ruchy: Show Quality Score` - Display detailed quality metrics
- `Ruchy: Verify Proofs` - Check mathematical proofs

## Installation

### From Marketplace (Coming Soon)
1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Ruchy"
4. Click Install

### From Source
```bash
cd ide/vscode-ruchy
npm install
npm run compile
code --install-extension ruchy-0.1.0.vsix
```

## Requirements

- Ruchy v1.20.0 or later installed and in PATH
- VS Code 1.74.0 or later

## Extension Settings

This extension contributes the following settings:

* `ruchy.binaryPath`: Path to the ruchy binary (default: "ruchy")
* `ruchy.enableLinting`: Enable automatic linting (default: true)
* `ruchy.showQualityScore`: Show quality score in status bar (default: true)
* `ruchy.lintOnSave`: Run lint on file save (default: true)

## Usage

### Getting Started
1. Install the extension
2. Open a `.ruchy` file
3. Quality score appears in status bar
4. Save file to trigger linting

### Running Tests
1. Open Command Palette (Ctrl+Shift+P)
2. Run "Ruchy: Run Tests"
3. View results in terminal

### Checking Quality
- Look at status bar for live score
- Click score for detailed analysis
- Run "Ruchy: Show Quality Score" for full report

## Features in Development

- [ ] Language Server Protocol (LSP) support
- [ ] Code completion
- [ ] Hover documentation
- [ ] Go to definition
- [ ] Find references
- [ ] Rename symbol
- [ ] Code formatting
- [ ] Debugging support

## Known Issues

- JSON parsing for lint results may fail on complex errors
- Quality score updates may lag on large files

## Release Notes

### 0.1.0 (Initial Release)

- Basic syntax highlighting
- Quality tools integration
- Status bar quality score
- Command palette commands

## Contributing

Contributions welcome! Please check the [Ruchy repository](https://github.com/ruchy-lang/ruchy) for guidelines.

## License

MIT - See LICENSE file for details