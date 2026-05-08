use noiz::rng::AnyValueFromBits;

// kind of a shitty name
#[derive(Clone, Copy, Default)]
pub struct FullRange;

// The docs for this trait don't make much sense to me so idk if this is correctly implemented
impl AnyValueFromBits<u32> for FullRange {
    #[inline]
    fn linear_equivalent_value(&self, bits: u32) -> u32 {
        bits
    }

    #[inline]
    fn finish_linear_equivalent_value(&self, value: u32) -> u32 {
        value
    }

    #[inline]
    fn finishing_derivative(&self) -> f32 {
        1.0
    }
}
