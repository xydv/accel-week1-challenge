use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct User {
    pub balance: u64,
    pub bump: u8,
}
