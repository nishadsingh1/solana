use chrono::prelude::*;
use serde_json::Value;
use solana_cli::{
    cli::{process_command, request_and_confirm_airdrop, CliCommand, CliConfig, PayCommand},
    cli_output::OutputFormat,
    nonce,
    offline::{
        blockhash_query::{self, BlockhashQuery},
        parse_sign_only_reply_string,
    },
    spend_utils::SpendAmount,
    test_utils::check_recent_balance,
};
use solana_client::rpc_client::RpcClient;
use solana_core::validator::TestValidator;
use solana_faucet::faucet::run_local_faucet;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    nonce::State as NonceState,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::{fs::remove_dir_all, sync::mpsc::channel};

#[test]
fn test_cli_timestamp_tx() {
    let TestValidator {
        server,
        leader_data,
        alice,
        ledger_path,
        ..
    } = TestValidator::run();
    let bob_pubkey = Pubkey::new_rand();

    let (sender, receiver) = channel();
    run_local_faucet(alice, sender, None);
    let faucet_addr = receiver.recv().unwrap();

    let rpc_client = RpcClient::new_socket(leader_data.rpc);
    let default_signer0 = Keypair::new();
    let default_signer1 = Keypair::new();

    let mut config_payer = CliConfig::recent_for_tests();
    config_payer.json_rpc_url =
        format!("http://{}:{}", leader_data.rpc.ip(), leader_data.rpc.port());
    config_payer.signers = vec![&default_signer0];

    let mut config_witness = CliConfig::recent_for_tests();
    config_witness.json_rpc_url = config_payer.json_rpc_url.clone();
    config_witness.signers = vec![&default_signer1];

    assert_ne!(
        config_payer.signers[0].pubkey(),
        config_witness.signers[0].pubkey()
    );

    request_and_confirm_airdrop(
        &rpc_client,
        &faucet_addr,
        &config_payer.signers[0].pubkey(),
        50,
        &config_witness,
    )
    .unwrap();
    check_recent_balance(50, &rpc_client, &config_payer.signers[0].pubkey());

    request_and_confirm_airdrop(
        &rpc_client,
        &faucet_addr,
        &config_witness.signers[0].pubkey(),
        1,
        &config_witness,
    )
    .unwrap();

    // Make transaction (from config_payer to bob_pubkey) requiring timestamp from config_witness
    let date_string = "\"2018-09-19T17:30:59Z\"";
    let dt: DateTime<Utc> = serde_json::from_str(&date_string).unwrap();
    config_payer.command = CliCommand::Pay(PayCommand {
        amount: SpendAmount::Some(10),
        to: bob_pubkey,
        timestamp: Some(dt),
        timestamp_pubkey: Some(config_witness.signers[0].pubkey()),
        ..PayCommand::default()
    });
    let sig_response = process_command(&config_payer);

    let object: Value = serde_json::from_str(&sig_response.unwrap()).unwrap();
    let process_id_str = object.get("processId").unwrap().as_str().unwrap();
    let process_id_vec = bs58::decode(process_id_str)
        .into_vec()
        .expect("base58-encoded public key");
    let process_id = Pubkey::new(&process_id_vec);

    check_recent_balance(40, &rpc_client, &config_payer.signers[0].pubkey()); // config_payer balance
    check_recent_balance(10, &rpc_client, &process_id); // contract balance
    check_recent_balance(0, &rpc_client, &bob_pubkey); // recipient balance

    // Sign transaction by config_witness
    config_witness.command = CliCommand::TimeElapsed(bob_pubkey, process_id, dt);
    process_command(&config_witness).unwrap();

    check_recent_balance(40, &rpc_client, &config_payer.signers[0].pubkey()); // config_payer balance
    check_recent_balance(0, &rpc_client, &process_id); // contract balance
    check_recent_balance(10, &rpc_client, &bob_pubkey); // recipient balance

    server.close().unwrap();
    remove_dir_all(ledger_path).unwrap();
}

