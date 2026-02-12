use anchor_lang::prelude::*;
use anchor_spl::token_interface::{approve, Approve, TokenAccount, TokenInterface};

use crate::state::{User, Vault};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub user: Signer<'info>,

    #[account(
        seeds = [b"vault"],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, User>,

    #[account(
        mut,
        token::mint = vault.mint,
        token::authority = vault,
        token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require_gte!(self.user_account.balance, amount);

        let signer_seeds: &[&[&[u8]]] =
            &[&[b"vault", self.vault.admin.as_ref(), &[self.vault.bump]]];

        approve(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Approve {
                    to: self.vault_token_account.to_account_info(),
                    delegate: self.user.to_account_info(),
                    authority: self.vault.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

        match self.user_account.balance.checked_sub(amount) {
            Some(x) => self.user_account.balance = x,
            None => {} // add error
        }

        Ok(())
    }
}
