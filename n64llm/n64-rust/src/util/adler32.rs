pub fn adler32(mut s1: u32, mut s2: u32, data: &[u8]) -> (u32, u32) {
    const MOD: u32 = 65521;
    for &b in data {
        s1 = (s1 + b as u32) % MOD;
        s2 = (s2 + s1) % MOD;
    }
    (s1, s2)
}

