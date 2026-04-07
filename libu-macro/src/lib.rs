//! Declarative macros module
//!
//! Provides a collection of convenient declarative macros for common operations.
//!
//! # Available Macros
//!
//! ## Control Flow
//!
//! | Macro | Description |
//! |-------|-------------|
//! | [`chk_if!`] | Conditional return |
//! | [`brk_if!`] | Conditional break |
//! | [`cnt_if!`] | Conditional continue |
//!
//! ## Collection Initialization
//!
//! | Macro | Description |
//! |-------|-------------|
//! | [`hmap!`] | Initialize HashMap |
//! | [`hset!`] | Initialize HashSet |
//! | [`tmap!`] | Initialize BTreeMap |
//! | [`tset!`] | Initialize BTreeSet |
//! | [`deque!`] | Initialize VecDeque |
//! | [`heap!`] | Initialize BinaryHeap |
//!
//! ## Utility
//!
//! | Macro | Description |
//! |-------|-------------|
//! | [`hash!`] | Compute hash value |

/// Conditional return
///
/// Returns early from a function if the condition is true.
/// Optionally executes a statement before returning.
///
/// # Syntax
///
/// - `chk_if!(cond)` - Return if `cond` is true
/// - `chk_if!(cond, val)` - Return `val` if `cond` is true
/// - `chk_if!(cond, stmt)` - Execute `stmt` then return if `cond` is true
/// - `chk_if!(cond, val, stmt)` - Execute `stmt` then return `val` if `cond` is true
///
/// # Example
///
/// ```rust
/// fn process(data: Option<i32>) -> i32 {
///   libu::chk_if!(data.is_none(), -1);
///   data.unwrap()
/// }
///
/// fn cleanup(s: String) {
///   libu::chk_if!(s.is_empty(), drop(s));
///   // process non-empty string...
/// }
/// ```
#[macro_export]
macro_rules! chk_if {
  ($cond:expr) => {
    if $cond {
      return;
    }
  };
  ($cond:expr, $val:expr) => {
    if $cond {
      return $val;
    }
  };
  ($cond:expr, $stmt:stmt) => {
    if $cond {
      $stmt;
      return;
    }
  };
  ($cond:expr, $val:expr, $stmt:stmt) => {
    if $cond {
      $stmt;
      return $val;
    }
  };
}

/// Conditional break
///
/// Breaks from a loop if the condition is true.
/// Optionally executes a statement before breaking.
///
/// # Syntax
///
/// - `brk_if!(cond)` - Break if `cond` is true
/// - `brk_if!(cond, label)` - Break from `label` if `cond` is true
/// - `brk_if!(cond, stmt)` - Execute `stmt` then break if `cond` is true
/// - `brk_if!(cond, label, stmt)` - Execute `stmt` then break from `label` if `cond` is true
///
/// # Example
///
/// ```rust
/// let mut count = 0;
/// loop {
///   count += 1;
///   libu::brk_if!(count > 10);
/// }
/// assert_eq!(count, 11);
///
/// 'outer: loop {
///   loop {
///     libu::brk_if!(true, 'outer); // Break from outer loop
///   }
/// }
/// ```
#[macro_export]
macro_rules! brk_if {
  ($cond:expr) => {
    if $cond {
      break;
    }
  };
  ($cond:expr, $label:lifetime) => {
    if $cond {
      break $label;
    }
  };
  ($cond:expr, $stmt:stmt) => {
    if $cond {
      $stmt;
      break;
    }
  };
  ($cond:expr, $label:lifetime, $stmt:stmt) => {
    if $cond {
      $stmt;
      break $label;
    }
  };
}