#[test]
fn test_cli_witness_tx() {
    let TestValidator {
        server,
        leader_data,
        alice,
        ledger_path,
        ..
    } = TestValidator::run();
    let bob_pubkey = Pubkey::new_rand();

    let (sender, receiver) = channel();
    run_local_faucet(alice, sender, None);
    let faucet_addr = receiver.recv().unwrap();

    let rpc_client = RpcClient::new_socket(leader_data.rpc);
    let default_signer0 = Keypair::new();
    let default_signer1 = Keypair::new();

    let mut config_payer = CliConfig::recent_for_tests();
    config_payer.json_rpc_url =
        format!("http://{}:{}", leader_data.rpc.ip(), leader_data.rpc.port());
    config_payer.signers = vec![&default_signer0];

    let mut config_witness = CliConfig::recent_for_tests();
    config_witness.json_rpc_url = config_payer.json_rpc_url.clone();
    config_witness.signers = vec![&default_signer1];

    assert_ne!(
        config_payer.signers[0].pubkey(),
        config_witness.signers[0].pubkey()
    );

    request_and_confirm_airdrop(
        &rpc_client,
        &faucet_addr,
        &config_payer.signers[0].pubkey(),
        50,
        &config_witness,
    )
    .unwrap();
    request_and_confirm_airdrop(
        &rpc_client,
        &faucet_addr,
        &config_witness.signers[0].pubkey(),
        1,
        &config_witness,
    )
    .unwrap();

    // Make transaction (from config_payer to bob_pubkey) requiring witness signature from config_witness
    config_payer.command = CliCommand::Pay(PayCommand {
        amount: SpendAmount::Some(10),
        to: bob_pubkey,
        witnesses: Some(vec![config_witness.signers[0].pubkey()]),
        ..PayCommand::default()
    });
    let sig_response = process_command(&config_payer);

    let object: Value = serde_json::from_str(&sig_response.unwrap()).unwrap();
    let process_id_str = object.get("processId").unwrap().as_str().unwrap();
    let process_id_vec = bs58::decode(process_id_str)
        .into_vec()
        .expect("base58-encoded public key");
    let process_id = Pubkey::new(&process_id_vec);

    check_recent_balance(40, &rpc_client, &config_payer.signers[0].pubkey()); // config_payer balance
    check_recent_balance(10, &rpc_client, &process_id); // contract balance
    check_recent_balance(0, &rpc_client, &bob_pubkey); // recipient balance

    // Sign transaction by config_witness
    config_witness.command = CliCommand::Witness(bob_pubkey, process_id);
    process_command(&config_witness).unwrap();

    check_recent_balance(40, &rpc_client, &config_payer.signers[0].pubkey()); // config_payer balance
    check_recent_balance(0, &rpc_client, &process_id); // contract balance
    check_recent_balance(10, &rpc_client, &bob_pubkey); // recipient balance

    server.close().unwrap();
    remove_dir_all(ledger_path).unwrap();
}

