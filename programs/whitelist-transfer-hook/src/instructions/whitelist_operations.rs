use anchor_lang::prelude::*;

use crate::state::{User, Vault};

#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct AddToWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        has_one = admin,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init,
        payer = admin,
        space = User::DISCRIMINATOR.len() + User::INIT_SPACE,
        seeds = [b"user", address.key().as_ref()],
        bump,
    )]
    pub user: Account<'info, User>,

    pub system_program: Program<'info, System>,
}

impl<'info> AddToWhitelist<'info> {
    pub fn add_to_whitelist(
        &mut self,
        _address: Pubkey,
        bumps: &AddToWhitelistBumps,
    ) -> Result<()> {
        self.user.set_inner(User {
            balance: 0,
            bump: bumps.user,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        has_one = admin,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        close = admin,
        seeds = [b"user", address.key().as_ref()],
        bump = user.bump,
    )]
    pub user: Account<'info, User>,

    pub system_program: Program<'info, System>,
}

impl<'info> RemoveFromWhitelist<'info> {
    pub fn remove_from_whitelist(&mut self, _address: Pubkey) -> Result<()> {
        Ok(())
    }
}
