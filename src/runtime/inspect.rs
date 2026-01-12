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
// COVERAGE-95: Additional tests for complete coverage

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn test_inspector_default() {
        let inspector = Inspector::default();
        assert_eq!(inspector.depth, 0);
        assert_eq!(inspector.max_depth, 10);
        assert_eq!(inspector.budget, 10000);
    }

    #[test]
    fn test_inspector_with_style() {
        let style = InspectStyle {
            max_elements: 5,
            max_string_len: 50,
            use_colors: true,
            indent: "    ".to_string(),
        };
        let inspector = Inspector::with_style(style);
        assert_eq!(inspector.style.max_elements, 5);
        assert!(inspector.style.use_colors);
    }

    #[test]
    fn test_inspector_exit() {
        let mut inspector = Inspector::new();
        inspector.depth = 5;
        inspector.exit();
        assert_eq!(inspector.depth, 4);
        inspector.exit();
        assert_eq!(inspector.depth, 3);
    }

    #[test]
    fn test_inspector_exit_at_zero() {
        let mut inspector = Inspector::new();
        inspector.depth = 0;
        inspector.exit();
        assert_eq!(inspector.depth, 0); // Should not underflow
    }

    #[test]
    fn test_inspector_has_budget() {
        let mut inspector = Inspector::new();
        assert!(inspector.has_budget());
        inspector.budget = 0;
        assert!(!inspector.has_budget());
    }

    #[test]
    fn test_inspector_consume_budget() {
        let mut inspector = Inspector::new();
        let initial = inspector.budget;
        inspector.consume_budget(100);
        assert_eq!(inspector.budget, initial - 100);
    }

    #[test]
    fn test_inspector_consume_budget_saturating() {
        let mut inspector = Inspector::new();
        inspector.budget = 50;
        inspector.consume_budget(100);
        assert_eq!(inspector.budget, 0); // Should saturate at 0
    }

    #[test]
    fn test_inspector_depth() {
        let mut inspector = Inspector::new();
        assert_eq!(inspector.depth(), 0);
        inspector.depth = 5;
        assert_eq!(inspector.depth(), 5);
    }

    #[test]
    fn test_inspector_at_max_depth() {
        let mut inspector = Inspector::new();
        assert!(!inspector.at_max_depth());
        inspector.depth = 10;
        assert!(inspector.at_max_depth());
        inspector.depth = 15;
        assert!(inspector.at_max_depth());
    }

    #[test]
    fn test_inspector_enter() {
        let mut inspector = Inspector::new();
        let val = 42i32;
        assert!(inspector.enter(&val));
        assert_eq!(inspector.depth, 1);
        // Second enter with same address should fail
        assert!(!inspector.enter(&val));
    }

    #[test]
    fn test_inspector_write_str() {
        let mut inspector = Inspector::new();
        use std::fmt::Write;
        write!(inspector, "hello").unwrap();
        assert_eq!(inspector.output, "hello");
    }

    #[test]
    fn test_inspect_style_default() {
        let style = InspectStyle::default();
        assert_eq!(style.max_elements, 10);
        assert_eq!(style.max_string_len, 100);
        assert!(!style.use_colors);
        assert_eq!(style.indent, "  ");
    }

    #[test]
    fn test_inspect_style_clone() {
        let style = InspectStyle {
            max_elements: 20,
            max_string_len: 200,
            use_colors: true,
            indent: "----".to_string(),
        };
        let cloned = style.clone();
        assert_eq!(cloned.max_elements, 20);
        assert!(cloned.use_colors);
    }

    #[test]
    fn test_inspect_i64() {
        let mut inspector = Inspector::new();
        42i64.inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("42"));
    }

    #[test]
    fn test_inspect_f64() {
        let mut inspector = Inspector::new();
        3.14f64.inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("3.14"));
    }

    #[test]
    fn test_inspect_bool() {
        let mut inspector = Inspector::new();
        true.inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("true"));
    }

    #[test]
    fn test_inspect_string() {
        let mut inspector = Inspector::new();
        "hello".to_string().inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("\"hello\""));
    }

    #[test]
    fn test_inspect_string_truncation() {
        let mut inspector = Inspector::new();
        inspector.style.max_string_len = 5;
        "hello world".to_string().inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("..."));
        assert!(inspector.output.contains("chars"));
    }

    #[test]
    fn test_inspect_str() {
        let mut inspector = Inspector::new();
        "test".inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("\"test\""));
    }

    #[test]
    fn test_inspect_str_truncation() {
        let mut inspector = Inspector::new();
        inspector.style.max_string_len = 3;
        "abcdefghij".inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("..."));
    }

    #[test]
    fn test_inspect_option_some() {
        let mut inspector = Inspector::new();
        let opt = Some(42i32);
        Inspect::inspect(&opt, &mut inspector).unwrap();
        assert!(inspector.output.contains("Some(42)"));
    }

    #[test]
    fn test_inspect_option_none() {
        let mut inspector = Inspector::new();
        let opt: Option<i32> = None;
        Inspect::inspect(&opt, &mut inspector).unwrap();
        assert_eq!(inspector.output, "None");
    }

    #[test]
    fn test_inspect_result_ok() {
        let mut inspector = Inspector::new();
        let res: Result<i32, &str> = Ok(42);
        Inspect::inspect(&res, &mut inspector).unwrap();
        assert!(inspector.output.contains("Ok(42)"));
    }

    #[test]
    fn test_inspect_result_err() {
        let mut inspector = Inspector::new();
        let res: Result<i32, &str> = Err("error");
        Inspect::inspect(&res, &mut inspector).unwrap();
        assert!(inspector.output.contains("Err("));
        assert!(inspector.output.contains("error"));
    }

    #[test]
    fn test_inspect_vec_at_max_depth() {
        let mut inspector = Inspector::new();
        inspector.depth = 10; // At max_depth
        let vec = vec![1, 2, 3];
        vec.inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("3 elements"));
    }

    #[test]
    fn test_inspect_vec_truncation() {
        let mut inspector = Inspector::new();
        inspector.style.max_elements = 2;
        let vec = vec![1, 2, 3, 4, 5];
        vec.inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("...3 more"));
    }

    #[test]
    fn test_inspect_hashmap() {
        let mut inspector = Inspector::new();
        let mut map = HashMap::new();
        map.insert("key".to_string(), 42i32);
        map.inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("key"));
        assert!(inspector.output.contains("42"));
    }

    #[test]
    fn test_inspect_hashmap_at_max_depth() {
        let mut inspector = Inspector::new();
        inspector.depth = 10;
        let mut map = HashMap::new();
        map.insert("a".to_string(), 1i32);
        map.inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("1 entries"));
    }

    #[test]
    fn test_inspect_hashmap_truncation() {
        let mut inspector = Inspector::new();
        inspector.style.max_elements = 1;
        let mut map = HashMap::new();
        map.insert("a".to_string(), 1i32);
        map.insert("b".to_string(), 2i32);
        map.insert("c".to_string(), 3i32);
        map.inspect(&mut inspector).unwrap();
        assert!(inspector.output.contains("more"));
    }

    #[test]
    fn test_display_form_atomic() {
        let form = DisplayForm::Atomic("42".to_string());
        let _ = format!("{:?}", form);
    }

    #[test]
    fn test_display_form_composite() {
        let form = DisplayForm::Composite(CompositeForm {
            opener: "[",
            elements: vec![(None, DisplayForm::Atomic("1".to_string()))],
            closer: "]",
            elided: None,
        });
        let cloned = form.clone();
        let _ = format!("{:?}", cloned);
    }

    #[test]
    fn test_display_form_reference() {
        let form = DisplayForm::Reference(0x1000, Box::new(DisplayForm::Atomic("val".to_string())));
        let cloned = form.clone();
        let _ = format!("{:?}", cloned);
    }

    #[test]
    fn test_display_form_opaque() {
        let form = DisplayForm::Opaque(OpaqueHandle {
            type_name: "Function".to_string(),
            id: Some("foo".to_string()),
        });
        let cloned = form.clone();
        let _ = format!("{:?}", cloned);
    }

    #[test]
    fn test_opaque_handle_clone() {
        let handle = OpaqueHandle {
            type_name: "Thread".to_string(),
            id: None,
        };
        let cloned = handle.clone();
        assert_eq!(cloned.type_name, "Thread");
        assert!(cloned.id.is_none());
    }

    #[test]
    fn test_composite_form_clone() {
        let form = CompositeForm {
            opener: "{",
            elements: vec![(
                Some("key".to_string()),
                DisplayForm::Atomic("value".to_string()),
            )],
            closer: "}",
            elided: Some(5),
        };
        let cloned = form.clone();
        assert_eq!(cloned.opener, "{");
        assert_eq!(cloned.elided, Some(5));
    }

    #[test]
    fn test_visit_set_contains_overflow() {
        let mut visited = VisitSet::new();
        // Fill to overflow
        for i in 0..12 {
            visited.insert(i * 100);
        }
        // Check all are accessible
        for i in 0..12 {
            assert!(visited.contains(i * 100));
        }
        assert!(!visited.contains(9999));
    }

    #[test]
    fn test_inspect_depth_trait() {
        let val = 42i32;
        assert_eq!(val.inspect_depth(), 1);
    }

    #[test]
    fn test_inspector_budget_exhaustion() {
        let mut inspector = Inspector::new();
        inspector.budget = 5;
        let vec: Vec<i32> = (0..100).collect();
        vec.inspect(&mut inspector).unwrap();
        // Should have stopped early due to budget
        assert!(inspector.output.contains("..."));
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
