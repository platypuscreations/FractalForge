// SPDX-License-Identifier: GPL-3.0-or-later

//! The C library's `rand()` as in glibc: the published additive feedback
//! generator, which Fractint-format IFS images use (docs/specs/ifs.md §4.1).

/// glibc-compatible `rand()`: outputs in 0..=2³¹ − 1.
#[derive(Clone, Debug)]
pub struct CRand {
    ring: [u32; 34],
    /// Index of the next value to generate.
    i: usize,
}

impl CRand {
    /// The generator after `srand(seed)`.
    pub fn new(seed: u32) -> Self {
        let mut ring = [0u32; 34];
        ring[0] = if seed == 0 { 1 } else { seed };
        for i in 1..31 {
            let prev = ring[i - 1] as i32 as i64;
            ring[i] = (16807 * prev).rem_euclid(2_147_483_647) as u32;
        }
        for i in 31..34 {
            ring[i] = ring[i - 31];
        }
        let mut r = Self { ring, i: 34 };
        for _ in 34..344 {
            r.advance();
        }
        r
    }

    fn advance(&mut self) -> u32 {
        // s[i] = s[i − 31] + s[i − 3], keeping the last 34 values.
        let v = self.ring[(self.i + 3) % 34].wrapping_add(self.ring[(self.i + 31) % 34]);
        self.ring[self.i % 34] = v;
        self.i += 1;
        v
    }

    /// The next output, as `rand()` returns it.
    pub fn next_u31(&mut self) -> u32 {
        self.advance() >> 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_glibc_seed_1() {
        let mut r = CRand::new(1);
        let got: Vec<u32> = (0..5).map(|_| r.next_u31()).collect();
        assert_eq!(
            got,
            [1804289383, 846930886, 1681692777, 1714636915, 1957747793]
        );
    }
}
