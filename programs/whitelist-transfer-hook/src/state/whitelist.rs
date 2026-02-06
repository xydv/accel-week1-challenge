use anchor_lang::prelude::*;

#[account]
// #[derive(Ini)]
pub struct Whitelist {
    pub address: Vec<Pubkey>,
    pub bump: u8,
}