/// Conditional continue
///
/// Continues to the next iteration of a loop if the condition is true.
/// Optionally executes a statement before continuing.
///
/// # Syntax
///
/// - `cnt_if!(cond)` - Continue if `cond` is true
/// - `cnt_if!(cond, label)` - Continue to `label` if `cond` is true
/// - `cnt_if!(cond, stmt)` - Execute `stmt` then continue if `cond` is true
/// - `cnt_if!(cond, label, stmt)` - Execute `stmt` then continue to `label` if `cond` is true
///
/// # Example
///
/// ```rust
/// let mut sum = 0;
/// for i in 1..=10 {
///   libu::cnt_if!(i % 2 == 0); // Skip even numbers
///   sum += i;
/// }
/// assert_eq!(sum, 25); // 1 + 3 + 5 + 7 + 9
/// ```
#[macro_export]
macro_rules! cnt_if {
  ($cond:expr) => {
    if $cond {
      continue;
    }
  };
  ($cond:expr, $label:lifetime) => {
    if $cond {
      continue $label;
    }
  };
  ($cond:expr, $stmt:stmt) => {
    if $cond {
      $stmt;
      continue;
    }
  };
  ($cond:expr, $label:lifetime, $stmt:stmt) => {
    if $cond {
      $stmt;
      continue $label;
    }
  };
}

/// Compute hash value
///
/// Computes the hash of a value using the default or a custom hasher.
///
/// # Syntax
///
/// - `hash!(value)` - Hash with DefaultHasher
/// - `hash!(value, hasher)` - Hash with custom hasher instance
///
/// # Example
///
/// ```rust
/// use std::hash::DefaultHasher;
///
/// let h1 = libu::hash!("hello");
/// let h2 = libu::hash!("hello", DefaultHasher::new());
///
/// // Hash values are consistent
/// assert_eq!(h1, h2);
/// ```
#[macro_export]
macro_rules! hash {
  ($e:expr) => {{
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    $e.hash(&mut hasher);
    hasher.finish()
  }};

  ($e:expr, $hasher:expr) => {{
    use std::hash::{Hash, Hasher};

    let mut hasher = $hasher;
    $e.hash(&mut hasher);
    hasher.finish()
  }};
}

/// Count number of arguments (internal use)
#[doc(hidden)]
#[macro_export]
macro_rules! count {
  (@subst $($x: tt)*) => (());
  ($($rest: expr),*) => (<[()]>::len(&[$($crate::count!(@subst $rest)),*]));
}

/// Initialize HashMap
///
/// Creates a HashMap with the given key-value pairs.
/// Pre-allocates capacity based on the number of elements.
///
/// # Syntax
///
/// - `hmap!()` - Empty HashMap
/// - `hmap!(k1 => v1, k2 => v2, ...)` - HashMap with entries
///
/// # Example
///
/// ```rust
/// let map = libu::hmap! {
///   "a" => 1,
///   "b" => 2,
/// };
///
/// assert_eq!(map["a"], 1);
/// assert_eq!(map.get("c"), None);
/// ```
#[macro_export]
macro_rules! hmap {
  () => { ::std::collections::HashMap::new() };

  ($($key:expr => $value:expr),+ $(,)?) => {{
    const CAP: usize = $crate::count!($($key),*);
    let mut map = ::std::collections::HashMap::with_capacity(CAP);
    $(
      let _ = map.insert($key, $value);
    )+
    map
  }};
}

/// Initialize HashSet
///
/// Creates a HashSet with the given elements.
/// Pre-allocates capacity based on the number of elements.
///
/// # Syntax
///
/// - `hset!()` - Empty HashSet
/// - `hset!(elem1, elem2, ...)` - HashSet with elements
///
/// # Example
///
/// ```rust
/// let set = libu::hset!["a", "b", "c"];
///
/// assert!(set.contains("a"));
/// assert!(!set.contains("d"));
/// ```
#[macro_export]
macro_rules! hset {
  () => { ::std::collections::HashSet::new() };

  ($($elem:expr),+ $(,)?) => {{
    const CAP: usize = $crate::count!($($elem),*);
    let mut set = ::std::collections::HashSet::with_capacity(CAP);
    $(
      let _ = set.insert($elem);
    )+
    set
  }};
}

