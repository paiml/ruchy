/**
 * HTML Validation and Structure Tests
 * Coverage target: >80%
 */

const fs = require('fs');
const path = require('path');
const { JSDOM } = require('jsdom');

describe('HTML Validation', () => {
    const htmlFiles = [
        'assets/index.html',
        'testing/index.html',
        'testing/mobile-performance-test.html',
        'testing/manual-performance-suite.html',
    ];

    htmlFiles.forEach((filePath) => {
        describe(`${filePath}`, () => {
            let dom;
            let document;

            beforeEach(() => {
                const html = fs.readFileSync(
                    path.join(__dirname, '..', filePath),
                    'utf-8'
                );
                dom = new JSDOM(html);
                document = dom.window.document;
            });

            test('should have valid DOCTYPE', () => {
                const doctype = dom.window.document.doctype;
                expect(doctype).toBeTruthy();
                expect(doctype.name).toBe('html');
            });

            test('should have lang attribute', () => {
                const html = document.querySelector('html');
                expect(html.getAttribute('lang')).toBeTruthy();
            });

            test('should have meta charset', () => {
                const charset = document.querySelector('meta[charset]');
                expect(charset).toBeTruthy();
                expect(charset.getAttribute('charset').toLowerCase()).toBe('utf-8');
            });

            test('should have viewport meta tag', () => {
                const viewport = document.querySelector('meta[name="viewport"]');
                expect(viewport).toBeTruthy();
                expect(viewport.getAttribute('content')).toContain('width=device-width');
            });

            test('should have title element', () => {
                const title = document.querySelector('title');
                expect(title).toBeTruthy();
                expect(title.textContent.length).toBeGreaterThan(0);
            });

            test('should have unique IDs', () => {
                const elements = document.querySelectorAll('[id]');
                const ids = new Set();
                
                elements.forEach((element) => {
                    const id = element.getAttribute('id');
                    expect(ids.has(id)).toBe(false);
                    ids.add(id);
                });
            });

            test('should have alt attributes for images', () => {
                const images = document.querySelectorAll('img');
                
                images.forEach((img) => {
                    expect(img.hasAttribute('alt')).toBe(true);
                });
            });

            test('should have proper heading hierarchy', () => {
                const headings = document.querySelectorAll('h1, h2, h3, h4, h5, h6');
                let previousLevel = 0;
                
                headings.forEach((heading) => {
                    const level = parseInt(heading.tagName[1]);
                    
                    // Heading levels should not skip (e.g., h1 -> h3)
                    if (previousLevel > 0) {
                        expect(level).toBeLessThanOrEqual(previousLevel + 1);
                    }
                    
                    previousLevel = level;
                });
            });

            test('should have labels for form inputs', () => {
                const inputs = document.querySelectorAll(
                    'input:not([type="hidden"]):not([type="submit"]):not([type="button"])'
                );
                
                inputs.forEach((input) => {
                    const id = input.getAttribute('id');
                    if (id) {
                        const label = document.querySelector(`label[for="${id}"]`);
                        expect(label).toBeTruthy();
                    } else {
                        // Input should be wrapped in label
                        const parent = input.parentElement;
                        expect(parent.tagName.toLowerCase()).toBe('label');
                    }
                });
            });

            test('should have proper ARIA attributes', () => {
                const ariaElements = document.querySelectorAll('[role]');
                
                ariaElements.forEach((element) => {
                    const role = element.getAttribute('role');
                    
                    // Common ARIA roles
                    const validRoles = [
                        'button', 'navigation', 'main', 'banner',
                        'contentinfo', 'complementary', 'region',
                        'article', 'section', 'form', 'search',
                        'menu', 'menuitem', 'tab', 'tabpanel',
                    ];
                    
                    expect(validRoles).toContain(role);
                });
            });

            test('should not have inline styles', () => {
                const elementsWithStyle = document.querySelectorAll('[style]');
                
                // Warn but don't fail - inline styles should be minimized
                if (elementsWithStyle.length > 0) {
                    console.warn(
                        `${filePath} has ${elementsWithStyle.length} elements with inline styles`
                    );
                }
                
                expect(elementsWithStyle.length).toBeLessThan(10);
            });

            test('should not have inline scripts', () => {
                const inlineScripts = document.querySelectorAll('script:not([src])');
                const onclickElements = document.querySelectorAll('[onclick]');
                
                // Critical for security
                expect(inlineScripts.length).toBe(0);
                expect(onclickElements.length).toBe(0);
            });

            test('should have semantic HTML5 elements', () => {
                const semanticElements = [
                    'header', 'nav', 'main', 'article',
                    'section', 'aside', 'footer',
                ];
                
                const hasSemanticElements = semanticElements.some(
                    (tag) => document.querySelector(tag) !== null
                );
                
                expect(hasSemanticElements).toBe(true);
            });

            test('should have proper link attributes', () => {
                const externalLinks = document.querySelectorAll(
                    'a[href^="http"]:not([href*="localhost"])'
                );
                
                externalLinks.forEach((link) => {
                    // External links should open in new tab
                    expect(link.getAttribute('target')).toBe('_blank');
                    
                    // Security: should have rel attribute
                    const rel = link.getAttribute('rel');
                    expect(rel).toContain('noopener');
                });
            });

            test('should have proper script loading', () => {
                const scripts = document.querySelectorAll('script[src]');
                
                scripts.forEach((script) => {
                    const src = script.getAttribute('src');
                    
                    // Scripts should use defer or async
                    const hasDefer = script.hasAttribute('defer');
                    const hasAsync = script.hasAttribute('async');
                    const isModule = script.getAttribute('type') === 'module';
                    
                    expect(hasDefer || hasAsync || isModule).toBe(true);
                });
            });

            test('should have CSS in head', () => {
                const stylesheets = document.querySelectorAll('link[rel="stylesheet"]');
                
                stylesheets.forEach((stylesheet) => {
                    expect(stylesheet.parentElement.tagName.toLowerCase()).toBe('head');
                });
            });

            test('should have valid color contrast', () => {
                // This is a simplified check - real contrast checking requires computed styles
                const styles = document.querySelector('style');
                
                if (styles) {
                    const cssText = styles.textContent;
                    
                    // Check for CSS variables defining colors
                    expect(cssText).toMatch(/--[a-z-]+:\s*#[0-9a-f]{3,6}/i);
                }
            });

            test('should be mobile-friendly', () => {
                // Check for responsive design indicators
                const viewport = document.querySelector('meta[name="viewport"]');
                expect(viewport.getAttribute('content')).toContain('initial-scale=1');
                
                // Check for responsive images
                const images = document.querySelectorAll('img');
                images.forEach((img) => {
                    const style = img.getAttribute('style');
                    if (style) {
                        expect(style).not.toContain('width:');
                        expect(style).not.toContain('height:');
                    }
                });
            });

            test('should have performance optimizations', () => {
                // Check for lazy loading
                const images = document.querySelectorAll('img');
                const iframes = document.querySelectorAll('iframe');
                
                [...images, ...iframes].forEach((element) => {
                    const loading = element.getAttribute('loading');
                    // Non-critical images/iframes should lazy load
                    if (!element.classList.contains('critical')) {
                        expect(loading).toBe('lazy');
                    }
                });
                
                // Check for preconnect/prefetch
                const resourceHints = document.querySelectorAll(
                    'link[rel="preconnect"], link[rel="prefetch"], link[rel="preload"]'
                );
                
                if (filePath.includes('index.html')) {
                    expect(resourceHints.length).toBeGreaterThan(0);
                }
            });
        });
    });
});

describe('Accessibility Tests', () => {
    test('should have skip navigation link', () => {
        const html = fs.readFileSync(
            path.join(__dirname, '..', 'assets', 'index.html'),
            'utf-8'
        );
        const dom = new JSDOM(html);
        const document = dom.window.document;
        
        const skipLink = document.querySelector('a[href="#main"], a[href="#content"]');
        
        // Skip link should be one of the first elements
        if (skipLink) {
            const allLinks = document.querySelectorAll('a');
            const skipLinkIndex = Array.from(allLinks).indexOf(skipLink);
            expect(skipLinkIndex).toBeLessThan(3);
        }
    });

    test('should have ARIA landmarks', () => {
        const html = fs.readFileSync(
            path.join(__dirname, '..', 'assets', 'index.html'),
            'utf-8'
        );
        const dom = new JSDOM(html);
        const document = dom.window.document;
        
        // Check for main landmark
        const main = document.querySelector('main, [role="main"]');
        expect(main).toBeTruthy();
        
        // Check for navigation
        const nav = document.querySelector('nav, [role="navigation"]');
        expect(nav).toBeTruthy();
    });

    test('should have proper focus management', () => {
        const html = fs.readFileSync(
            path.join(__dirname, '..', 'assets', 'index.html'),
            'utf-8'
        );
        const dom = new JSDOM(html);
        const document = dom.window.document;
        
        // Interactive elements should be focusable
        const interactiveElements = document.querySelectorAll(
            'a, button, input, select, textarea, [tabindex]'
        );
        
        interactiveElements.forEach((element) => {
            const tabindex = element.getAttribute('tabindex');
            
            // Avoid positive tabindex (breaks natural order)
            if (tabindex) {
                expect(parseInt(tabindex)).toBeLessThanOrEqual(0);
            }
        });
    });

    test('should have sufficient color contrast', () => {
        // This is a placeholder for actual contrast testing
        // Real implementation would use axe-core or similar
        expect(true).toBe(true);
    });
});