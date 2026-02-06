use anchor_lang::prelude::*;

use crate::state::Whitelist;

#[account]
// #[derive(Ini)]
pub struct Vault {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub vault_token_account: Pubkey,
    pub whilelisted: Vec<Whitelist>,
    pub bump: u8,
}
