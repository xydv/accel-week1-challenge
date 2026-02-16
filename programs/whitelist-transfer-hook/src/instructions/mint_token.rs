use anchor_lang::{
    prelude::*,
    solana_program::rent::{DEFAULT_EXEMPTION_THRESHOLD, DEFAULT_LAMPORTS_PER_BYTE_YEAR},
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::MintTo,
    token_interface::{
        self, token_metadata_initialize, Mint, TokenAccount, TokenInterface,
        TokenMetadataInitialize,
    },
};
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;

#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub user: SystemAccount<'info>,

    #[account(
        init_if_needed,
        payer = admin,
        mint::decimals = 9,
        mint::authority = admin,
        extensions::transfer_hook::authority = admin,
        extensions::transfer_hook::program_id = crate::ID,
        extensions::metadata_pointer::authority = admin,
        extensions::metadata_pointer::metadata_address = mint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> TokenFactory<'info> {
    pub fn init_mint(
        &mut self,
        name: String,
        symbol: String,
        uri: String,
        amount: u64,
    ) -> Result<()> {
        // https://github.com/solana-developers/program-examples/blob/main/tokens/token-2022/metadata/anchor/programs/metadata/src/instructions/initialize.rs
        let token_metadata = TokenMetadata {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            ..Default::default()
        };

        // Add 4 extra bytes for size of MetadataExtension (2 bytes for type, 2 bytes for length)
        let data_len = 4 + token_metadata.get_packed_len()?;

        // Calculate lamports required for the additional metadata
        let lamports =
            data_len as u64 * DEFAULT_LAMPORTS_PER_BYTE_YEAR * DEFAULT_EXEMPTION_THRESHOLD as u64;

        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.admin.to_account_info(),
                    to: self.mint.to_account_info(),
                },
            ),
            lamports,
        )?;

        token_metadata_initialize(
            CpiContext::new(
                self.token_program.to_account_info(),
                TokenMetadataInitialize {
                    mint: self.mint.to_account_info(),
                    metadata: self.mint.to_account_info(), // points to self
                    mint_authority: self.admin.to_account_info(),
                    update_authority: self.admin.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                },
            ),
            name,
            symbol,
            uri,
        )?;

        let cpi = CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.mint.to_account_info(),
                to: self.user_ata.to_account_info(),
                authority: self.admin.to_account_info(),
            },
        );

        token_interface::mint_to(cpi, amount)?;

        Ok(())
    }
}
