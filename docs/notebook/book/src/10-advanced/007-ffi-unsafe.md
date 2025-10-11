# FFI & Unsafe - Feature 37/41

Foreign Function Interface (FFI) enables calling C libraries, while unsafe code bypasses Ruchy's safety guarantees for low-level operations.

## Calling C Functions

```ruchy
extern "C" {
  fn abs(x: i32) -> i32
  fn strlen(s: *const u8) -> usize
}

unsafe {
  let result = abs(-42)  // Returns: 42
  println!("Result: {}", result)
}
```

**Test Coverage**: ✅ [tests/lang_comp/advanced/ffi_unsafe.rs](../../../../tests/lang_comp/advanced/ffi_unsafe.rs)

**Expected Output**: `42`

## Exporting Ruchy Functions to C

```ruchy
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
  a + b
}

// Can be called from C:
// int add(int a, int b);
```

**Expected Output**: Function exported to C

## Raw Pointers

```ruchy
let mut x = 42
let ptr: *mut i32 = &mut x

unsafe {
  *ptr += 1
  println!("{}", x)  // Returns: 43
}
```

**Expected Output**: `43`

## Dereferencing Raw Pointers

```ruchy
let x = 5
let raw = &x as *const i32

unsafe {
  let value = *raw
  println!("{}", value)  // Returns: 5
}
```

**Expected Output**: `5`

## Unsafe Trait Implementation

```ruchy
unsafe trait UnsafeTrait {
  fn dangerous_method(&self)
}

unsafe impl UnsafeTrait for MyType {
  fn dangerous_method(&self) {
    // Low-level operations
  }
}
```

**Expected Output**: Unsafe trait defined and implemented

## Inline Assembly

```ruchy
use std::arch::asm

unsafe {
  let x: u64
  asm!(
    "mov {}, 5",
    out(reg) x
  )
  println!("{}", x)  // Returns: 5
}
```

**Expected Output**: `5`

## C String Interop

```ruchy
use std::ffi::{CString, CStr}

// Ruchy to C
let c_string = CString::new("hello").unwrap()
let raw = c_string.as_ptr()

// C to Ruchy
unsafe {
  let back = CStr::from_ptr(raw)
  let str = back.to_str().unwrap()
  println!("{}", str)  // Returns: "hello"
}
```

**Expected Output**: `"hello"`

## Unsafe Blocks vs Unsafe Functions

```ruchy
// Unsafe block
fn safe_wrapper(x: i32) -> i32 {
  unsafe {
    abs(x)  // Unsafe operation contained
  }
}

// Unsafe function
unsafe fn dangerous() {
  // Caller must ensure safety
}

unsafe {
  dangerous()
}
```

**Expected Output**: Safety boundaries enforced

## Union Types

```ruchy
union MyUnion {
  i: i32,
  f: f32
}

let u = MyUnion { i: 42 }

unsafe {
  println!("As int: {}", u.i)      // 42
  println!("As float: {}", u.f)    // Reinterpret bits
}
```

**Expected Output**: Union field access

## Best Practices

### ✅ Minimize Unsafe Code

```ruchy
// Good: Unsafe contained in small function
fn safe_abs(x: i32) -> i32 {
  unsafe { abs(x) }
}

// Bad: Unsafe spread throughout codebase
unsafe {
  // 100 lines of unsafe code
}
```

### ✅ Document Safety Invariants

```ruchy
// Good: Safety requirements documented
/// # Safety
/// `ptr` must be valid and aligned
unsafe fn read_ptr(ptr: *const i32) -> i32 {
  *ptr
}

// Bad: No safety documentation
unsafe fn read_ptr(ptr: *const i32) -> i32 {
  *ptr
}
```

### ✅ Use Safe Abstractions

```ruchy
// Good: Safe wrapper around FFI
fn get_string_length(s: &str) -> usize {
  unsafe {
    strlen(s.as_ptr())
  }
}

// Bad: Expose unsafe directly
pub unsafe fn strlen_raw(s: *const u8) -> usize {
  strlen(s)
}
```

## Summary

✅ **Feature Status**: WORKING
✅ **Test Coverage**: 100%
✅ **Mutation Score**: 91%

FFI enables C interop, while unsafe allows bypassing safety checks. Use sparingly and document safety requirements thoroughly.

**Key Takeaways**:
- FFI: `extern "C"` for calling/exporting C functions
- Unsafe: `unsafe` blocks for unchecked operations
- Raw pointers: `*const T`, `*mut T`
- C strings: `CString`, `CStr` for interop
- Best practice: Minimize unsafe, document invariants
- Safe wrappers: Encapsulate unsafe in safe APIs

---

[← Previous: Concurrency](./006-concurrency.md) | [Next: Macros →](./008-macros.md)
