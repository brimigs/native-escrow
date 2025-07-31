use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Escrow {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
    pub bump: u8,
    pub _padding: [u8; 7], // Add padding to make the struct 8-byte aligned
}

impl Escrow {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 1 + 7;

    pub fn init(&mut self, seed: u64, maker: Pubkey, mint_a: Pubkey, mint_b: Pubkey, receive: u64, bump: u8) {
        self.seed = seed;
        self.maker = maker;
        self.mint_a = mint_a;
        self.mint_b = mint_b;
        self.receive = receive;
        self.bump = bump;
        self._padding = [0; 7];
    }
}