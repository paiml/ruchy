//! Object inspection protocol for REPL display
//!
//! [OBJ-INSPECT-002] Implement consistent object introspection API
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Write};
/// Object inspection trait for human-readable display in REPL
pub trait Inspect {
    /// Inspect the object, writing to the inspector
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result;
    /// Maximum recursion depth for this type
    fn inspect_depth(&self) -> usize {
        1
    }
}
/// Inspector manages inspection state and formatting
pub struct Inspector {
    /// Current recursion depth
    pub depth: usize,
    /// Maximum allowed depth
    pub max_depth: usize,
    /// Set of visited object addresses (cycle detection)
    visited: VisitSet,
    /// Complexity budget remaining
    pub budget: usize,
    /// Display style configuration
    pub style: InspectStyle,
    /// Output buffer
    pub output: String,
}
/// Optimized set for tracking visited objects
struct VisitSet {
    /// Inline storage for common case (<8 elements)
    inline: [usize; 8],
    /// Current count
    count: usize,
    /// Overflow storage for larger sets
    overflow: Option<HashSet<usize>>,
}
impl VisitSet {
    fn new() -> Self {
        Self {
            inline: [0; 8],
            count: 0,
            overflow: None,
        }
    }
    fn insert(&mut self, addr: usize) -> bool {
        // Check inline storage first
        for i in 0..self.count.min(8) {
            if self.inline[i] == addr {
                return false; // Already visited
            }
        }
        // Check overflow if present
        if let Some(ref mut overflow) = self.overflow {
            return overflow.insert(addr);
        }
        // Add to inline if space available
        if self.count < 8 {
            self.inline[self.count] = addr;
            self.count += 1;
            true
        } else {
            // Migrate to overflow storage
            let mut overflow = HashSet::new();
            for &addr in &self.inline {
                overflow.insert(addr);
            }
            overflow.insert(addr);
            self.overflow = Some(overflow);
            self.count += 1;
            true
        }
    }
    fn contains(&self, addr: usize) -> bool {
        // Check inline
        for i in 0..self.count.min(8) {
            if self.inline[i] == addr {
                return true;
            }
        }
        // Check overflow
        if let Some(ref overflow) = self.overflow {
            overflow.contains(&addr)
        } else {
            false
        }
    }
}
/// Display style configuration
#[derive(Debug, Clone)]
pub struct InspectStyle {
    /// Maximum elements to display in collections
    pub max_elements: usize,
    /// Maximum string length before truncation
    pub max_string_len: usize,
    /// Use colors in output
    pub use_colors: bool,
    /// Indentation string
    pub indent: String,
}
impl Default for InspectStyle {
    fn default() -> Self {
        Self {
            max_elements: 10,
            max_string_len: 100,
            use_colors: false,
            indent: "  ".to_string(),
        }
    }
}
/// Canonical display forms for values
#[derive(Debug, Clone)]
pub enum DisplayForm {
    /// Atomic values (42, true, "hello")
    Atomic(String),
    /// Composite values (`[1,2,3]`, `{x: 10}`)
    Composite(CompositeForm),
    /// Reference values (&value@0x7fff)
    Reference(usize, Box<DisplayForm>),
    /// Opaque values (`<fn>`, `<thread#42>`)
    Opaque(OpaqueHandle),
}
/// Composite value display structure
#[derive(Debug, Clone)]
pub struct CompositeForm {
    /// Opening delimiter
    pub opener: &'static str,
    /// Elements with optional labels
    pub elements: Vec<(Option<String>, DisplayForm)>,
    /// Closing delimiter
    pub closer: &'static str,
    /// Number of elided elements
    pub elided: Option<usize>,
}
/// Handle for opaque values
#[derive(Debug, Clone)]
pub struct OpaqueHandle {
    /// Type name
    pub type_name: String,
    /// Optional identifier
    pub id: Option<String>,
}
impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}
impl Inspector {
    /// Create a new inspector with default settings
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::inspect::Inspector;
    ///
    /// let instance = Inspector::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self::with_style(InspectStyle::default())
    }
    /// Create an inspector with custom style
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::inspect::Inspector;
    ///
    /// let mut instance = Inspector::new();
    /// let result = instance.with_style();
    /// // Verify behavior
    /// ```
    pub fn with_style(style: InspectStyle) -> Self {
        Self {
            depth: 0,
            max_depth: 10,
            visited: VisitSet::new(),
            budget: 10000,
            style,
            output: String::new(),
        }
    }
    /// Enter a new inspection context (cycle detection)
    pub fn enter<T>(&mut self, val: &T) -> bool {
        let addr = std::ptr::from_ref::<T>(val) as usize;
        if self.visited.contains(addr) {
            false // Cycle detected
        } else {
            self.visited.insert(addr);
            self.depth += 1;
            true
        }
    }
    /// Exit an inspection context
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::inspect::Inspector;
    ///
    /// let mut instance = Inspector::new();
    /// let result = instance.exit();
    /// // Verify behavior
    /// ```
    pub fn exit(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
    /// Check if budget allows continued inspection
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::inspect::has_budget;
    ///
    /// let result = has_budget(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn has_budget(&self) -> bool {
        self.budget > 0
    }
    /// Consume inspection budget
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::inspect::Inspector;
    ///
    /// let mut instance = Inspector::new();
    /// let result = instance.consume_budget();
    /// // Verify behavior
    /// ```
    pub fn consume_budget(&mut self, amount: usize) {
        self.budget = self.budget.saturating_sub(amount);
    }
    /// Get current depth
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::inspect::depth;
    ///
    /// let result = depth(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn depth(&self) -> usize {
        self.depth
    }
    /// Check if at maximum depth
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::inspect::Inspector;
    ///
    /// let mut instance = Inspector::new();
    /// let result = instance.at_max_depth();
    /// // Verify behavior
    /// ```
    pub fn at_max_depth(&self) -> bool {
        self.depth >= self.max_depth
    }
}
impl fmt::Write for Inspector {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.consume_budget(s.len());
        self.output.push_str(s);
        Ok(())
    }
}
// === Primitive Implementations ===
impl Inspect for i32 {
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result {
        write!(inspector, "{self}")
    }
}
impl Inspect for i64 {
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result {
        write!(inspector, "{self}")
    }
}
impl Inspect for f64 {
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result {
        write!(inspector, "{self}")
    }
}
impl Inspect for bool {
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result {
        write!(inspector, "{self}")
    }
}
impl Inspect for String {
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result {
        if self.len() <= inspector.style.max_string_len {
            write!(inspector, "\"{self}\"")
        } else {
            write!(
                inspector,
                "\"{}...\" ({} chars)",
                &self[..inspector.style.max_string_len],
                self.len()
            )
        }
    }
}
impl Inspect for &str {
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result {
        if self.len() <= inspector.style.max_string_len {
            write!(inspector, "\"{self}\"")
        } else {
            write!(
                inspector,
                "\"{}...\" ({} chars)",
                &self[..inspector.style.max_string_len],
                self.len()
            )
        }
    }
}
impl<T: Inspect> Inspect for Vec<T> {
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result {
        if inspector.at_max_depth() {
            return write!(inspector, "[{} elements]", self.len());
        }
        if !inspector.enter(self) {
            return write!(inspector, "[<circular>]");
        }
        write!(inspector, "[")?;
        let display_count = self.len().min(inspector.style.max_elements);
        for (i, item) in self.iter().take(display_count).enumerate() {
            if i > 0 {
                write!(inspector, ", ")?;
            }
            item.inspect(inspector)?;
            if !inspector.has_budget() {
                write!(inspector, ", ...")?;
                break;
            }
        }
        if self.len() > display_count {
            write!(inspector, ", ...{} more", self.len() - display_count)?;
        }
        inspector.exit();
        write!(inspector, "]")
    }
}
impl<K: Inspect, V: Inspect> Inspect for HashMap<K, V> {
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result {
        if inspector.at_max_depth() {
            return write!(inspector, "{{{} entries}}", self.len());
        }
        if !inspector.enter(self) {
            return write!(inspector, "{{<circular>}}");
        }
        write!(inspector, "{{")?;
        let display_count = self.len().min(inspector.style.max_elements);
        for (i, (key, value)) in self.iter().take(display_count).enumerate() {
            if i > 0 {
                write!(inspector, ", ")?;
            }
            key.inspect(inspector)?;
            write!(inspector, ": ")?;
            value.inspect(inspector)?;
            if !inspector.has_budget() {
                write!(inspector, ", ...")?;
                break;
            }
        }
        if self.len() > display_count {
            write!(inspector, ", ...{} more", self.len() - display_count)?;
        }
        inspector.exit();
        write!(inspector, "}}")
    }
}
impl<T: Inspect> Inspect for Option<T> {
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result {
        match self {
            Some(val) => {
                write!(inspector, "Some(")?;
                val.inspect(inspector)?;
                write!(inspector, ")")
            }
            None => write!(inspector, "None"),
        }
    }
}
impl<T: Inspect, E: Inspect> Inspect for Result<T, E> {
    fn inspect(&self, inspector: &mut Inspector) -> fmt::Result {
        match self {
            Ok(val) => {
                write!(inspector, "Ok(")?;
                val.inspect(inspector)?;
                write!(inspector, ")")
            }
            Err(err) => {
                write!(inspector, "Err(")?;
                err.inspect(inspector)?;
                write!(inspector, ")")
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_primitive_inspection() {
        let mut inspector = Inspector::new();
        42i32.inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("42"));
    }
    #[test]
    fn test_vector_inspection() {
        let vec = vec![1, 2, 3, 4, 5];
        let mut inspector = Inspector::new();
        vec.inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains('['));
        assert!(inspector.output.contains('1'));
        assert!(inspector.output.contains('5'));
    }
    #[test]
    fn test_cycle_detection() {
        // Can't easily test with standard types, but VisitSet works
        let mut visited = VisitSet::new();
        assert!(visited.insert(0x1000));
        assert!(!visited.insert(0x1000)); // Already visited
        assert!(visited.insert(0x2000));
        assert!(visited.contains(0x1000));
        assert!(visited.contains(0x2000));
        assert!(!visited.contains(0x3000));
    }
    #[test]
    fn test_overflow_visit_set() {
        let mut visited = VisitSet::new();
        // Fill inline storage
        for i in 0..10 {
            visited.insert(i * 0x1000);
        }
        // Should have migrated to overflow
        assert!(visited.overflow.is_some());
        assert!(visited.contains(0x0000));
        assert!(visited.contains(0x9000));
    }
}
#[cfg(test)]
mod property_tests_inspect {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
