//! 2-space pretty-printing for any `Debug` value.

use std::fmt::Debug;

use extend::ext;

/// Pretty-print any `Debug` value with 2-space indentation
///
/// `{:#?}` pretty output is hardcoded to 4 spaces per level (RFC 0640
/// deferred making it configurable). `Debug for str` escapes newlines, so
/// every newline in std/derived debug output comes from the formatter
/// itself and leading whitespace is always exactly `4 * depth` spaces —
/// re-indenting the final string to 2 spaces per level is lossless and
/// applies inside nested std containers (`Vec`, `HashMap`, ...) that a
/// custom `Debug` derive cannot reach.
///
/// # Example
///
/// ```rust
/// use libu_trait::Pretty;
///
/// #[derive(Debug)]
/// struct St {
///   name: String,
///   tags: Vec<String>,
/// }
///
/// let s = St {
///   name: "hello".into(),
///   tags: vec!["a".into(), "b".into()],
/// };
///
/// // 2-space indentation, nested containers included
/// assert_eq!(
///   s.pretty(),
///   "St {\n  name: \"hello\",\n  tags: [\n    \"a\",\n    \"b\",\n  ],\n}"
/// );
/// ```
#[ext(pub, name = Pretty)]
impl<T: Debug> T {
  /// Pretty-print with 2-space indentation.
  #[inline]
  fn pretty(&self) -> String {
    format!("{:#?}", self)
      .lines()
      .map(|line| {
        let lead = line.len() - line.trim_start_matches(' ').len();
        " ".repeat(lead / 4 * 2) + &line[lead..]
      })
      .collect::<Vec<_>>()
      .join("\n")
  }
}
