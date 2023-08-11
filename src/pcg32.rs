/// Based on rand_pcg pcg64

// This is the default multiplier used by PCG for 64-bit state.
const MULTIPLIER: u64 = 6364136223846793005;

/// A PCG random number generator (XSH RR 64/32 (LCG) variant).
///
/// Permuted Congruential Generator with 64-bit state, internal Linear
/// Congruential Generator, and 32-bit output via "xorshift high (bits),
/// random rotation" output function.
///
/// This is a 64-bit LCG with explicitly chosen stream with the PCG-XSH-RR
/// output function. This combination is the standard `pcg32`.
#[derive(Clone, PartialEq, Eq)]
pub struct Lcg64Xsh32 {
    state: u64,
    increment: u64,
}

/// [`Lcg64Xsh32`] is also officially known as `pcg32`.
pub type Pcg32 = Lcg64Xsh32;

impl Lcg64Xsh32 {
    /// Construct an instance compatible with PCG seed and stream.
    ///
    /// Note that the highest bit of the `stream` parameter is discarded
    /// to simplify upholding internal invariants.
    ///
    /// Note that two generators with different stream parameters may be closely
    /// correlated.
    ///
    /// PCG specifies the following default values for both parameters:
    ///
    /// - `state = 0xcafef00dd15ea5e5`
    /// - `stream = 0xa02bdbf7bb3c0a7`
    // Note: stream is 1442695040888963407u64 >> 1
    #[inline]
    pub fn new(state: u64, stream: u64) -> Self {
        // The increment must be odd, hence we discard one bit:
        let increment = (stream << 1) | 1;
        Lcg64Xsh32::from_state_incr(state, increment)
    }

    #[inline]
    fn from_state_incr(state: u64, increment: u64) -> Self {
        let mut pcg = Lcg64Xsh32 { state, increment };
        // Move away from initial value:
        pcg.state = pcg.state.wrapping_add(pcg.increment);
        pcg.step();
        pcg
    }

    #[inline]
    pub fn step(&mut self) {
        // prepare the LCG for the next round
        self.state = self
            .state
            .wrapping_mul(MULTIPLIER)
            .wrapping_add(self.increment);
    }

    /// Apply output function to PRNG state without advancing the state
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        let state = self.state;

        // Output function XSH RR: xorshift high (bits), followed by a random rotate
        // Constants are for 64-bit state, 32-bit output
        const ROTATE: u32 = 59; // 64 - 5
        const XSHIFT: u32 = 18; // (5 + 32) / 2
        const SPARE: u32 = 27; // 64 - 32 - 5

        let rot = (state >> ROTATE) as u32;
        let xsh = (((state >> XSHIFT) ^ state) >> SPARE) as u32;
        xsh.rotate_right(rot)
    }
}
