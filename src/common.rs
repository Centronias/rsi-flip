#[extend::ext]
pub impl i64 {
    /// Casts to `u32`, panicking if negative.
    fn assert_positive(self) -> u32 {
        u32::try_from(self).expect("should always be positive")
    }

    /// Returns the Euclidean remainder as a `u32`. The Euclidean remainder is always non-negative
    /// by definition, so `assert_positive` cannot panic here.
    fn rem_euclid_asserting(self, other: i64) -> u32 {
        self.rem_euclid(other).assert_positive()
    }
}
