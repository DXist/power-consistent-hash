use pcg32::Pcg32;
use std::arch::asm;
use thiserror::Error;
use tracing::trace;

mod pcg32;

pub struct PowerConsistentHasher {
    // number of buckets
    n: u32,
    // m - 1
    m_minus_one: u32,
    // m/2 - 1
    m_half_minus_one: u32,
}

#[derive(Debug, Error)]
pub enum PowerConsistentHasherError {
    #[error("at least 2 buckets are required for consistent hashing")]
    NotEnoughBuckets,
}

impl PowerConsistentHasher {
    pub fn try_new(n: u32) -> Result<Self, PowerConsistentHasherError> {
        if n < 2 {
            return Err(PowerConsistentHasherError::NotEnoughBuckets);
        }

        // closest larger power of 2
        let mut m = n;
        // https://graphics.stanford.edu/~seander/bithacks.html#RoundUpPowerOf2
        m -= 1;
        m |= m >> 1;
        m |= m >> 2;
        m |= m >> 4;
        m |= m >> 8;
        m |= m >> 16;
        let m_minus_one = m;
        m += 1;
        let m_half_minus_one = (m >> 1) - 1;

        trace!(
            n = n,
            upper_power_of_two = m,
            "PowerConsistentHasher is initialized"
        );

        Ok(Self {
            n,
            m_minus_one,
            m_half_minus_one,
        })
    }

    #[cfg(feature = "seahash")]
    pub fn hash_bytes(&self, buf: &[u8]) -> u32 {
        let key = seahash::hash(buf);
        self.hash_u64(key)
    }

    /// Hash u64 consistently.
    ///
    /// Used when keys are sufficiently distributed over u64 range
    pub fn hash_u64(&self, key: u64) -> u32 {
        let (r1, maybe_rng) = consistent_hash_power_of_two(key, self.m_minus_one);

        if r1 < self.n {
            trace!(r1 = r1, "Choice in [0; m) range");
            return r1;
        }
        let mut rng = maybe_rng.expect("when r1 is not 0 rng has to be initialized");
        rng.step();
        let r2 = g(self.n, self.m_half_minus_one, rng);
        trace!(
            r2 = r2,
            m_half_minus_one = self.m_half_minus_one,
            "Calculated r2"
        );

        if r2 > self.m_half_minus_one {
            trace!(r2 = r2, "Choice in [m/2; n) range");
            return r2;
        }
        let (r, _) = consistent_hash_power_of_two(key, self.m_half_minus_one);
        trace!(
            r = r,
            m_half_minus_one = self.m_half_minus_one,
            "Choice in [0, m/2) range"
        );
        r
    }
}

// f function, maps key to uniform integer range [0; m - 1]
//
// m has to be power of 2
// Key should countain reasonbly random bits. key width >= log2(m).
fn consistent_hash_power_of_two(key: u64, m_minus_one: u32) -> (u32, Option<Pcg32>) {
    trace!(
        key = key,
        m_minus_one = m_minus_one,
        "consistent_hash_power_of_two"
    );

    let log2bits = (key & m_minus_one as u64) as u32;
    trace!(log2bits = log2bits, "log2bits");
    if log2bits == 0 {
        return (0, None);
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    // SAFETY: log2bits is not zero, bsr instruction will output definite result
    let msb_set = unsafe { msb_bit_index(log2bits) };
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    const MAX_BIT_INDEX: u32 = std::mem::size_of::<u64>() as u32 * 2 - 1;
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    let msb_set = MAX_BIT_INDEX - log2bits.leading_zeros();

    let mut rng = Pcg32::new(key, msb_set as u64);
    // 2 ** msb_set, h >= 1
    let h = 1_u32 << msb_set;
    trace!(h = h, "Power of 2");
    // rand integer in [h; 2h - 1] range
    let r = h.wrapping_add(rng.next_u32() & h.wrapping_sub(1));
    (r, Some(rng))
}

unsafe fn msb_bit_index(n: u32) -> u32 {
    let bsr: u32;
    asm!("bsr {:e}, {:e}", lateout(reg) bsr, in(reg) n, options(pure, nomem, nostack));
    bsr
}

// returns integer in range [s, n) with a weighted probability
//
// U > (x+1)/(j+1), U is random in (0, 1)
//
// Transform to use integer PRNG:
//
// (j + 1) * U * u32::MAX > (x + 1) * u32::MAX
// (j + 1) * (1 + rand_32) > (x + 1) * u32::MAX
//
// min_j + 1 = (x + 1) * u32::MAX.div_euclid(1 + rand32) + 1
// min_j = ((x + 1) * u32::MAX.div_euclid(1 + rand32)
// min_j = scaled_x.div_euclid(1 + rand32)
//      where scaled_x = (x + 1) * u32::MAX
//
// r = min_j
//
// if min_j >= n
// then
// n * (1 + rand_32) <= scaled_x
//
// and x - result
//
// if doesn't hold then compute min_j, x = min_j and continue

fn g(n: u32, s: u32, mut rng: Pcg32) -> u32 {
    let mut x = s; // x < n
    let n = n as u64;

    loop {
        let scaled_x: u64 = (x as u64 + 1) * u32::MAX as u64;
        let rnd_plus_one = rng.next_u32() as u64 + 1;

        // if x >= n then scaled_x >= (n + 1) * u32::MAX
        // n * rnd_plus_one <= n * (1 + u32::MAX)
        //                  <= n * u32::MAX + n <= (n + 1) * u32::MAX <= scaled_x
        //                                          It always holds for 32 bit n
        if n * rnd_plus_one <= scaled_x {
            break;
        }
        // x < n

        // n * rnd_plus_one > scaled_x
        // n > scaled_x / rnd_plus_one
        // n > scaled_x.div_euclid(rnd_plus_one)
        // n > min_j = r = new_x
        // thus x is set to r
        rng.step();
        // new x is not less then the previous x and less then n
        x = scaled_x.div_euclid(rnd_plus_one) as u32;
        debug_assert!((x as u64) < n);
    }
    x
}

#[cfg(test)]
mod tests;
