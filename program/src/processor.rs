use {
    borsh::{BorshDeserialize, BorshSerialize},
    crate::{
        error::TransferError, 
        instruction::TransferInstruction, 
        state::{SPLVault, SPL_VAULT_LEN, AUTHORIZED_PUBLIC_KEY, NonceBank},
        utils::create_pda_account,
    },
    solana_program::{
        program::{invoke, invoke_signed},
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_instruction,
        rent::Rent,
        system_program,
        sysvar::Sysvar,
        program_pack::Pack,
        keccak,
        secp256k1_recover::secp256k1_recover,
    },
  
};

pub struct Processor {}

impl Processor {
    pub fn process_transfer(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        to: String,
        amount: u64,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let src_account = next_account_info(accounts_iter)?;
        let vault_account = next_account_info(accounts_iter)?;
        let vault_spl_account = next_account_info(accounts_iter)?;
        let src_owner_account = next_account_info(accounts_iter)?;
        let mint_account = next_account_info(accounts_iter)?;
        let spl_token_program_account = next_account_info(accounts_iter)?;
        let system_program_account = next_account_info(accounts_iter)?;
        let rent_program_account = next_account_info(accounts_iter)?;
        let spl_token_program_id = spl_token_program_account.key;

        let (gen_vault_account, gen_vault_account_bump) = Pubkey::find_program_address(
            &[
                b"vault",
                &spl_token_program_account.key.to_bytes(),
                &mint_account.key.to_bytes(),
            ],
            program_id,
        );
        if gen_vault_account != *vault_account.key {
            msg!("Error: vault_account address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }

        let vault_account_data_len  = vault_account.data_len();
        if vault_account_data_len != 0 && vault_account_data_len != SPL_VAULT_LEN {
            return Err(ProgramError::InvalidAccountData)
        }
        if vault_account_data_len == 0 {
            let rent = Rent::get()?;
            let vault_account_signer_seeds: &[&[_]] = &[
                b"vault",
                &spl_token_program_id.to_bytes(),
                &mint_account.key.to_bytes(),
                &[gen_vault_account_bump],
            ];
    
            create_pda_account(
                src_owner_account,
                &rent,
                SPL_VAULT_LEN,
                program_id,
                system_program_account,
                vault_account,
                vault_account_signer_seeds
            )?;
            let spl_vault = SPLVault{
                state: 0x1,
                mint: *mint_account.key,
                amount: 0,
            };
            spl_vault.serialize(&mut *vault_account.data.borrow_mut())?;
        }

        let (gen_vault_spl_account, gen_vault_spl_account_bump) = Pubkey::find_program_address(
            &[
                b"vault_spl",
                &spl_token_program_account.key.to_bytes(),
                &mint_account.key.to_bytes(),
            ],
            program_id,
        ); 
        if gen_vault_spl_account != *vault_spl_account.key {
            msg!("Error: vault_spl_account address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }
        if mint_account.key ==system_program_account.key {
            if vault_spl_account.lamports() == 0 {
                let rent = Rent::get()?;
                let vault_spl_account_signer_seeds: &[&[_]] = &[
                    b"vault_spl",
                    &spl_token_program_id.to_bytes(),
                    &mint_account.key.to_bytes(),
                    &[gen_vault_spl_account_bump],
                ];
        
                create_pda_account(
                    src_owner_account,
                    &rent,
                    0,
                    system_program_account.key,
                    system_program_account,
                    vault_spl_account,
                    vault_spl_account_signer_seeds
                )?;
            }

            invoke(
                &system_instruction::transfer(
                    src_owner_account.key,
                    vault_spl_account.key,
                    amount,
                ),
                &[src_owner_account.clone(),vault_spl_account.clone()],
            )?;
            msg!("{} transfer {} SOL to {}", src_account.key, amount, to);
        } else {
            let vault_spl_account_data_len  = vault_spl_account.data_len();
            if vault_spl_account_data_len != 0 && vault_spl_account_data_len != spl_token::state::Account::LEN {
                return Err(ProgramError::InvalidAccountData)
            }

            if vault_spl_account_data_len == 0 {
                let rent = Rent::get()?;
                let vault_spl_account_signer_seeds: &[&[_]] = &[
                    b"vault_spl",
                    &spl_token_program_id.to_bytes(),
                    &mint_account.key.to_bytes(),
                    &[gen_vault_spl_account_bump],
                ];
        
                create_pda_account(
                    src_owner_account,
                    &rent,
                    spl_token::state::Account::LEN,
                    spl_token_program_id,
                    system_program_account,
                    vault_spl_account,
                    vault_spl_account_signer_seeds
                )?;
                invoke_signed(
                    &system_instruction::assign(vault_spl_account.key, &spl_token::id()),
                    &[
                        vault_spl_account.clone(),
                        system_program_account.clone(),
                    ],
                    &[&vault_spl_account_signer_seeds],
                )?;

                invoke(
                    &spl_token::instruction::initialize_account(
                        &spl_token::id(),
                        vault_spl_account.key,
                        mint_account.key,
                        vault_account.key,
                    )?,
                    &[
                        vault_spl_account.clone(),
                        mint_account.clone(),
                        vault_account.clone(),
                        rent_program_account.clone(),
                    ],
                )?;
            }

            invoke(
                &spl_token::instruction::transfer(
                    spl_token_program_account.key,
                    src_account.key,
                    vault_spl_account.key,
                    src_owner_account.key,
                    &[],
                    amount,
                )?,
                &[src_account.clone(),vault_spl_account.clone(),src_owner_account.clone(),spl_token_program_account.clone()],
            )?;
            msg!("{} transfer {} {} to {}", src_account.key, amount, mint_account.key, to);
        }
        Ok(())
    }

  
    pub fn process_withdraw(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        nonce:u64,
        slot:u64,
        signature: String,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let withdraw_account = next_account_info(accounts_iter)?;
        let vault_account = next_account_info(accounts_iter)?;
        let vault_spl_account = next_account_info(accounts_iter)?;
        let withdraw_spl_account = next_account_info(accounts_iter)?;
        let nonce_bank_account = next_account_info(accounts_iter)?;
        let mint_account = next_account_info(accounts_iter)?;
        let spl_token_program_account = next_account_info(accounts_iter)?;
        let system_program_account = next_account_info(accounts_iter)?;
        //let rent_program_account = next_account_info(accounts_iter)?;
        let spl_token_program_id = spl_token_program_account.key;

        // vault account
        let (gen_vault_account, gen_vault_account_bump) = Pubkey::find_program_address(
            &[
                b"vault",
                &spl_token_program_account.key.to_bytes(),
                &mint_account.key.to_bytes(),
            ],
            program_id,
        );
        if gen_vault_account != *vault_account.key {
            msg!("Error: vault_account address does not match seed derivation {} {}", gen_vault_account, *vault_account.key);
            return Err(ProgramError::InvalidSeeds);
        }

        let vault_account_data_len  = vault_account.data_len();
        if vault_account_data_len != 0 && vault_account_data_len != SPL_VAULT_LEN {
            return Err(ProgramError::InvalidAccountData)
        }

        // nonce bank account
        let (gen_nonce_bank_account, gen_nonce_bank_account_bump) = Pubkey::find_program_address(
            &[
                b"nonce_bank",
                &nonce.to_le_bytes(),
            ],
            program_id,
        );
        if gen_nonce_bank_account != *nonce_bank_account.key {
            msg!("Error: nonce_bank_account address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }

        let new_nonce_bank = NonceBank{ 
            state:0x2u8,
            nonce: nonce, 
            withdrawer:*withdraw_account.key 
        };
        if nonce_bank_account.data_len() >0 {
            msg!("Error: nonce is used");
            return Err(ProgramError::InvalidArgument);
        }
        
        let gen_nonce_bank_account_signer_seeds: &[&[_]] =&[
            b"nonce_bank",
            &nonce.to_le_bytes(),
            &[gen_nonce_bank_account_bump],
        ]; 
        let rent = Rent::get()?;
        create_pda_account(
            withdraw_account,
            &rent,
            new_nonce_bank.try_to_vec()?.len(),
            program_id,
            system_program_account,
            nonce_bank_account,
            gen_nonce_bank_account_signer_seeds
        )?;

        new_nonce_bank.serialize(&mut *nonce_bank_account.data.borrow_mut())?;
        msg!("after write nonce bank");
        // vault spl account 
        let (gen_vault_spl_account, gen_vault_spl_account_bump) = Pubkey::find_program_address(
            &[
                b"vault_spl",
                &spl_token_program_account.key.to_bytes(),
                &mint_account.key.to_bytes(),
            ],
            program_id,
        ); 
        if gen_vault_spl_account != *vault_spl_account.key {
            msg!("Error: vault_spl_account address does not match seed derivation");
            return Err(ProgramError::InvalidSeeds);
        }

        let vault_spl_account_data_len  = vault_spl_account.data_len();
        if vault_spl_account_data_len != 0 && vault_spl_account_data_len != spl_token::state::Account::LEN {
            return Err(ProgramError::InvalidAccountData)
        }

        let message = format!("{}-{}-{}-{}-{}", mint_account.key.to_string(), withdraw_account.key.to_string(), amount, slot, nonce);;
        msg!("message:{}",message);

        let message_hash = {
            let mut hasher = keccak::Hasher::default();
            hasher.hash(message.as_bytes());
            hasher.result()
        };

        let vault_account_signer_seeds: &[&[_]] = &[
            b"vault",
            &spl_token_program_id.to_bytes(),
            &mint_account.key.to_bytes(),
            &[gen_vault_account_bump],
        ];
        let vault_spl_account_signer_seeds: &[&[_]] = &[
            b"vault_spl",
            &spl_token_program_id.to_bytes(),
            &mint_account.key.to_bytes(),
            &[gen_vault_spl_account_bump],
        ];
        msg!("invoke transfer");
        if mint_account.key ==system_program_account.key {
            invoke_signed(
                &system_instruction::transfer(
                    vault_spl_account.key,
                    withdraw_account.key,
                    amount,
                ),
                &[vault_spl_account.clone(),withdraw_account.clone()],
                &[&vault_spl_account_signer_seeds] 
            )?;
            msg!("{} withdraw {}  on nonce {}", withdraw_account.key, amount, nonce);
        } else {
            invoke_signed(
                &spl_token::instruction::transfer(
                    spl_token_program_account.key,
                    vault_spl_account.key,
                    withdraw_spl_account.key,
                    vault_account.key,
                    &[],
                    amount,
                )?,
                &[vault_spl_account.clone(),withdraw_spl_account.clone(),vault_account.clone(),spl_token_program_account.clone()],
                &[&vault_account_signer_seeds] 
            )?;

            msg!("{} withdraw {} {} on nonce {}", withdraw_account.key, amount, mint_account.key, nonce);
        }
        Ok(())
    }

    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = TransferInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        msg!("Instruction unpacked");

        match instruction {
            TransferInstruction::Transfer{ 
                to,
                amount} => {
                Processor::process_transfer(program_id, accounts, to, amount)?;
            }
            TransferInstruction::Withdraw{ 
                amount,
                nonce,
                slot,
                signature } => {
                Processor::process_withdraw(program_id, accounts, amount,nonce, slot, signature )?;
            }
        }
        Ok(())
    }
}
