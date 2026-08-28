//! Conditional element removal for `Vec`.

use extend::ext;

/// Remove elements by condition
///
/// Removes elements that satisfy the predicate, returning them as a new
/// `Vec` in their original relative order. A single O(n) pass built on
/// `Vec::extract_if`; `self` retains exactly the elements for which the
/// predicate returned `false`.
///
/// # Example
///
/// ```rust
/// use libu_trait::RemoveIf;
///
/// let mut vec = vec![1, 2, 3, 4, 5, 6];
/// let removed = vec.remove_if(|x| x % 2 == 0);
///
/// assert_eq!(vec, vec![1, 3, 5]);
/// assert_eq!(removed, vec![2, 4, 6]);
/// ```
#[ext(pub, name = RemoveIf)]
impl<T, F: Fn(&T) -> bool> Vec<T> {
  fn remove_if(&mut self, predicate: F) -> Self {
    self.extract_if(.., |x| predicate(x)).collect()
  }
}
