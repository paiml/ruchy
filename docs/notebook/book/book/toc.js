// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="00-introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">Part 1: Foundation (Features 1-9)</li><li class="chapter-item expanded "><a href="01-basic-syntax/index.html"><strong aria-hidden="true">1.</strong> Basic Syntax</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="01-basic-syntax/01-literals.html"><strong aria-hidden="true">1.1.</strong> Literals</a></li><li class="chapter-item expanded "><a href="01-basic-syntax/02-variables.html"><strong aria-hidden="true">1.2.</strong> Variables &amp; Assignment</a></li><li class="chapter-item expanded "><a href="01-basic-syntax/03-comments.html"><strong aria-hidden="true">1.3.</strong> Comments</a></li></ol></li><li class="chapter-item expanded "><a href="02-operators/index.html"><strong aria-hidden="true">2.</strong> Operators</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="02-operators/01-arithmetic.html"><strong aria-hidden="true">2.1.</strong> Arithmetic Operators</a></li><li class="chapter-item expanded "><a href="02-operators/02-comparison.html"><strong aria-hidden="true">2.2.</strong> Comparison Operators</a></li><li class="chapter-item expanded "><a href="02-operators/03-logical.html"><strong aria-hidden="true">2.3.</strong> Logical Operators</a></li><li class="chapter-item expanded "><a href="02-operators/04-bitwise.html"><strong aria-hidden="true">2.4.</strong> Bitwise Operators</a></li></ol></li><li class="chapter-item expanded "><a href="03-control-flow/index.html"><strong aria-hidden="true">3.</strong> Control Flow</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="03-control-flow/01-if-else.html"><strong aria-hidden="true">3.1.</strong> If-Else Expressions</a></li><li class="chapter-item expanded "><a href="03-control-flow/02-match.html"><strong aria-hidden="true">3.2.</strong> Match Expressions</a></li><li class="chapter-item expanded "><a href="03-control-flow/03-for-loops.html"><strong aria-hidden="true">3.3.</strong> For Loops</a></li><li class="chapter-item expanded "><a href="03-control-flow/04-while-loops.html"><strong aria-hidden="true">3.4.</strong> While Loops</a></li><li class="chapter-item expanded "><a href="03-control-flow/05-loop-control.html"><strong aria-hidden="true">3.5.</strong> Loop Control (break/continue)</a></li></ol></li><li class="chapter-item expanded "><li class="part-title">Part 2: Functions &amp; Data (Features 10-20)</li><li class="chapter-item expanded "><a href="04-functions/index.html"><strong aria-hidden="true">4.</strong> Functions</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="04-functions/01-definitions.html"><strong aria-hidden="true">4.1.</strong> Function Definitions</a></li><li class="chapter-item expanded "><a href="04-functions/02-parameters.html"><strong aria-hidden="true">4.2.</strong> Parameters &amp; Return Values</a></li><li class="chapter-item expanded "><a href="04-functions/03-closures.html"><strong aria-hidden="true">4.3.</strong> Closures &amp; Lambdas</a></li><li class="chapter-item expanded "><a href="04-functions/04-higher-order.html"><strong aria-hidden="true">4.4.</strong> Higher-Order Functions</a></li></ol></li><li class="chapter-item expanded "><a href="05-data-structures/index.html"><strong aria-hidden="true">5.</strong> Data Structures</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="05-data-structures/01-arrays.html"><strong aria-hidden="true">5.1.</strong> Arrays</a></li><li class="chapter-item expanded "><a href="05-data-structures/02-tuples.html"><strong aria-hidden="true">5.2.</strong> Tuples</a></li><li class="chapter-item expanded "><a href="05-data-structures/03-objects.html"><strong aria-hidden="true">5.3.</strong> Objects/Maps</a></li><li class="chapter-item expanded "><a href="05-data-structures/04-structs.html"><strong aria-hidden="true">5.4.</strong> Structs</a></li><li class="chapter-item expanded "><a href="05-data-structures/05-enums.html"><strong aria-hidden="true">5.5.</strong> Enums</a></li></ol></li><li class="chapter-item expanded "><li class="part-title">Part 3: Advanced Features (Features 21-30)</li><li class="chapter-item expanded "><a href="06-pattern-matching/index.html"><strong aria-hidden="true">6.</strong> Pattern Matching</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="06-pattern-matching/01-destructuring.html"><strong aria-hidden="true">6.1.</strong> Destructuring</a></li><li class="chapter-item expanded "><a href="06-pattern-matching/02-guards.html"><strong aria-hidden="true">6.2.</strong> Guards</a></li><li class="chapter-item expanded "><a href="06-pattern-matching/03-exhaustiveness.html"><strong aria-hidden="true">6.3.</strong> Exhaustiveness</a></li></ol></li><li class="chapter-item expanded "><a href="07-error-handling/index.html"><strong aria-hidden="true">7.</strong> Error Handling</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="07-error-handling/01-try-catch.html"><strong aria-hidden="true">7.1.</strong> Try-Catch</a></li><li class="chapter-item expanded "><a href="07-error-handling/02-option.html"><strong aria-hidden="true">7.2.</strong> Option Type</a></li><li class="chapter-item expanded "><a href="07-error-handling/03-result.html"><strong aria-hidden="true">7.3.</strong> Result Type</a></li></ol></li><li class="chapter-item expanded "><a href="08-strings/index.html"><strong aria-hidden="true">8.</strong> String Features</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="08-strings/01-interpolation.html"><strong aria-hidden="true">8.1.</strong> String Interpolation</a></li><li class="chapter-item expanded "><a href="08-strings/02-methods.html"><strong aria-hidden="true">8.2.</strong> String Methods</a></li><li class="chapter-item expanded "><a href="08-strings/03-escaping.html"><strong aria-hidden="true">8.3.</strong> String Escaping</a></li></ol></li><li class="chapter-item expanded "><li class="part-title">Part 4: Standard Library (Features 26-30)</li><li class="chapter-item expanded "><a href="09-stdlib/index.html"><strong aria-hidden="true">9.</strong> Standard Library</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="09-stdlib/01-collections.html"><strong aria-hidden="true">9.1.</strong> Collections</a></li><li class="chapter-item expanded "><a href="09-stdlib/02-iterators.html"><strong aria-hidden="true">9.2.</strong> Iterators</a></li><li class="chapter-item expanded "><a href="09-stdlib/03-io.html"><strong aria-hidden="true">9.3.</strong> I/O Operations</a></li><li class="chapter-item expanded "><a href="09-stdlib/04-math.html"><strong aria-hidden="true">9.4.</strong> Math Functions</a></li><li class="chapter-item expanded "><a href="09-stdlib/05-time.html"><strong aria-hidden="true">9.5.</strong> Time &amp; Date</a></li></ol></li><li class="chapter-item expanded "><li class="part-title">Part 5: Advanced Features (Features 31-42)</li><li class="chapter-item expanded "><a href="10-advanced/index.html"><strong aria-hidden="true">10.</strong> Advanced Features</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="10-advanced/01-generics.html"><strong aria-hidden="true">10.1.</strong> Generics</a></li><li class="chapter-item expanded "><a href="10-advanced/02-traits.html"><strong aria-hidden="true">10.2.</strong> Traits</a></li><li class="chapter-item expanded "><a href="10-advanced/03-lifetimes.html"><strong aria-hidden="true">10.3.</strong> Lifetimes</a></li><li class="chapter-item expanded "><a href="10-advanced/04-async-await.html"><strong aria-hidden="true">10.4.</strong> Async/Await</a></li><li class="chapter-item expanded "><a href="10-advanced/05-futures.html"><strong aria-hidden="true">10.5.</strong> Futures</a></li><li class="chapter-item expanded "><a href="10-advanced/006-concurrency.html"><strong aria-hidden="true">10.6.</strong> Concurrency</a></li><li class="chapter-item expanded "><a href="10-advanced/007-ffi-unsafe.html"><strong aria-hidden="true">10.7.</strong> FFI &amp; Unsafe</a></li><li class="chapter-item expanded "><a href="10-advanced/008-macros.html"><strong aria-hidden="true">10.8.</strong> Macros</a></li><li class="chapter-item expanded "><a href="10-advanced/009-metaprogramming.html"><strong aria-hidden="true">10.9.</strong> Metaprogramming</a></li><li class="chapter-item expanded "><a href="10-advanced/10-advanced-patterns.html"><strong aria-hidden="true">10.10.</strong> Advanced Patterns</a></li><li class="chapter-item expanded "><a href="10-advanced/11-optimization.html"><strong aria-hidden="true">10.11.</strong> Optimization</a></li><li class="chapter-item expanded "><a href="10-advanced/12-testing.html"><strong aria-hidden="true">10.12.</strong> Testing</a></li></ol></li><li class="chapter-item expanded "><li class="part-title">Part 6: Quality Proof</li><li class="chapter-item expanded "><a href="10-validation/index.html"><strong aria-hidden="true">11.</strong> Testing &amp; Validation</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="10-validation/01-coverage.html"><strong aria-hidden="true">11.1.</strong> Test Coverage Report</a></li><li class="chapter-item expanded "><a href="10-validation/02-mutation.html"><strong aria-hidden="true">11.2.</strong> Mutation Testing Report</a></li><li class="chapter-item expanded "><a href="10-validation/03-e2e.html"><strong aria-hidden="true">11.3.</strong> E2E Test Report</a></li><li class="chapter-item expanded "><a href="10-validation/04-wasm.html"><strong aria-hidden="true">11.4.</strong> WASM Validation</a></li></ol></li><li class="chapter-item expanded "><a href="11-conclusion.html">Conclusion</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
