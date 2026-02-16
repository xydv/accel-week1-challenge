use anchor_lang::{
    prelude::*,
    solana_program::sysvar::instructions::{
        load_current_index_checked, load_instruction_at_checked,
    },
};
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

    /// CHECK: instructions sysvar account
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require_gte!(self.user_account.balance, amount);

        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", &[self.vault.bump]]];

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

    pub fn check_transfer_instruction(&self, amount: u64) -> Result<()> {
        // instruction introspection
        let current_index =
            load_current_index_checked(&self.instructions.to_account_info())? as usize;
        let ix =
            load_instruction_at_checked(current_index + 1, &self.instructions.to_account_info())?;

        // check below ix is from token22 program and transferchecked
        require_keys_eq!(ix.program_id, anchor_spl::token_2022::ID);
        require_eq!(ix.data.split_first().unwrap().0, &12);

        let authority = ix.accounts.get(4).unwrap();
        require_keys_eq!(authority.pubkey, self.user.key());

        let amount_bytes = &ix.data[1..9];
        let amount_ix = u64::from_le_bytes(amount_bytes.try_into().unwrap());
        require_eq!(amount_ix, amount);

        Ok(())
    }
}
