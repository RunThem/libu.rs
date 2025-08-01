/**
[`when`]

```rust
fn main() {
  libu::when! {
    true => {},
    false => {},
  }

  let a = libu::when! {
    1==2 => 0,
    2==2 => 1,
    @ => 2,
  };
}
```
*/
#[macro_export]
macro_rules! when {
  ($(let $pat:pat = )? $fcond:expr => $fbranch:expr
  $(, @ => $def_branch:expr)? $(,)?) => {
    if $(let $pat = )? $fcond { $fbranch }
    $(
      else { $def_branch }
    )?
  };

  ($(let $fpat:pat = )? $fcond:expr => $fbranch:expr,
  $($(let $rpat:pat = )? $rcond:expr => $rbranch:expr),+
  $(, @ => $def_branch:expr)? $(,)?) => {
    if $(let $fpat = )? $fcond { $fbranch }
    $(
      else if $(let $rpat = )? $rcond { $rbranch }
    )+
    $(
      else { $def_branch }
    )?
  };
}

/**
[`hash`]

```
use std::hash::DefaultHasher;

fn main() {
  // No Hasher
  let hash = libu::hash!("a");
  assert_eq!(8_186_225_505_942_432_243, hash);

  // With Hasher
  let hash = libu::hash!("b", DefaultHasher::new());
  assert_eq!(16_993_177_596_579_750_922, hash);
  }
```
*/
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

#[doc(hidden)]
macro_rules! count {
  (@subst $($x: tt)*) => (());
  ($($rest: expr),*) => (<[()]>::len(&[$($crate::count!(@subst $rest)),*]));
}

/**
[`hmap`]

```
fn main() {
  let map = libu::hmap!{
      "a" => 1,
      "b" => 2,
  };

  assert_eq!(map["a"], 1);
  assert_eq!(map["b"], 2);
  assert_eq!(map.get("c"), None);
}
```
*/
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

/**
[`hset`]

```rust
fn main() {
  let set = libu::hset!["a", "b"];

  assert!(set.contains("a"));
  assert!(set.contains("b"));
  assert!(!set.contains("c"));
}
```
*/
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

/**
[`tmap`]

```rust
fn main() {
  let map = libu::tmap! {
      "a" => 1,
      "b" => 2,
  };

  assert_eq!(map["a"], 1);
  assert_eq!(map["b"], 2);
  assert_eq!(map.get("c"), None);
}
```
*/
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

/**
[`tset`]

```rust
fn main() {
  let set = libu::tset!["a", "b"];

  assert!(set.contains("a"));
  assert!(set.contains("b"));
  assert!(!set.contains("c"));

  let mut iter = set.iter();
  assert_eq!(Some(&"a"), iter.next());
  assert_eq!(Some(&"b"), iter.next());
  assert_eq!(None, iter.next());
}
```
*/
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

/**
[`deque`]

```rust
fn main() {
  let deque = libu::deque![1, 2, 3, 4];
  let deque2: std::collections::VecDeque<_> = (1..=4).collect();

  assert_eq!(deque, deque2);
}
```
*/
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

/**
[`heap`]

```rust
fn main() {
  let mut heap = libu::heap![4, 1, 3, 2];

  assert_eq!(Some(4), heap.pop());
  assert_eq!(Some(3), heap.pop());
  assert_eq!(Some(2), heap.pop());
  assert_eq!(Some(1), heap.pop());
  assert_eq!(None, heap.pop());
}
```
*/
#[macro_export]
macro_rules! heap {
  () => { ::std::collections::BinaryHeap::new() };

  ($($elem:expr),+ $(,)?) => {{
    const CAP: usize = $crate::count!($($elem),*);
    let mut bheap = ::std::collections::BinaryHeap::with_capacity(CAP);
    $(
        bheap.push($elem);
    )+
    bheap
  }}
}
