#[extend::ext]
pub impl i64 {
    fn assert_positive(self) -> u32 {
        u32::try_from(self).expect("should always be positive")
    }

    fn rem_euclid_asserting(self, other: i64) -> u32 {
        self.rem_euclid(other).assert_positive()
    }
}