#[test]
fn test_cli_cancel_tx() {
    let TestValidator {
        server,
        leader_data,
        alice,
        ledger_path,
        ..
    } = TestValidator::run();
    let bob_pubkey = Pubkey::new_rand();

    let (sender, receiver) = channel();
    run_local_faucet(alice, sender, None);
    let faucet_addr = receiver.recv().unwrap();

    let rpc_client = RpcClient::new_socket(leader_data.rpc);
    let default_signer0 = Keypair::new();
    let default_signer1 = Keypair::new();

    let mut config_payer = CliConfig::recent_for_tests();
    config_payer.json_rpc_url =
        format!("http://{}:{}", leader_data.rpc.ip(), leader_data.rpc.port());
    config_payer.signers = vec![&default_signer0];

    let mut config_witness = CliConfig::recent_for_tests();
    config_witness.json_rpc_url = config_payer.json_rpc_url.clone();
    config_witness.signers = vec![&default_signer1];

    assert_ne!(
        config_payer.signers[0].pubkey(),
        config_witness.signers[0].pubkey()
    );

    request_and_confirm_airdrop(
        &rpc_client,
        &faucet_addr,
        &config_payer.signers[0].pubkey(),
        50,
        &config_witness,
    )
    .unwrap();

    // Make transaction (from config_payer to bob_pubkey) requiring witness signature from config_witness
    config_payer.command = CliCommand::Pay(PayCommand {
        amount: SpendAmount::Some(10),
        to: bob_pubkey,
        witnesses: Some(vec![config_witness.signers[0].pubkey()]),
        cancelable: true,
        ..PayCommand::default()
    });
    let sig_response = process_command(&config_payer).unwrap();

    let object: Value = serde_json::from_str(&sig_response).unwrap();
    let process_id_str = object.get("processId").unwrap().as_str().unwrap();
    let process_id_vec = bs58::decode(process_id_str)
        .into_vec()
        .expect("base58-encoded public key");
    let process_id = Pubkey::new(&process_id_vec);

    check_recent_balance(40, &rpc_client, &config_payer.signers[0].pubkey()); // config_payer balance
    check_recent_balance(10, &rpc_client, &process_id); // contract balance
    check_recent_balance(0, &rpc_client, &bob_pubkey); // recipient balance

    // Sign transaction by config_witness
    config_payer.command = CliCommand::Cancel(process_id);
    process_command(&config_payer).unwrap();

    check_recent_balance(50, &rpc_client, &config_payer.signers[0].pubkey()); // config_payer balance
    check_recent_balance(0, &rpc_client, &process_id); // contract balance
    check_recent_balance(0, &rpc_client, &bob_pubkey); // recipient balance

    server.close().unwrap();
    remove_dir_all(ledger_path).unwrap();
}

#[test]
fn test_offline_pay_tx() {
    let TestValidator {
        server,
        leader_data,
        alice,
        ledger_path,
        ..
    } = TestValidator::run();
    let bob_pubkey = Pubkey::new_rand();

    let (sender, receiver) = channel();
    run_local_faucet(alice, sender, None);
    let faucet_addr = receiver.recv().unwrap();

    let rpc_client = RpcClient::new_socket(leader_data.rpc);
    let default_signer = Keypair::new();
    let default_offline_signer = Keypair::new();

    let mut config_offline = CliConfig::recent_for_tests();
    config_offline.json_rpc_url =
        format!("http://{}:{}", leader_data.rpc.ip(), leader_data.rpc.port());
    config_offline.signers = vec![&default_offline_signer];
    let mut config_online = CliConfig::recent_for_tests();
    config_online.json_rpc_url =
        format!("http://{}:{}", leader_data.rpc.ip(), leader_data.rpc.port());
    config_online.signers = vec![&default_signer];
    assert_ne!(
        config_offline.signers[0].pubkey(),
        config_online.signers[0].pubkey()
    );

    request_and_confirm_airdrop(
        &rpc_client,
        &faucet_addr,
        &config_offline.signers[0].pubkey(),
        50,
        &config_offline,
    )
    .unwrap();

    request_and_confirm_airdrop(
        &rpc_client,
        &faucet_addr,
        &config_online.signers[0].pubkey(),
        50,
        &config_offline,
    )
    .unwrap();
    check_recent_balance(50, &rpc_client, &config_offline.signers[0].pubkey());
    check_recent_balance(50, &rpc_client, &config_online.signers[0].pubkey());

    let (blockhash, _) = rpc_client.get_recent_blockhash().unwrap();
    config_offline.command = CliCommand::Pay(PayCommand {
        amount: SpendAmount::Some(10),
        to: bob_pubkey,
        blockhash_query: BlockhashQuery::None(blockhash),
        sign_only: true,
        ..PayCommand::default()
    });
    config_offline.output_format = OutputFormat::JsonCompact;
    let sig_response = process_command(&config_offline).unwrap();

    check_recent_balance(50, &rpc_client, &config_offline.signers[0].pubkey());
    check_recent_balance(50, &rpc_client, &config_online.signers[0].pubkey());
    check_recent_balance(0, &rpc_client, &bob_pubkey);

    let sign_only = parse_sign_only_reply_string(&sig_response);
    assert!(sign_only.has_all_signers());
    let offline_presigner = sign_only
        .presigner_of(&config_offline.signers[0].pubkey())
        .unwrap();
    let online_pubkey = config_online.signers[0].pubkey();
    config_online.signers = vec![&offline_presigner];
    config_online.command = CliCommand::Pay(PayCommand {
        amount: SpendAmount::Some(10),
        to: bob_pubkey,
        blockhash_query: BlockhashQuery::FeeCalculator(blockhash_query::Source::Cluster, blockhash),
        ..PayCommand::default()
    });
    process_command(&config_online).unwrap();

    check_recent_balance(40, &rpc_client, &config_offline.signers[0].pubkey());
    check_recent_balance(50, &rpc_client, &online_pubkey);
    check_recent_balance(10, &rpc_client, &bob_pubkey);

    server.close().unwrap();
    remove_dir_all(ledger_path).unwrap();
}

