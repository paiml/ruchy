import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

let statusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
    console.log('Ruchy extension activated');

    // Create status bar item for quality score
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.command = 'ruchy.score';
    context.subscriptions.push(statusBarItem);

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('ruchy.runTest', runTest),
        vscode.commands.registerCommand('ruchy.lint', lintFile),
        vscode.commands.registerCommand('ruchy.score', showQualityScore),
        vscode.commands.registerCommand('ruchy.prove', verifyProofs)
    );

    // Register diagnostics collection for lint results
    const diagnosticCollection = vscode.languages.createDiagnosticCollection('ruchy');
    context.subscriptions.push(diagnosticCollection);

    // Auto-lint on save
    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument((document) => {
            if (document.languageId === 'ruchy') {
                const config = vscode.workspace.getConfiguration('ruchy');
                if (config.get('lintOnSave')) {
                    lintDocument(document, diagnosticCollection);
                }
                if (config.get('showQualityScore')) {
                    updateQualityScore(document);
                }
            }
        })
    );

    // Update quality score when active editor changes
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor(editor => {
            if (editor && editor.document.languageId === 'ruchy') {
                updateQualityScore(editor.document);
            }
        })
    );

    // Initial update
    if (vscode.window.activeTextEditor?.document.languageId === 'ruchy') {
        updateQualityScore(vscode.window.activeTextEditor.document);
    }
}

async function runTest() {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) {
        vscode.window.showErrorMessage('No workspace folder open');
        return;
    }

    const terminal = vscode.window.createTerminal('Ruchy Test');
    terminal.show();
    terminal.sendText(`ruchy test ${workspaceFolder.uri.fsPath}`);
}

async function lintFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'ruchy') {
        vscode.window.showErrorMessage('No Ruchy file active');
        return;
    }

    const terminal = vscode.window.createTerminal('Ruchy Lint');
    terminal.show();
    terminal.sendText(`ruchy lint "${editor.document.uri.fsPath}" --fix`);
}

async function lintDocument(document: vscode.TextDocument, diagnosticCollection: vscode.DiagnosticCollection) {
    try {
        const config = vscode.workspace.getConfiguration('ruchy');
        const ruchyPath = config.get<string>('binaryPath') || 'ruchy';
        
        const { stdout } = await execAsync(`${ruchyPath} lint "${document.uri.fsPath}" --format=json`);
        
        // Parse lint results (assuming JSON output)
        try {
            const results = JSON.parse(stdout);
            const diagnostics: vscode.Diagnostic[] = [];
            
            if (results.issues) {
                for (const issue of results.issues) {
                    const range = new vscode.Range(
                        new vscode.Position(issue.line - 1, issue.column - 1),
                        new vscode.Position(issue.line - 1, issue.column)
                    );
                    
                    const severity = issue.severity === 'error' 
                        ? vscode.DiagnosticSeverity.Error
                        : issue.severity === 'warning'
                        ? vscode.DiagnosticSeverity.Warning
                        : vscode.DiagnosticSeverity.Information;
                    
                    diagnostics.push(new vscode.Diagnostic(range, issue.message, severity));
                }
            }
            
            diagnosticCollection.set(document.uri, diagnostics);
        } catch (e) {
            // Lint passed with no issues
            diagnosticCollection.set(document.uri, []);
        }
    } catch (error) {
        console.error('Lint error:', error);
    }
}

async function showQualityScore() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'ruchy') {
        vscode.window.showErrorMessage('No Ruchy file active');
        return;
    }

    try {
        const config = vscode.workspace.getConfiguration('ruchy');
        const ruchyPath = config.get<string>('binaryPath') || 'ruchy';
        
        const { stdout } = await execAsync(`${ruchyPath} score "${editor.document.uri.fsPath}"`);
        
        // Parse score from output
        const scoreMatch = stdout.match(/Score:\s*([\d.]+)\/1\.0/);
        if (scoreMatch) {
            const score = parseFloat(scoreMatch[1]);
            const grade = getGrade(score);
            vscode.window.showInformationMessage(`Quality Score: ${score}/1.0 (${grade})`);
        }
    } catch (error) {
        vscode.window.showErrorMessage('Failed to get quality score');
    }
}

async function updateQualityScore(document: vscode.TextDocument) {
    try {
        const config = vscode.workspace.getConfiguration('ruchy');
        if (!config.get('showQualityScore')) {
            statusBarItem.hide();
            return;
        }

        const ruchyPath = config.get<string>('binaryPath') || 'ruchy';
        const { stdout } = await execAsync(`${ruchyPath} score "${document.uri.fsPath}"`);
        
        const scoreMatch = stdout.match(/Score:\s*([\d.]+)\/1\.0/);
        if (scoreMatch) {
            const score = parseFloat(scoreMatch[1]);
            const grade = getGrade(score);
            statusBarItem.text = `$(dashboard) ${grade} (${score})`;
            statusBarItem.tooltip = `Ruchy Quality Score: ${score}/1.0`;
            statusBarItem.show();
        }
    } catch (error) {
        statusBarItem.hide();
    }
}

async function verifyProofs() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'ruchy') {
        vscode.window.showErrorMessage('No Ruchy file active');
        return;
    }

    const terminal = vscode.window.createTerminal('Ruchy Prove');
    terminal.show();
    terminal.sendText(`ruchy prove "${editor.document.uri.fsPath}" --check`);
}

function getGrade(score: number): string {
    if (score >= 0.97) return 'A+';
    if (score >= 0.93) return 'A';
    if (score >= 0.90) return 'A-';
    if (score >= 0.87) return 'B+';
    if (score >= 0.83) return 'B';
    if (score >= 0.80) return 'B-';
    if (score >= 0.77) return 'C+';
    if (score >= 0.73) return 'C';
    if (score >= 0.70) return 'C-';
    if (score >= 0.67) return 'D+';
    if (score >= 0.63) return 'D';
    if (score >= 0.60) return 'D-';
    return 'F';
}

export function deactivate() {
    // Clean up
}