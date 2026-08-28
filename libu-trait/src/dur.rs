//! `Duration` parsing and numeric duration literals.

use std::time::Duration;

use extend::ext;

/// Parse string to `Duration`
///
/// # Accepted Grammar
///
/// A single number (integer or float, digits and `.`) immediately
/// followed by a unit, no spaces:
///
/// | Unit | Description |
/// |------|-------------|
/// | `ns` | Nanoseconds |
/// | `us` | Microseconds |
/// | `ms` | Milliseconds |
/// | `s` | Seconds |
/// | `m` | Minutes |
/// | `h` | Hours |
/// | `d` | Days |
///
/// Use [`ToDur::try_to_dur`] to handle invalid input; [`ToDur::to_dur`]
/// is the panicking convenience wrapper.
///
/// # Example
///
/// ```rust
/// use libu_trait::ToDur;
/// use std::time::Duration;
///
/// let dur = "100ms".to_dur();
/// assert_eq!(dur, Duration::from_millis(100));
///
/// let dur = "5s".to_dur();
/// assert_eq!(dur, Duration::from_secs(5));
///
/// let dur = "2m".to_dur();
/// assert_eq!(dur, Duration::from_secs(120));
/// ```
#[ext(pub, name = ToDur)]
impl str {
  /// Parses the string into a `Duration`, panicking on invalid input.
  ///
  /// # Panics
  ///
  /// Panics when the input has no unit, the number or unit cannot be
  /// parsed, or the resulting duration overflows. See
  /// [`ToDur::try_to_dur`] for the non-panicking variant.
  #[inline]
  fn to_dur(&self) -> Duration {
    self
      .try_to_dur()
      .expect("invalid duration literal; see ToDur::try_to_dur")
  }

  /// Parses the string into a `Duration`, reporting failures as `Err`.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_trait::ToDur;
  /// use std::time::Duration;
  ///
  /// // Floats and h/d units are supported.
  /// assert_eq!("1.5s".try_to_dur(), Ok(Duration::from_secs_f64(1.5)));
  /// assert_eq!("2h".try_to_dur(), Ok(Duration::from_secs(7200)));
  /// assert_eq!("1d".try_to_dur(), Ok(Duration::from_secs(86400)));
  ///
  /// // Failures come back as errors instead of panicking.
  /// assert!("100".try_to_dur().is_err()); // missing unit
  /// assert!("10x".try_to_dur().is_err()); // unsupported unit
  /// ```
  fn try_to_dur(&self) -> Result<Duration, String> {
    let Some(split) = self.find(|c: char| !c.is_ascii_digit() && c != '.') else {
      return Err(format!("missing time unit in {self:?}"));
    };

    let (num, unit) = self.split_at(split);
    let num: f64 = num
      .parse()
      .map_err(|_| format!("invalid number {num:?} in {self:?}"))?;

    let secs = match unit {
      "ns" => num * 1e-9,
      "us" => num * 1e-6,
      "ms" => num * 1e-3,
      "s" => num,
      "m" => num * 60.0,
      "h" => num * 3600.0,
      "d" => num * 86_400.0,

      _ => return Err(format!("unsupported time unit {unit:?} in {self:?}")),
    };

    Duration::try_from_secs_f64(secs).map_err(|_| format!("duration overflows in {self:?}"))
  }
}

/// Duration literals on numbers
///
/// The write-side counterpart of [`ToDur`]: builds a `Duration` straight
/// from a numeric literal. Multiplications saturate at `u64::MAX` instead
/// of overflowing.
///
/// # Example
///
/// ```rust
/// use libu_trait::DurExt;
/// use std::time::Duration;
///
/// assert_eq!(5.secs(), Duration::from_secs(5));
/// assert_eq!(250.ms(), Duration::from_millis(250));
/// assert_eq!(2.hours(), Duration::from_secs(7200));
/// assert_eq!(1.days(), Duration::from_secs(86400));
/// ```
#[ext(pub, name = DurExt)]
impl u64 {
  /// `self` nanoseconds.
  #[inline]
  fn ns(self) -> Duration {
    Duration::from_nanos(self.into())
  }

  /// `self` microseconds.
  #[inline]
  fn us(self) -> Duration {
    Duration::from_micros(self)
  }

  /// `self` milliseconds.
  #[inline]
  fn ms(self) -> Duration {
    Duration::from_millis(self)
  }

  /// `self` seconds.
  #[inline]
  fn secs(self) -> Duration {
    Duration::from_secs(self)
  }

  /// `self` minutes.
  #[inline]
  fn mins(self) -> Duration {
    Duration::from_secs(self.saturating_mul(60))
  }

  /// `self` hours.
  #[inline]
  fn hours(self) -> Duration {
    Duration::from_secs(self.saturating_mul(3600))
  }

  /// `self` days.
  #[inline]
  fn days(self) -> Duration {
    Duration::from_secs(self.saturating_mul(86_400))
  }
}
