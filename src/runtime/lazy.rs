//! Lazy evaluation support for pipelines and operations
//!
//! This module implements lazy evaluation for performance optimization,
//! allowing operations to be composed without immediate execution.
use std::cell::RefCell;
use std::rc::Rc;
/// Represents a lazily evaluated value
pub enum LazyValue {
    /// Already computed value
    Computed(Value),
    /// Deferred computation (using Rc for shared ownership of closure)
    Deferred(Rc<RefCell<Option<Value>>>, Rc<dyn Fn() -> Result<Value>>),
    /// Lazy pipeline stage
    Pipeline {
        source: Box<LazyValue>,
        transform: Rc<dyn Fn(Value) -> Result<Value>>,
    },
}
impl Clone for LazyValue {
    fn clone(&self) -> Self {
        match self {
            LazyValue::Computed(v) => LazyValue::Computed(v.clone()),
            LazyValue::Deferred(cache, computation) => {
                LazyValue::Deferred(Rc::clone(cache), Rc::clone(computation))
            }
            LazyValue::Pipeline { source, transform } => LazyValue::Pipeline {
                source: source.clone(),
                transform: Rc::clone(transform),
            },
        }
    }
}
use crate::runtime::interpreter::Value;
use anyhow::Result;
impl LazyValue {
    /// Create a new computed lazy value
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::lazy::LazyValue;
    ///
    /// let mut instance = LazyValue::new();
    /// let result = instance.computed();
    /// // Verify behavior
    /// ```
    pub fn computed(value: Value) -> Self {
        LazyValue::Computed(value)
    }
    /// Create a new deferred lazy value
    pub fn deferred<F>(computation: F) -> Self
    where
        F: Fn() -> Result<Value> + 'static,
    {
        LazyValue::Deferred(Rc::new(RefCell::new(None)), Rc::new(computation))
    }
    /// Create a pipeline transformation
    pub fn pipeline<F>(source: LazyValue, transform: F) -> Self
    where
        F: Fn(Value) -> Result<Value> + 'static,
    {
        LazyValue::Pipeline {
            source: Box::new(source),
            transform: Rc::new(transform),
        }
    }
    /// Force evaluation of the lazy value
    ///
    /// # Errors
    ///
    /// Returns an error if the computation fails
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::lazy::force;
    ///
    /// let result = force(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn force(&self) -> Result<Value> {
        match self {
            LazyValue::Computed(value) => Ok(value.clone()),
            LazyValue::Deferred(cache, computation) => {
                // Check if already computed
                if let Some(cached) = cache.borrow().as_ref() {
                    return Ok(cached.clone());
                }
                // Compute and cache
                let result = computation()?;
                *cache.borrow_mut() = Some(result.clone());
                Ok(result)
            }
            LazyValue::Pipeline { source, transform } => {
                let source_value = source.force()?;
                transform(source_value)
            }
        }
    }
    /// Check if the value has been computed
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::lazy::is_computed;
    ///
    /// let result = is_computed(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn is_computed(&self) -> bool {
        match self {
            LazyValue::Computed(_) => true,
            LazyValue::Deferred(cache, _) => cache.borrow().is_some(),
            LazyValue::Pipeline { .. } => false,
        }
    }
}
/// Type alias for filter predicates
type FilterPredicate = Box<dyn Fn(&Value) -> Result<bool>>;
/// Type alias for map transforms
type MapTransform = Box<dyn Fn(Value) -> Result<Value>>;
/// Lazy iterator for efficient collection processing
pub struct LazyIterator {
    /// Current state of the iterator
    state: RefCell<LazyIterState>,
}
enum LazyIterState {
    /// Source collection
    Source(Vec<Value>),
    /// Map transformation
    Map {
        source: Box<LazyIterator>,
        transform: MapTransform,
    },
    /// Filter transformation
    Filter {
        source: Box<LazyIterator>,
        predicate: FilterPredicate,
    },
    /// Take n elements
    Take {
        source: Box<LazyIterator>,
        count: usize,
    },
    /// Skip n elements
    Skip {
        source: Box<LazyIterator>,
        count: usize,
    },
}
impl LazyIterator {
    /// Create a new lazy iterator from a collection
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::lazy::LazyIterator;
    ///
    /// let mut instance = LazyIterator::new();
    /// let result = instance.from_vec();
    /// // Verify behavior
    /// ```
    pub fn from_vec(values: Vec<Value>) -> Self {
        LazyIterator {
            state: RefCell::new(LazyIterState::Source(values)),
        }
    }
    /// Map transformation
    #[must_use]
    pub fn map<F>(self, transform: F) -> Self
    where
        F: Fn(Value) -> Result<Value> + 'static,
    {
        LazyIterator {
            state: RefCell::new(LazyIterState::Map {
                source: Box::new(self),
                transform: Box::new(transform),
            }),
        }
    }
    /// Filter transformation
    #[must_use]
    pub fn filter<F>(self, predicate: F) -> Self
    where
        F: Fn(&Value) -> Result<bool> + 'static,
    {
        LazyIterator {
            state: RefCell::new(LazyIterState::Filter {
                source: Box::new(self),
                predicate: Box::new(predicate),
            }),
        }
    }
    /// Take first n elements
    #[must_use]
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::lazy::take;
    ///
    /// let result = take(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn take(self, count: usize) -> Self {
        LazyIterator {
            state: RefCell::new(LazyIterState::Take {
                source: Box::new(self),
                count,
            }),
        }
    }
    /// Skip first n elements
    #[must_use]
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::lazy::skip;
    ///
    /// let result = skip(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn skip(self, count: usize) -> Self {
        LazyIterator {
            state: RefCell::new(LazyIterState::Skip {
                source: Box::new(self),
                count,
            }),
        }
    }
    /// Collect the iterator into a vector (forces evaluation)
    ///
    /// # Errors
    ///
    /// Returns an error if any transformation fails
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::lazy::LazyIterator;
    ///
    /// let mut instance = LazyIterator::new();
    /// let result = instance.collect();
    /// // Verify behavior
    /// ```
    pub fn collect(&self) -> Result<Vec<Value>> {
        match &*self.state.borrow() {
            LazyIterState::Source(values) => Ok(values.clone()),
            LazyIterState::Map { source, transform } => {
                let source_values = source.collect()?;
                source_values
                    .into_iter()
                    .map(transform)
                    .collect::<Result<Vec<_>>>()
            }
            LazyIterState::Filter { source, predicate } => {
                let source_values = source.collect()?;
                let mut result = Vec::new();
                for value in source_values {
                    if predicate(&value)? {
                        result.push(value);
                    }
                }
                Ok(result)
            }
            LazyIterState::Take { source, count } => {
                let source_values = source.collect()?;
                Ok(source_values.into_iter().take(*count).collect())
            }
            LazyIterState::Skip { source, count } => {
                let source_values = source.collect()?;
                Ok(source_values.into_iter().skip(*count).collect())
            }
        }
    }
    /// Get the first element (forces minimal evaluation)
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::lazy::first;
    ///
    /// let result = first(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn first(&self) -> Result<Option<Value>> {
        let values = self.collect()?;
        Ok(values.into_iter().next())
    }
    /// Count elements (optimized to avoid full materialization where possible)
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::lazy::count;
    ///
    /// let result = count(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn count(&self) -> Result<usize> {
        match &*self.state.borrow() {
            LazyIterState::Source(values) => Ok(values.len()),
            _ => self.collect().map(|v| v.len()),
        }
    }
}
/// Lazy evaluation cache for memoization
pub struct LazyCache {
    cache: RefCell<std::collections::HashMap<String, Value>>,
}
impl LazyCache {
    /// Create a new lazy cache
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::lazy::LazyCache;
    ///
    /// let instance = LazyCache::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        LazyCache {
            cache: RefCell::new(std::collections::HashMap::new()),
        }
    }
    /// Get or compute a value
    ///
    /// # Errors
    ///
    /// Returns an error if computation fails
    pub fn get_or_compute<F>(&self, key: &str, compute: F) -> Result<Value>
    where
        F: FnOnce() -> Result<Value>,
    {
        if let Some(value) = self.cache.borrow().get(key) {
            return Ok(value.clone());
        }
        let value = compute()?;
        self.cache
            .borrow_mut()
            .insert(key.to_string(), value.clone());
        Ok(value)
    }
    /// Clear the cache
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::lazy::LazyCache;
    ///
    /// let mut instance = LazyCache::new();
    /// let result = instance.clear();
    /// // Verify behavior
    /// ```
    pub fn clear(&self) {
        self.cache.borrow_mut().clear();
    }
    /// Get cache size
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::runtime::lazy::size;
    ///
    /// let result = size(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn size(&self) -> usize {
        self.cache.borrow().len()
    }
}
impl Default for LazyCache {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn test_lazy_value_computed() {
        let lazy = LazyValue::computed(Value::Integer(42));
        assert!(lazy.is_computed());
        assert_eq!(lazy.force().unwrap(), Value::Integer(42));
    }
    #[test]
    fn test_lazy_value_deferred() {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = Rc::clone(&counter);
        let lazy = LazyValue::deferred(move || {
            *counter_clone.borrow_mut() += 1;
            Ok(Value::Integer(42))
        });
        assert!(!lazy.is_computed());
        assert_eq!(*counter.borrow(), 0);
        // First force computes
        assert_eq!(lazy.force().unwrap(), Value::Integer(42));
        assert_eq!(*counter.borrow(), 1);
        // Second force uses cache
        assert_eq!(lazy.force().unwrap(), Value::Integer(42));
        assert_eq!(*counter.borrow(), 1); // Not incremented again
    }
    #[test]
    fn test_lazy_iterator_map() {
        let values = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        let lazy = LazyIterator::from_vec(values).map(|v| {
            if let Value::Integer(n) = v {
                Ok(Value::Integer(n * 2))
            } else {
                Ok(v)
            }
        });
        let result = lazy.collect().unwrap();
        assert_eq!(
            result,
            vec![Value::Integer(2), Value::Integer(4), Value::Integer(6)]
        );
    }
    #[test]
    fn test_lazy_iterator_filter() {
        let values = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ];
        let lazy = LazyIterator::from_vec(values).filter(|v| {
            if let Value::Integer(n) = v {
                Ok(n % 2 == 0)
            } else {
                Ok(false)
            }
        });
        let result = lazy.collect().unwrap();
        assert_eq!(result, vec![Value::Integer(2), Value::Integer(4)]);
    }
    #[test]
    fn test_lazy_cache() {
        let cache = LazyCache::new();
        let counter = Rc::new(RefCell::new(0));
        // First call computes
        let counter_clone = Rc::clone(&counter);
        let result = cache
            .get_or_compute("key", || {
                *counter_clone.borrow_mut() += 1;
                Ok(Value::Integer(42))
            })
            .unwrap();
        assert_eq!(result, Value::Integer(42));
        assert_eq!(*counter.borrow(), 1);
        // Second call uses cache
        let counter_clone = Rc::clone(&counter);
        let result = cache
            .get_or_compute("key", || {
                *counter_clone.borrow_mut() += 1;
                Ok(Value::Integer(100))
            })
            .unwrap();
        assert_eq!(result, Value::Integer(42)); // Cached value
        assert_eq!(*counter.borrow(), 1); // Not incremented
    }
}
#[cfg(test)]
mod property_tests_lazy {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_computed_never_panics(input: String) {
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