#[test]
fn test_nonced_pay_tx() {
    solana_logger::setup();

    let TestValidator {
        server,
        leader_data,
        alice,
        ledger_path,
        ..
    } = TestValidator::run();
    let (sender, receiver) = channel();
    run_local_faucet(alice, sender, None);
    let faucet_addr = receiver.recv().unwrap();

    let rpc_client = RpcClient::new_socket(leader_data.rpc);
    let default_signer = Keypair::new();

    let mut config = CliConfig::recent_for_tests();
    config.json_rpc_url = format!("http://{}:{}", leader_data.rpc.ip(), leader_data.rpc.port());
    config.signers = vec![&default_signer];

    let minimum_nonce_balance = rpc_client
        .get_minimum_balance_for_rent_exemption(NonceState::size())
        .unwrap();

    request_and_confirm_airdrop(
        &rpc_client,
        &faucet_addr,
        &config.signers[0].pubkey(),
        50 + minimum_nonce_balance,
        &config,
    )
    .unwrap();
    check_recent_balance(
        50 + minimum_nonce_balance,
        &rpc_client,
        &config.signers[0].pubkey(),
    );

    // Create nonce account
    let nonce_account = Keypair::new();
    config.command = CliCommand::CreateNonceAccount {
        nonce_account: 1,
        seed: None,
        nonce_authority: Some(config.signers[0].pubkey()),
        amount: SpendAmount::Some(minimum_nonce_balance),
    };
    config.signers.push(&nonce_account);
    process_command(&config).unwrap();

    check_recent_balance(50, &rpc_client, &config.signers[0].pubkey());
    check_recent_balance(minimum_nonce_balance, &rpc_client, &nonce_account.pubkey());

    // Fetch nonce hash
    let nonce_hash = nonce::get_account_with_commitment(
        &rpc_client,
        &nonce_account.pubkey(),
        CommitmentConfig::recent(),
    )
    .and_then(|ref a| nonce::data_from_account(a))
    .unwrap()
    .blockhash;

    let bob_pubkey = Pubkey::new_rand();
    config.signers = vec![&default_signer];
    config.command = CliCommand::Pay(PayCommand {
        amount: SpendAmount::Some(10),
        to: bob_pubkey,
        blockhash_query: BlockhashQuery::FeeCalculator(
            blockhash_query::Source::NonceAccount(nonce_account.pubkey()),
            nonce_hash,
        ),
        nonce_account: Some(nonce_account.pubkey()),
        ..PayCommand::default()
    });
    process_command(&config).expect("failed to process pay command");

    check_recent_balance(40, &rpc_client, &config.signers[0].pubkey());
    check_recent_balance(10, &rpc_client, &bob_pubkey);

    // Verify that nonce has been used
    let nonce_hash2 = nonce::get_account_with_commitment(
        &rpc_client,
        &nonce_account.pubkey(),
        CommitmentConfig::recent(),
    )
    .and_then(|ref a| nonce::data_from_account(a))
    .unwrap()
    .blockhash;
    assert_ne!(nonce_hash, nonce_hash2);

    server.close().unwrap();
    remove_dir_all(ledger_path).unwrap();
}
