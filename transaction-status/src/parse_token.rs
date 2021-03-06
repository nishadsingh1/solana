use crate::parse_instruction::{ParsableProgram, ParseInstructionError};
use serde_json::{json, Map, Value};
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use spl_token_v1_0::instruction::TokenInstruction;

pub fn parse_token(
    instruction: &CompiledInstruction,
    account_keys: &[Pubkey],
) -> Result<Value, ParseInstructionError> {
    let token_instruction = TokenInstruction::unpack(&instruction.data)
        .map_err(|_| ParseInstructionError::InstructionNotParsable(ParsableProgram::SplToken))?;
    if instruction.accounts.len() > account_keys.len() {
        // Runtime should prevent this from ever happening
        return Err(ParseInstructionError::InstructionKeyMismatch(
            ParsableProgram::SplToken,
        ));
    }
    match token_instruction {
        TokenInstruction::InitializeMint { amount, decimals } => {
            if instruction.accounts.len() < 2 {
                return Err(ParseInstructionError::InstructionKeyMismatch(
                    ParsableProgram::SplToken,
                ));
            }
            let mut value = json!({
                "type": "initializeMint",
                "mint": account_keys[instruction.accounts[0] as usize].to_string(),
                "amount": amount,
                "decimals":decimals,
            });
            let map = value.as_object_mut().unwrap();
            if amount == 0 {
                map.insert(
                    "owner".to_string(),
                    json!(account_keys[instruction.accounts[1] as usize].to_string()),
                );
            } else {
                map.insert(
                    "account".to_string(),
                    json!(account_keys[instruction.accounts[1] as usize].to_string()),
                );
                if let Some(i) = instruction.accounts.get(2) {
                    map.insert(
                        "owner".to_string(),
                        json!(account_keys[*i as usize].to_string()),
                    );
                }
            }
            Ok(value)
        }
        TokenInstruction::InitializeAccount => {
            if instruction.accounts.len() < 3 {
                return Err(ParseInstructionError::InstructionKeyMismatch(
                    ParsableProgram::SplToken,
                ));
            }
            Ok(json!({
                "type": "initializeAccount",
                "account": account_keys[instruction.accounts[0] as usize].to_string(),
                "mint": account_keys[instruction.accounts[1] as usize].to_string(),
                "owner": account_keys[instruction.accounts[2] as usize].to_string(),
            }))
        }
        TokenInstruction::InitializeMultisig { m } => {
            if instruction.accounts.len() < 2 {
                return Err(ParseInstructionError::InstructionKeyMismatch(
                    ParsableProgram::SplToken,
                ));
            }
            let mut signers: Vec<String> = vec![];
            for i in instruction.accounts[1..].iter() {
                signers.push(account_keys[*i as usize].to_string());
            }
            Ok(json!({
                "type": "initializeMultisig",
                "multisig": account_keys[instruction.accounts[0] as usize].to_string(),
                "signers": signers,
                "m": m,
            }))
        }
        TokenInstruction::Transfer { amount } => {
            if instruction.accounts.len() < 3 {
                return Err(ParseInstructionError::InstructionKeyMismatch(
                    ParsableProgram::SplToken,
                ));
            }
            let mut value = json!({
                "type": "transfer",
                "source": account_keys[instruction.accounts[0] as usize].to_string(),
                "destination": account_keys[instruction.accounts[1] as usize].to_string(),
                "amount": amount,
            });
            let mut map = value.as_object_mut().unwrap();
            parse_signers(
                &mut map,
                2,
                account_keys,
                &instruction.accounts,
                "authority",
                "multisigAuthority",
            );
            Ok(value)
        }
        TokenInstruction::Approve { amount } => {
            if instruction.accounts.len() < 3 {
                return Err(ParseInstructionError::InstructionKeyMismatch(
                    ParsableProgram::SplToken,
                ));
            }
            let mut value = json!({
                "type": "approve",
                "source": account_keys[instruction.accounts[0] as usize].to_string(),
                "delegate": account_keys[instruction.accounts[1] as usize].to_string(),
                "amount": amount,
            });
            let mut map = value.as_object_mut().unwrap();
            parse_signers(
                &mut map,
                2,
                account_keys,
                &instruction.accounts,
                "owner",
                "multisigOwner",
            );
            Ok(value)
        }
        TokenInstruction::Revoke => {
            if instruction.accounts.len() < 2 {
                return Err(ParseInstructionError::InstructionKeyMismatch(
                    ParsableProgram::SplToken,
                ));
            }
            let mut value = json!({
                "type": "revoke",
                "source": account_keys[instruction.accounts[0] as usize].to_string(),
            });
            let mut map = value.as_object_mut().unwrap();
            parse_signers(
                &mut map,
                1,
                account_keys,
                &instruction.accounts,
                "owner",
                "multisigOwner",
            );
            Ok(value)
        }
        TokenInstruction::SetOwner => {
            if instruction.accounts.len() < 3 {
                return Err(ParseInstructionError::InstructionKeyMismatch(
                    ParsableProgram::SplToken,
                ));
            }
            let mut value = json!({
                "type": "setOwner",
                "owned": account_keys[instruction.accounts[0] as usize].to_string(),
                "newOwner": account_keys[instruction.accounts[1] as usize].to_string(),
            });
            let mut map = value.as_object_mut().unwrap();
            parse_signers(
                &mut map,
                2,
                account_keys,
                &instruction.accounts,
                "owner",
                "multisigOwner",
            );
            Ok(value)
        }
        TokenInstruction::MintTo { amount } => {
            if instruction.accounts.len() < 3 {
                return Err(ParseInstructionError::InstructionKeyMismatch(
                    ParsableProgram::SplToken,
                ));
            }
            let mut value = json!({
                "type": "mintTo",
                "mint": account_keys[instruction.accounts[0] as usize].to_string(),
                "account": account_keys[instruction.accounts[1] as usize].to_string(),
                "amount": amount,
            });
            let mut map = value.as_object_mut().unwrap();
            parse_signers(
                &mut map,
                2,
                account_keys,
                &instruction.accounts,
                "owner",
                "multisigOwner",
            );
            Ok(value)
        }
        TokenInstruction::Burn { amount } => {
            if instruction.accounts.len() < 2 {
                return Err(ParseInstructionError::InstructionKeyMismatch(
                    ParsableProgram::SplToken,
                ));
            }
            let mut value = json!({
                "type": "burn",
                "account": account_keys[instruction.accounts[0] as usize].to_string(),
                "amount": amount,
            });
            let mut map = value.as_object_mut().unwrap();
            parse_signers(
                &mut map,
                1,
                account_keys,
                &instruction.accounts,
                "authority",
                "multisigAuthority",
            );
            Ok(value)
        }
        TokenInstruction::CloseAccount => {
            if instruction.accounts.len() < 3 {
                return Err(ParseInstructionError::InstructionKeyMismatch(
                    ParsableProgram::SplToken,
                ));
            }
            let mut value = json!({
                "type": "closeAccount",
                "account": account_keys[instruction.accounts[0] as usize].to_string(),
                "destination": account_keys[instruction.accounts[1] as usize].to_string(),
            });
            let mut map = value.as_object_mut().unwrap();
            parse_signers(
                &mut map,
                2,
                account_keys,
                &instruction.accounts,
                "owner",
                "multisigOwner",
            );
            Ok(value)
        }
    }
}

