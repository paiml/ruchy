// Ruchy WASM Loader Demo
//
// This demonstrates loading WASM compiled from main.ruchy
// Note: Actual WASM loading depends on your Ruchy WASM implementation

// Update status indicator
const updateStatus = (message, isSuccess = true) => {
    const resultElement = document.getElementById('result');
    if (resultElement) {
        resultElement.textContent = message;
        resultElement.style.color = isSuccess ? '#10b981' : '#ef4444';
    }
};

// Simulate WASM loading
const loadWasm = async () => {
    try {
        updateStatus('Checking for WASM module...');

        // Check if main.wasm exists
        const response = await fetch('main.wasm');

        if (response.ok) {
            updateStatus(`✅ WASM module found!\n\nFile size: ${response.headers.get('content-length') || 'Unknown'} bytes\nLast modified: ${new Date().toLocaleString()}\n\nEdit main.ruchy and save to see auto-compilation!`, true);
        } else {
            updateStatus(`ℹ️  WASM module not found yet.\n\nTo compile:\n1. Run: ruchy serve --watch --watch-wasm\n2. Edit main.ruchy and save\n3. Watch it compile automatically!`, true);
        }
    } catch (error) {
        updateStatus(`⚠️  Could not load WASM module.\n\nMake sure to run:\nruchy serve --watch --watch-wasm\n\nThen edit and save main.ruchy`, false);
        console.error('WASM loading error:', error);
    }
};

// Auto-reload detection
let lastCheck = Date.now();
const checkForUpdates = async () => {
    try {
        const response = await fetch('main.wasm', { method: 'HEAD' });
        const lastModified = response.headers.get('last-modified');

        if (lastModified) {
            const modifiedTime = new Date(lastModified).getTime();
            if (modifiedTime > lastCheck) {
                console.log('🔄 WASM updated! Reloading...');
                setTimeout(() => location.reload(), 500);
            }
        }
    } catch (e) {
        // Silently ignore errors (file might not exist yet)
    }
};

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    updateStatus('Initializing...');
    loadWasm();

    // Check for updates every 2 seconds
    setInterval(checkForUpdates, 2000);

    console.log(`
🚀 Ruchy Dev Server Demo

Features enabled:
✓ Hot reload on file changes
✓ WASM compilation from .ruchy files
✓ Graceful shutdown (Ctrl+C)
✓ Network access for mobile testing

Try this:
1. Edit main.ruchy in your editor
2. Save the file
3. Watch the terminal output
4. See this page reload automatically!
    `);
});
