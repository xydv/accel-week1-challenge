#[cfg(test)]
mod tests {

    use {
        anchor_lang::{prelude::msg, AccountDeserialize, InstructionData, ToAccountMetas},
        anchor_spl::{associated_token, token_2022},
        litesvm::LiteSVM,
        solana_account::Account,
        solana_address::Address,
        solana_instruction::{AccountMeta, Instruction},
        solana_keypair::Keypair,
        solana_message::Message,
        solana_native_token::LAMPORTS_PER_SOL,
        solana_pubkey::Pubkey,
        solana_rpc_client::rpc_client::RpcClient,
        solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID,
        solana_signer::Signer,
        solana_transaction::Transaction,
        std::{path::PathBuf, str::FromStr},
    };

    static PROGRAM_ID: Pubkey = crate::ID;

    // Setup function to initialize LiteSVM and create a payer keypair
    // Also loads an account from devnet into the LiteSVM environment (for testing purposes)
    fn setup() -> (LiteSVM, Keypair) {
        // Initialize LiteSVM and payer
        let mut program = LiteSVM::new();
        let payer = Keypair::new();

        // Airdrop some SOL to the payer keypair
        program
            .airdrop(&payer.pubkey(), 100 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        // Load program SO file
        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/whitelist_transfer_hook.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        program.add_program(PROGRAM_ID, &program_data);

        // Example on how to Load an account from devnet
        // LiteSVM does not have access to real Solana network data since it does not have network access,
        // so we use an RPC client to fetch account data from devnet
        let rpc_client = RpcClient::new("https://api.devnet.solana.com");
        let account_address =
            Address::from_str("DRYvf71cbF2s5wgaJQvAGkghMkRcp5arvsK2w97vXhi2").unwrap();
        let fetched_account = rpc_client
            .get_account(&account_address)
            .expect("Failed to fetch account from devnet");

        // Set the fetched account in the LiteSVM environment
        // This allows us to simulate interactions with this account during testing
        program
            .set_account(
                payer.pubkey(),
                Account {
                    lamports: 100 * LAMPORTS_PER_SOL,
                    data: fetched_account.data,
                    owner: Pubkey::from(fetched_account.owner.to_bytes()),
                    executable: fetched_account.executable,
                    rent_epoch: fetched_account.rent_epoch,
                },
            )
            .unwrap();

        msg!("Lamports of fetched account: {}", fetched_account.lamports);

        // Return the LiteSVM instance and payer keypair
        (program, payer)
    }

    #[test]
    fn test_deposit() {
        let (mut program, admin) = setup();
        let admin_pubkey = admin.pubkey();

        let user = Keypair::new();

        program
            .airdrop(&user.pubkey(), 100 * LAMPORTS_PER_SOL)
            .unwrap();

        let (vault_pda, _v_bump) = Pubkey::find_program_address(&[b"vault"], &PROGRAM_ID);
        let (user_state_pda, _u_bump) =
            Pubkey::find_program_address(&[b"user", user.pubkey().as_ref()], &PROGRAM_ID);

        let mint_keypair = Keypair::new();
        let mint_pubkey = mint_keypair.pubkey();

        let user_ata = associated_token::get_associated_token_address_with_program_id(
            &user.pubkey(),
            &mint_pubkey,
            &anchor_spl::token_2022::ID,
        );

        let vault_ata = associated_token::get_associated_token_address_with_program_id(
            &vault_pda,
            &mint_pubkey,
            &anchor_spl::token_2022::ID,
        );

        let (meta_list_pda, _bump) = Pubkey::find_program_address(
            &[b"extra-account-metas", mint_pubkey.as_ref()],
            &PROGRAM_ID,
        );

        let setup_ixs = vec![
            // Init Mint and Send to user
            Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::TokenFactory {
                    admin: admin_pubkey,
                    user: user.pubkey(),
                    mint: mint_keypair.pubkey(),
                    user_ata: user_ata,
                    system_program: solana_sdk_ids::system_program::ID,
                    token_program: anchor_spl::token_2022::ID,
                    associated_token_program: associated_token::ID,
                }
                .to_account_metas(None),
                data: crate::instruction::MintToken {
                    amount: 1_000_000_000_000,
                    name: "test token".to_string(),
                    symbol: "TEST".to_string(),
                    uri: "".to_string(),
                }
                .data(),
            },
            // Init extra account meta list
            Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::InitializeExtraAccountMetaList {
                    payer: admin_pubkey,
                    extra_account_meta_list: meta_list_pda,
                    mint: mint_pubkey,
                    system_program: solana_sdk_ids::system_program::ID,
                }
                .to_account_metas(None),
                data: crate::instruction::InitializeTransferHook {}.data(),
            },
            // Initialize Vault
            Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::InitializeVault {
                    admin: admin_pubkey,
                    vault: vault_pda,
                    mint: mint_pubkey,
                    vault_token_account: vault_ata,
                    associated_token_program: associated_token::ID,
                    token_program: anchor_spl::token_2022::ID,
                    system_program: SYSTEM_PROGRAM_ID,
                }
                .to_account_metas(None),
                data: crate::instruction::InitializeVault {}.data(),
            },
            // Add User to Whitelist
            Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::AddToWhitelist {
                    admin: admin_pubkey,
                    vault: vault_pda,
                    user: user_state_pda,
                    system_program: SYSTEM_PROGRAM_ID,
                }
                .to_account_metas(None),
                data: crate::instruction::AddToWhitelist {
                    user: user.pubkey(),
                }
                .data(),
            },
        ];

        program
            .send_transaction(Transaction::new(
                &[&admin, &mint_keypair],
                Message::new(&setup_ixs, Some(&admin_pubkey)),
                program.latest_blockhash(),
            ))
            .unwrap();

        let mut transfer_ix =
            anchor_spl::token_2022::spl_token_2022::instruction::transfer_checked(
                &anchor_spl::token_2022::ID,
                &user_ata,
                &mint_pubkey,
                &vault_ata,
                &user.pubkey(),
                &[&user.pubkey()],
                10_000_000_000, // 10 tokens
                9,
            )
            .unwrap();

        transfer_ix.accounts.extend(vec![
            AccountMeta::new_readonly(meta_list_pda, false),
            AccountMeta::new_readonly(vault_pda, false),
            AccountMeta::new_readonly(user_state_pda, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ]);

        let deposit_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Deposit {
                user: user.pubkey(),
                user_state: user_state_pda,
                user_ata,
                vault_ata,
                vault: vault_pda,
                mint: mint_pubkey,
                instructions: anchor_lang::solana_program::sysvar::instructions::ID,
                system_program: SYSTEM_PROGRAM_ID,
                token_program: anchor_spl::token_2022::ID,
                associated_token_program: associated_token::ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Deposit {}.data(),
        };

        let message = Message::new(&[transfer_ix, deposit_ix], Some(&user.pubkey()));
        let tx = Transaction::new(&[&user], message, program.latest_blockhash());

        let tx_res = program
            .send_transaction(tx)
            .expect("Introspection check failed");

        msg!("Deposit successful. CUs: {}", tx_res.compute_units_consumed);

        let user_state_account = program.get_account(&user_state_pda).unwrap();
        let user_state =
            crate::state::User::try_deserialize(&mut user_state_account.data.as_ref()).unwrap();

        assert_eq!(
            user_state.balance, 10_000_000_000,
            "User state balance should match deposit"
        );

        let vault_ata_account = program.get_account(&vault_ata).unwrap();

        // get amount manually, unpacking throws error
        let vault_amount = u64::from_le_bytes(vault_ata_account.data[64..72].try_into().unwrap());

        assert_eq!(
            vault_amount, 10_000_000_000,
            "Vault ATA should have received tokens"
        );
    }

    #[test]
    fn test_withdraw() {
        let (mut program, admin) = setup();
        let admin_pubkey = admin.pubkey();

        let user = Keypair::new();

        program
            .airdrop(&user.pubkey(), 100 * LAMPORTS_PER_SOL)
            .unwrap();

        let (vault_pda, _v_bump) = Pubkey::find_program_address(&[b"vault"], &PROGRAM_ID);
        let (user_state_pda, _u_bump) =
            Pubkey::find_program_address(&[b"user", user.pubkey().as_ref()], &PROGRAM_ID);

        let mint_keypair = Keypair::new();
        let mint_pubkey = mint_keypair.pubkey();

        let user_ata = associated_token::get_associated_token_address_with_program_id(
            &user.pubkey(),
            &mint_pubkey,
            &anchor_spl::token_2022::ID,
        );

        let vault_ata = associated_token::get_associated_token_address_with_program_id(
            &vault_pda,
            &mint_pubkey,
            &anchor_spl::token_2022::ID,
        );

        let (meta_list_pda, _bump) = Pubkey::find_program_address(
            &[b"extra-account-metas", mint_pubkey.as_ref()],
            &PROGRAM_ID,
        );

        let setup_ixs = vec![
            // Init Mint and Send to user
            Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::TokenFactory {
                    admin: admin_pubkey,
                    user: user.pubkey(),
                    mint: mint_keypair.pubkey(),
                    user_ata: user_ata,
                    system_program: solana_sdk_ids::system_program::ID,
                    token_program: anchor_spl::token_2022::ID,
                    associated_token_program: associated_token::ID,
                }
                .to_account_metas(None),
                data: crate::instruction::MintToken {
                    amount: 1_000_000_000_000, // 1000 tokens
                    name: "test token".to_string(),
                    symbol: "TEST".to_string(),
                    uri: "".to_string(),
                }
                .data(),
            },
            // Init extra account meta list
            Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::InitializeExtraAccountMetaList {
                    payer: admin_pubkey,
                    extra_account_meta_list: meta_list_pda,
                    mint: mint_pubkey,
                    system_program: solana_sdk_ids::system_program::ID,
                }
                .to_account_metas(None),
                data: crate::instruction::InitializeTransferHook {}.data(),
            },
            // Initialize Vault
            Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::InitializeVault {
                    admin: admin_pubkey,
                    vault: vault_pda,
                    mint: mint_pubkey,
                    vault_token_account: vault_ata,
                    associated_token_program: associated_token::ID,
                    token_program: anchor_spl::token_2022::ID,
                    system_program: SYSTEM_PROGRAM_ID,
                }
                .to_account_metas(None),
                data: crate::instruction::InitializeVault {}.data(),
            },
            // Add User to Whitelist
            Instruction {
                program_id: PROGRAM_ID,
                accounts: crate::accounts::AddToWhitelist {
                    admin: admin_pubkey,
                    vault: vault_pda,
                    user: user_state_pda,
                    system_program: SYSTEM_PROGRAM_ID,
                }
                .to_account_metas(None),
                data: crate::instruction::AddToWhitelist {
                    user: user.pubkey(),
                }
                .data(),
            },
        ];

        program
            .send_transaction(Transaction::new(
                &[&admin, &mint_keypair],
                Message::new(&setup_ixs, Some(&admin_pubkey)),
                program.latest_blockhash(),
            ))
            .unwrap();

        let mut transfer_ix =
            anchor_spl::token_2022::spl_token_2022::instruction::transfer_checked(
                &anchor_spl::token_2022::ID,
                &user_ata,
                &mint_pubkey,
                &vault_ata,
                &user.pubkey(),
                &[&user.pubkey()],
                10_000_000_000, // 10 tokens
                9,
            )
            .unwrap();

        transfer_ix.accounts.extend(vec![
            AccountMeta::new_readonly(meta_list_pda, false),
            AccountMeta::new_readonly(vault_pda, false),
            AccountMeta::new_readonly(user_state_pda, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ]);

        let deposit_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Deposit {
                user: user.pubkey(),
                user_state: user_state_pda,
                user_ata,
                vault_ata,
                vault: vault_pda,
                mint: mint_pubkey,
                instructions: anchor_lang::solana_program::sysvar::instructions::ID,
                system_program: SYSTEM_PROGRAM_ID,
                token_program: anchor_spl::token_2022::ID,
                associated_token_program: associated_token::ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Deposit {}.data(),
        };

        let message = Message::new(&[transfer_ix, deposit_ix], Some(&user.pubkey()));
        let tx = Transaction::new(&[&user], message, program.latest_blockhash());

        // send deposit transaction
        program
            .send_transaction(tx)
            .expect("Introspection check failed");

        let withdraw_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Withdraw {
                user: user.pubkey(),
                vault: vault_pda,
                user_account: user_state_pda,
                vault_token_account: vault_ata,
                token_program: token_2022::ID,
                instructions: anchor_lang::solana_program::sysvar::instructions::ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Withdraw {
                amount: 10_000_000_000,
            }
            .data(),
        };

        let mut transfer_out_ix = spl_token_2022::instruction::transfer_checked(
            &token_2022::ID,
            &vault_ata,
            &mint_pubkey,
            &user_ata,
            &user.pubkey(),
            &[],
            10_000_000_000,
            9,
        )
        .unwrap();

        transfer_out_ix.accounts.extend(vec![
            AccountMeta::new_readonly(meta_list_pda, false),
            AccountMeta::new_readonly(vault_pda, false),
            AccountMeta::new_readonly(user_state_pda, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
        ]);

        let withdraw_msg = Message::new(&[withdraw_ix, transfer_out_ix], Some(&user.pubkey()));

        program
            .send_transaction(Transaction::new(
                &[&user],
                withdraw_msg,
                program.latest_blockhash(),
            ))
            .expect("Failed to withdraw");

        let user_state_acc = program.get_account(&user_state_pda).unwrap();
        let user_state =
            crate::state::User::try_deserialize(&mut user_state_acc.data.as_ref()).unwrap();

        assert_eq!(user_state.balance, 0);
    }
}
