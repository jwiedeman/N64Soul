pub struct Dbuf<const N: usize> {
    a: [u8; N],
    b: [u8; N],
    use_a: bool,
}

impl<const N: usize> Dbuf<N> {
    pub const fn new() -> Self {
        Self {
            a: [0; N],
            b: [0; N],
            use_a: true,
        }
    }

    pub fn pair(&mut self) -> (&mut [u8; N], &mut [u8; N]) {
        if self.use_a {
            self.use_a = false;
            (&mut self.a, &mut self.b)
        } else {
            self.use_a = true;
            (&mut self.b, &mut self.a)
        }
    }
}
