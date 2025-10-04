/**
 * Jest test setup file
 * Configures the testing environment for Ruchy Notebook
 */

// Mock WebAssembly
global.WebAssembly = {
    instantiate: jest.fn().mockResolvedValue({
        instance: {
            exports: {
                memory: new WebAssembly.Memory({ initial: 256 }),
                __wbindgen_malloc: jest.fn(),
                __wbindgen_realloc: jest.fn(),
                __wbindgen_free: jest.fn(),
                init: jest.fn(),
                execute_code: jest.fn().mockReturnValue('42'),
            },
        },
    }),
    Memory: jest.fn().mockImplementation((descriptor) => ({
        buffer: new ArrayBuffer(descriptor.initial * 65536),
        grow: jest.fn(),
    })),
};

// Mock fetch for WASM loading
global.fetch = jest.fn().mockImplementation((url) => {
    if (url.includes('.wasm')) {
        return Promise.resolve({
            ok: true,
            arrayBuffer: () => Promise.resolve(new ArrayBuffer(8)),
        });
    }
    return Promise.resolve({
        ok: false,
        status: 404,
    });
});

// Mock localStorage
const localStorageMock = {
    getItem: jest.fn(),
    setItem: jest.fn(),
    removeItem: jest.fn(),
    clear: jest.fn(),
};
global.localStorage = localStorageMock;

// Mock Worker
class WorkerMock {
    constructor(scriptURL) {
        this.scriptURL = scriptURL;
        this.onmessage = null;
        this.onerror = null;
    }

    postMessage(message) {
        // Simulate async worker response
        setTimeout(() => {
            if (this.onmessage) {
                this.onmessage({
                    data: {
                        type: 'result',
                        result: '42',
                        cellId: message.cellId,
                    },
                });
            }
        }, 10);
    }

    terminate() {
        // Mock terminate
    }
}
global.Worker = WorkerMock;

// Mock IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
    constructor(callback, options) {
        this.callback = callback;
        this.options = options;
        this.elements = new Set();
    }

    observe(element) {
        this.elements.add(element);
        // Simulate all elements being visible
        setTimeout(() => {
            this.callback([
                {
                    target: element,
                    isIntersecting: true,
                    intersectionRatio: 1,
                },
            ]);
        }, 0);
    }

    unobserve(element) {
        this.elements.delete(element);
    }

    disconnect() {
        this.elements.clear();
    }
};

// Mock performance API
global.performance = {
    now: jest.fn(() => Date.now()),
    mark: jest.fn(),
    measure: jest.fn(),
    getEntriesByType: jest.fn(() => []),
    clearMarks: jest.fn(),
    clearMeasures: jest.fn(),
};

// Mock requestAnimationFrame
global.requestAnimationFrame = (cb) => setTimeout(cb, 16);
global.cancelAnimationFrame = clearTimeout;

// Mock console methods for cleaner test output
const originalConsole = { ...console };
global.console = {
    ...console,
    log: jest.fn(),
    warn: jest.fn(),
    error: jest.fn(),
    info: jest.fn(),
    debug: jest.fn(),
};

// Restore console for test debugging
afterEach(() => {
    if (process.env.DEBUG_TESTS) {
        console.log = originalConsole.log;
        console.error = originalConsole.error;
    }
});

// Clean up after each test
afterEach(() => {
    jest.clearAllMocks();
    localStorageMock.getItem.mockClear();
    localStorageMock.setItem.mockClear();
    document.body.innerHTML = '';
});