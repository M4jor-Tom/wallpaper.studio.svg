//! CPython's `random` module, reimplemented for the calls this program makes.
//! The golden corpus pins this stream: each call must consume exactly CPython's number of 32-bit words.
use sha2::{Digest, Sha512};

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908_b0df;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;

pub struct PyRandom {
    mt: [u32; N],
    mti: usize,
}

impl PyRandom {
    /// `random.Random(s)`: `int.from_bytes(s + sha512(s))`, split into LE 32-bit words for `init_by_array`.
    pub fn new(seed: &str) -> Self {
        let mut be = seed.as_bytes().to_vec();
        be.extend_from_slice(&Sha512::digest(seed.as_bytes()));

        // The integer's magnitude, so leading zero bytes carry no words.
        let lead = be.iter().take_while(|b| **b == 0).count();
        let be = &be[lead..];
        let key: Vec<u32> = if be.is_empty() {
            vec![0]
        } else {
            let bits = be.len() * 8 - be[0].leading_zeros() as usize;
            let words = (bits - 1) / 32 + 1;
            let mut le = be.to_vec();
            le.reverse();
            le.resize(words * 4, 0);
            le.chunks(4)
                .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
                .collect()
        };
        Self::init_by_array(&key)
    }

    fn init_genrand(s: u32) -> Self {
        let mut mt = [0u32; N];
        mt[0] = s;
        for i in 1..N {
            mt[i] = 1812433253u32
                .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Self { mt, mti: N }
    }

    fn init_by_array(key: &[u32]) -> Self {
        let mut r = Self::init_genrand(19650218);
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..N.max(key.len()) {
            r.mt[i] = (r.mt[i] ^ ((r.mt[i - 1] ^ (r.mt[i - 1] >> 30)).wrapping_mul(1664525)))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                r.mt[0] = r.mt[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..N - 1 {
            r.mt[i] = (r.mt[i] ^ ((r.mt[i - 1] ^ (r.mt[i - 1] >> 30)).wrapping_mul(1566083941)))
                .wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                r.mt[0] = r.mt[N - 1];
                i = 1;
            }
        }
        r.mt[0] = 0x8000_0000;
        r
    }

    fn genrand_u32(&mut self) -> u32 {
        if self.mti >= N {
            for i in 0..N {
                let y = (self.mt[i] & UPPER_MASK) | (self.mt[(i + 1) % N] & LOWER_MASK);
                self.mt[i] =
                    self.mt[(i + M) % N] ^ (y >> 1) ^ if y & 1 != 0 { MATRIX_A } else { 0 };
            }
            self.mti = 0;
        }
        let mut y = self.mt[self.mti];
        self.mti += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }

    /// `random()` -- always two words, as `genrand_res53`.
    pub fn random(&mut self) -> f64 {
        let a = self.genrand_u32() >> 5;
        let b = self.genrand_u32() >> 6;
        (a as f64 * 67108864.0 + b as f64) * (1.0 / 9007199254740992.0)
    }

    pub fn getrandbits(&mut self, k: u32) -> u32 {
        assert!((1..=32).contains(&k), "getrandbits({k}) is out of range");
        self.genrand_u32() >> (32 - k)
    }

    /// `_randbelow(n)` -- rejection sampling; a modulo shortcut would desync the stream.
    fn randbelow(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        let k = 32 - n.leading_zeros();
        loop {
            let r = self.getrandbits(k);
            if r < n {
                return r;
            }
        }
    }

    pub fn randrange(&mut self, n: u32) -> u32 {
        self.randbelow(n)
    }

    /// `shuffle(x)` -- Fisher-Yates walking down, as CPython.
    pub fn shuffle<T>(&mut self, xs: &mut [T]) {
        for i in (1..xs.len()).rev() {
            let j = self.randbelow(i as u32 + 1) as usize;
            xs.swap(i, j);
        }
    }

    /// `sample(range(n), k)`, pool branch only: every call here is far below CPython's set threshold.
    pub fn sample(&mut self, n: usize, k: usize) -> Vec<usize> {
        let mut pool: Vec<usize> = (0..n).collect();
        let mut out = Vec::with_capacity(k);
        for i in 0..k {
            let j = self.randbelow((n - i) as u32) as usize;
            out.push(pool[j]);
            pool[j] = pool[n - i - 1];
        }
        out
    }

    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.random()
    }

    pub fn choice(&mut self, s: &[u8]) -> u8 {
        s[self.randbelow(s.len() as u32) as usize]
    }
}

/// Per-cell stream keyed by coordinates, so one cell's draws never shift another's.
pub fn cell_rng(tag: &str, seed: u32, r: i32, c: i32) -> PyRandom {
    PyRandom::new(&format!("{tag}:{seed}:{r}:{c}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Captured from CPython 3.14.7. If these move, every golden moves.
    #[test]
    fn matches_cpython() {
        let mut r = PyRandom::new("trihex:0");
        assert_eq!(
            [r.random(), r.random(), r.random()],
            [0.323979587515701, 0.480793333456907, 0.521798912248572]
        );

        let mut r = PyRandom::new("trihex:0");
        assert_eq!(
            [
                r.getrandbits(32),
                r.getrandbits(32),
                r.getrandbits(32),
                r.getrandbits(32)
            ],
            [1391481750, 664579869, 2064991622, 3668470967]
        );

        let mut r = PyRandom::new("trihex:0");
        assert_eq!(r.sample(6, 6), vec![2, 1, 3, 5, 4, 0]);

        let mut r = PyRandom::new("trihex:0");
        let mut xs: Vec<u32> = (0..10).collect();
        r.shuffle(&mut xs);
        assert_eq!(xs, vec![9, 8, 1, 0, 3, 4, 6, 7, 2, 5]);

        assert_eq!(
            PyRandom::new("rain:0:0:3").uniform(18.0, 34.0),
            18.93273603129014
        );
        assert_eq!(PyRandom::new("rain:0:0:3").randrange(97), 7);
        assert_eq!(
            PyRandom::new("star:0:-1:-1").choice(b"0123456789ABCDEF"),
            b'2'
        );
    }
}
