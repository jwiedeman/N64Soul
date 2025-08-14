pub struct Dbuf<const N: usize> {
    cur: [u8; N],
    nxt: [u8; N],
}

impl<const N: usize> Dbuf<N> {
    pub const fn new() -> Self { Self { cur: [0; N], nxt: [0; N] } }
    #[inline(always)]
    pub fn cur_mut(&mut self) -> &mut [u8; N] { &mut self.cur }
    #[inline(always)]
    pub fn nxt_mut(&mut self) -> &mut [u8; N] { &mut self.nxt }
    #[inline(always)]
    pub fn swap(&mut self) { core::mem::swap(&mut self.cur, &mut self.nxt); }
}