/// Initialize BTreeMap
///
/// Creates a BTreeMap with the given key-value pairs.
/// Elements are maintained in sorted order by key.
///
/// # Syntax
///
/// - `tmap!()` - Empty BTreeMap
/// - `tmap!(k1 => v1, k2 => v2, ...)` - BTreeMap with entries
///
/// # Example
///
/// ```rust
/// let map = libu::tmap! {
///   "b" => 2,
///   "a" => 1,
/// };
///
/// // Keys are sorted
/// let keys: Vec<_> = map.keys().collect();
/// assert_eq!(keys, vec!["a", "b"]);
/// ```
#[macro_export]
macro_rules! tmap {
  () => { ::std::collections::BTreeMap::new() };

  ($($key:expr => $value:expr),+ $(,)?) => {{
    let mut map = ::std::collections::BTreeMap::new();
    $(
      let _ = map.insert($key, $value);
    )+
    map
  }};
}

/// Initialize BTreeSet
///
/// Creates a BTreeSet with the given elements.
/// Elements are maintained in sorted order.
///
/// # Syntax
///
/// - `tset!()` - Empty BTreeSet
/// - `tset!(elem1, elem2, ...)` - BTreeSet with elements
///
/// # Example
///
/// ```rust
/// let set = libu::tset![3, 1, 2];
///
/// // Elements are sorted
/// let elems: Vec<_> = set.iter().collect();
/// assert_eq!(elems, vec![&1, &2, &3]);
/// ```
#[macro_export]
macro_rules! tset {
  () => { ::std::collections::BTreeSet::new() };

  ($($elem:expr),+ $(,)?) => {{
    let mut set = ::std::collections::BTreeSet::new();
    $(
      set.insert($elem);
    )+
    set
  }};
}

/// Initialize VecDeque
///
/// Creates a VecDeque with the given elements or repeated values.
///
/// # Syntax
///
/// - `deque!()` - Empty VecDeque
/// - `deque!(elem1, elem2, ...)` - VecDeque with elements
/// - `deque!(elem; n)` - VecDeque with `n` copies of `elem`
///
/// # Example
///
/// ```rust
/// let deque = libu::deque![1, 2, 3];
/// assert_eq!(deque.len(), 3);
///
/// let deque = libu::deque![0; 5];
/// assert_eq!(deque, vec![0, 0, 0, 0, 0]);
/// ```
#[macro_export]
macro_rules! deque {
  () => { ::std::collections::VecDeque::new() };

  ($elem:expr; $n:expr) => {{
    let mut deque = ::std::collections::VecDeque::new();
    deque.resize_with($n, || $elem);
    deque
  }};

  ($($elem:expr),+ $(,)?) => {{
    const CAP: usize = $crate::count!($($elem),*);
    let mut deque = ::std::collections::VecDeque::with_capacity(CAP);
    $(
      deque.push_back($elem);
    )+
    deque
  }};
}

/// Initialize BinaryHeap
///
/// Creates a BinaryHeap (max-heap) with the given elements.
///
/// # Syntax
///
/// - `heap!()` - Empty BinaryHeap
/// - `heap!(elem1, elem2, ...)` - BinaryHeap with elements
///
/// # Example
///
/// ```rust
/// let mut heap = libu::heap![1, 3, 2];
///
/// // Elements are popped in descending order (max-heap)
/// assert_eq!(heap.pop(), Some(3));
/// assert_eq!(heap.pop(), Some(2));
/// assert_eq!(heap.pop(), Some(1));
/// ```
#[macro_export]
macro_rules! heap {
  () => { ::std::collections::BinaryHeap::new() };

  ($($elem:expr),+ $(,)?) => {{
    const CAP: usize = $crate::count!($($elem),*);
    let mut heap = ::std::collections::BinaryHeap::with_capacity(CAP);
    $(
      heap.push($elem);
    )+
    heap
  }}
}