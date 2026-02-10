use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub vault_token_account: Pubkey,
    pub bump: u8,
}
