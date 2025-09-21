import { defineConfig } from 'vite';

export default defineConfig({
  test: {
    coverage: {
      provider: 'istanbul',
      reporter: ['text', 'json', 'lcov'],
      branches: 90,
      functions: 95,
      lines: 85,
      statements: 85,
      include: ['src/**/*.js', 'pkg/**/*.js'],
      exclude: ['**/*.test.js', '**/node_modules/**']
    },
    environment: 'jsdom',
    testTimeout: 30000
  }
});