fn parse_signers(
    map: &mut Map<String, Value>,
    last_nonsigner_index: usize,
    account_keys: &[Pubkey],
    accounts: &[u8],
    owner_field_name: &str,
    multisig_field_name: &str,
) {
    if accounts.len() > last_nonsigner_index + 1 {
        let mut signers: Vec<String> = vec![];
        for i in accounts[last_nonsigner_index + 1..].iter() {
            signers.push(account_keys[*i as usize].to_string());
        }
        map.insert(
            multisig_field_name.to_string(),
            json!(account_keys[accounts[last_nonsigner_index] as usize].to_string()),
        );
        map.insert("signers".to_string(), json!(signers));
    } else {
        map.insert(
            owner_field_name.to_string(),
            json!(account_keys[accounts[last_nonsigner_index] as usize].to_string()),
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_sdk::instruction::CompiledInstruction;
    use spl_token_v1_0::{
        instruction::*,
        solana_sdk::{
            instruction::CompiledInstruction as SplTokenCompiledInstruction, message::Message,
            pubkey::Pubkey as SplTokenPubkey,
        },
    };
    use std::str::FromStr;

    fn convert_pubkey(pubkey: Pubkey) -> SplTokenPubkey {
        SplTokenPubkey::from_str(&pubkey.to_string()).unwrap()
    }

    fn convert_compiled_instruction(
        instruction: &SplTokenCompiledInstruction,
    ) -> CompiledInstruction {
        CompiledInstruction {
            program_id_index: instruction.program_id_index,
            accounts: instruction.accounts.clone(),
            data: instruction.data.clone(),
        }
    }

    #[test]
    fn test_parse_token() {
        let mut keys: Vec<Pubkey> = vec![];
        for _ in 0..10 {
            keys.push(Pubkey::new_rand());
        }

        // Test InitializeMint variations
        let initialize_mint_ix = initialize_mint(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[0]),
            Some(&convert_pubkey(keys[1])),
            Some(&convert_pubkey(keys[2])),
            42,
            2,
        )
        .unwrap();
        let message = Message::new(&[initialize_mint_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "initializeMint",
                "mint": keys[0].to_string(),
                "amount": 42,
                "decimals": 2,
                "account": keys[1].to_string(),
                "owner": keys[2].to_string(),
            })
        );

        let initialize_mint_ix = initialize_mint(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[0]),
            Some(&convert_pubkey(keys[1])),
            None,
            42,
            2,
        )
        .unwrap();
        let message = Message::new(&[initialize_mint_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "initializeMint",
                "mint": keys[0].to_string(),
                "amount": 42,
                "decimals": 2,
                "account": keys[1].to_string(),
            })
        );

        let initialize_mint_ix = initialize_mint(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[0]),
            None,
            Some(&convert_pubkey(keys[1])),
            0,
            2,
        )
        .unwrap();
        let message = Message::new(&[initialize_mint_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "initializeMint",
                "mint": keys[0].to_string(),
                "amount": 0,
                "decimals": 2,
                "owner": keys[1].to_string(),
            })
        );

        // Test InitializeAccount
        let initialize_account_ix = initialize_account(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[0]),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[2]),
        )
        .unwrap();
        let message = Message::new(&[initialize_account_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "initializeAccount",
                "account": keys[0].to_string(),
                "mint": keys[1].to_string(),
                "owner": keys[2].to_string(),
            })
        );

        // Test InitializeMultisig
        let initialize_multisig_ix = initialize_multisig(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[0]),
            &[
                &convert_pubkey(keys[1]),
                &convert_pubkey(keys[2]),
                &convert_pubkey(keys[3]),
            ],
            2,
        )
        .unwrap();
        let message = Message::new(&[initialize_multisig_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "initializeMultisig",
                "multisig": keys[0].to_string(),
                "m": 2,
                "signers": keys[1..4].iter().map(|key| key.to_string()).collect::<Vec<String>>(),
            })
        );

        // Test Transfer, incl multisig
        let transfer_ix = transfer(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[0]),
            &[],
            42,
        )
        .unwrap();
        let message = Message::new(&[transfer_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "transfer",
                "source": keys[1].to_string(),
                "destination": keys[2].to_string(),
                "authority": keys[0].to_string(),
                "amount": 42,
            })
        );

        let transfer_ix = transfer(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[3]),
            &convert_pubkey(keys[4]),
            &[&convert_pubkey(keys[0]), &convert_pubkey(keys[1])],
            42,
        )
        .unwrap();
        let message = Message::new(&[transfer_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "transfer",
                "source": keys[2].to_string(),
                "destination": keys[3].to_string(),
                "multisigAuthority": keys[4].to_string(),
                "signers": keys[0..2].iter().map(|key| key.to_string()).collect::<Vec<String>>(),
                "amount": 42,
            })
        );

        // Test Approve, incl multisig
        let approve_ix = approve(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[0]),
            &[],
            42,
        )
        .unwrap();
        let message = Message::new(&[approve_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "approve",
                "source": keys[1].to_string(),
                "delegate": keys[2].to_string(),
                "owner": keys[0].to_string(),
                "amount": 42,
            })
        );

        let approve_ix = approve(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[3]),
            &convert_pubkey(keys[4]),
            &[&convert_pubkey(keys[0]), &convert_pubkey(keys[1])],
            42,
        )
        .unwrap();
        let message = Message::new(&[approve_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "approve",
                "source": keys[2].to_string(),
                "delegate": keys[3].to_string(),
                "multisigOwner": keys[4].to_string(),
                "signers": keys[0..2].iter().map(|key| key.to_string()).collect::<Vec<String>>(),
                "amount": 42,
            })
        );

        // Test Revoke
        let revoke_ix = revoke(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[0]),
            &[],
        )
        .unwrap();
        let message = Message::new(&[revoke_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "revoke",
                "source": keys[1].to_string(),
                "owner": keys[0].to_string(),
            })
        );

        // Test SetOwner
        let set_owner_ix = set_owner(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[0]),
            &[],
        )
        .unwrap();
        let message = Message::new(&[set_owner_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "setOwner",
                "owned": keys[1].to_string(),
                "newOwner": keys[2].to_string(),
                "owner": keys[0].to_string(),
            })
        );

        // Test MintTo
        let mint_to_ix = mint_to(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[0]),
            &[],
            42,
        )
        .unwrap();
        let message = Message::new(&[mint_to_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "mintTo",
                "mint": keys[1].to_string(),
                "account": keys[2].to_string(),
                "owner": keys[0].to_string(),
                "amount": 42,
            })
        );

        // Test Burn
        let burn_ix = burn(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[0]),
            &[],
            42,
        )
        .unwrap();
        let message = Message::new(&[burn_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "burn",
                "account": keys[1].to_string(),
                "authority": keys[0].to_string(),
                "amount": 42,
            })
        );

        // Test CloseAccount
        let close_account_ix = close_account(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[0]),
            &[],
        )
        .unwrap();
        let message = Message::new(&[close_account_ix], None);
        let compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert_eq!(
            parse_token(&compiled_instruction, &keys).unwrap(),
            json!({
                "type": "closeAccount",
                "account": keys[1].to_string(),
                "destination": keys[2].to_string(),
                "owner": keys[0].to_string(),
            })
        );
    }

    #[test]
    fn test_token_ix_not_enough_keys() {
        let mut keys: Vec<Pubkey> = vec![];
        for _ in 0..10 {
            keys.push(Pubkey::new_rand());
        }

        // Test InitializeMint variations
        let initialize_mint_ix = initialize_mint(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[0]),
            Some(&convert_pubkey(keys[1])),
            Some(&convert_pubkey(keys[2])),
            42,
            2,
        )
        .unwrap();
        let message = Message::new(&[initialize_mint_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..2]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 2].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        let initialize_mint_ix = initialize_mint(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[0]),
            Some(&convert_pubkey(keys[1])),
            None,
            42,
            2,
        )
        .unwrap();
        let message = Message::new(&[initialize_mint_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..1]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 1].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        let initialize_mint_ix = initialize_mint(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[0]),
            None,
            Some(&convert_pubkey(keys[1])),
            0,
            2,
        )
        .unwrap();
        let message = Message::new(&[initialize_mint_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..1]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 1].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        // Test InitializeAccount
        let initialize_account_ix = initialize_account(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[0]),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[2]),
        )
        .unwrap();
        let message = Message::new(&[initialize_account_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..2]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 1].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        // Test InitializeMultisig
        let initialize_multisig_ix = initialize_multisig(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[0]),
            &[
                &convert_pubkey(keys[1]),
                &convert_pubkey(keys[2]),
                &convert_pubkey(keys[3]),
            ],
            2,
        )
        .unwrap();
        let message = Message::new(&[initialize_multisig_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..3]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 3].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        // Test Transfer, incl multisig
        let transfer_ix = transfer(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[0]),
            &[],
            42,
        )
        .unwrap();
        let message = Message::new(&[transfer_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..2]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 1].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        let transfer_ix = transfer(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[3]),
            &convert_pubkey(keys[4]),
            &[&convert_pubkey(keys[0]), &convert_pubkey(keys[1])],
            42,
        )
        .unwrap();
        let message = Message::new(&[transfer_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..4]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 3].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        // Test Approve, incl multisig
        let approve_ix = approve(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[0]),
            &[],
            42,
        )
        .unwrap();
        let message = Message::new(&[approve_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..2]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 1].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        let approve_ix = approve(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[3]),
            &convert_pubkey(keys[4]),
            &[&convert_pubkey(keys[0]), &convert_pubkey(keys[1])],
            42,
        )
        .unwrap();
        let message = Message::new(&[approve_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..4]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 3].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        // Test Revoke
        let revoke_ix = revoke(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[0]),
            &[],
        )
        .unwrap();
        let message = Message::new(&[revoke_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..1]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 1].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        // Test SetOwner
        let set_owner_ix = set_owner(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[0]),
            &[],
        )
        .unwrap();
        let message = Message::new(&[set_owner_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..2]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 1].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        // Test MintTo
        let mint_to_ix = mint_to(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[0]),
            &[],
            42,
        )
        .unwrap();
        let message = Message::new(&[mint_to_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..2]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 1].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        // Test Burn
        let burn_ix = burn(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[0]),
            &[],
            42,
        )
        .unwrap();
        let message = Message::new(&[burn_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..1]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 1].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());

        // Test CloseAccount
        let close_account_ix = close_account(
            &spl_token_v1_0::id(),
            &convert_pubkey(keys[1]),
            &convert_pubkey(keys[2]),
            &convert_pubkey(keys[0]),
            &[],
        )
        .unwrap();
        let message = Message::new(&[close_account_ix], None);
        let mut compiled_instruction = convert_compiled_instruction(&message.instructions[0]);
        assert!(parse_token(&compiled_instruction, &keys[0..2]).is_err());
        compiled_instruction.accounts =
            compiled_instruction.accounts[0..compiled_instruction.accounts.len() - 1].to_vec();
        assert!(parse_token(&compiled_instruction, &keys).is_err());
    }
}
