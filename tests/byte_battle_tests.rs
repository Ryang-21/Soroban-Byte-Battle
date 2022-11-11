#![cfg(test)]
mod helper;
use helper::{create_byte_battle_contract, create_token_contract, generate_contract_id};
use soroban_auth::{testutils::ed25519, Identifier, Signature};

use soroban_sdk::{
    serde::Serialize,
    symbol,
    testutils::{Accounts, Ledger, LedgerInfo},
    BigInt, Env,
};
#[test]
fn test_initialize() {
    let e = Env::default();
    let token_contract_address = generate_contract_id(&e);
    let battle_bytes_contract_address = generate_contract_id(&e);
    let battle_bytes_client = create_byte_battle_contract(&e, &battle_bytes_contract_address);

    battle_bytes_client.initialize(&token_contract_address);
    assert_eq!(battle_bytes_client.token(), token_contract_address);
}

#[test]
#[should_panic(expected = "Error: Already initialized")]
fn test_initialize_already_initialized_expect_panic() {
    let e = Env::default();
    let token_contract_address = generate_contract_id(&e);
    let battle_bytes_contract_address = generate_contract_id(&e);
    let battle_bytes_client = create_byte_battle_contract(&e, &battle_bytes_contract_address);
    battle_bytes_client.initialize(&token_contract_address);
    battle_bytes_client.initialize(&token_contract_address);
}

#[test]
fn test_battle() {
    let e = Env::default();
    e.ledger().set(LedgerInfo {
        timestamp: 1,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });
    //Initialize token contract and bytebattle contract
    let token_admin = e.accounts().generate_and_create();
    let token_contract_address = generate_contract_id(&e);
    let token = create_token_contract(&e, &token_contract_address, &token_admin);

    let battle_bytes_contract_address = generate_contract_id(&e);
    let battle_bytes_id = Identifier::Contract(battle_bytes_contract_address.clone());
    let battle_bytes_client = create_byte_battle_contract(&e, &battle_bytes_contract_address);
    battle_bytes_client.initialize(&token_contract_address);

    let (player1_id, player1_sig) = ed25519::generate(&e);
    let (player2_id, player2_sig) = ed25519::generate(&e);

    let bet_amount = BigInt::from_u32(&e, 100);

    //Mint tokens to each player and approve for transfer
    token.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &player1_id,
        &bet_amount,
    );
    token.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &player2_id,
        &bet_amount,
    );
    let player1_token_nonce = token.nonce(&player1_id);
    let approval_sig = ed25519::sign(
        &e,
        &player1_sig,
        &token_contract_address,
        symbol!("approve"),
        (
            &player1_id,
            &player1_token_nonce,
            &battle_bytes_id,
            &bet_amount,
        ),
    );
    token.approve(
        &approval_sig,
        &player1_token_nonce,
        &battle_bytes_id,
        &bet_amount,
    );
    let player2_token_nonce = token.nonce(&player2_id);
    let approval_sig = ed25519::sign(
        &e,
        &player2_sig,
        &token_contract_address,
        symbol!("approve"),
        (
            &player2_id,
            &player2_token_nonce,
            &battle_bytes_id,
            &bet_amount,
        ),
    );
    token.approve(
        &approval_sig,
        &player2_token_nonce,
        &battle_bytes_id,
        &bet_amount,
    );

    //Generate signatures for each player
    let player1_nonce = battle_bytes_client.nonce(&player1_id);
    let player1_battle_sig = ed25519::sign(
        &e,
        &player1_sig,
        &battle_bytes_contract_address,
        symbol!("battle"),
        (&player1_id, &player1_nonce, &player2_id, &bet_amount),
    );
    let player2_nonce = battle_bytes_client.nonce(&player2_id);
    let player2_battle_sig = ed25519::sign(
        &e,
        &player2_sig,
        &battle_bytes_contract_address,
        symbol!("battle"),
        (&player2_id, &player2_nonce, &player1_id, &bet_amount),
    );
    let (player1_key, player2_key, player1_byte_to_compare, player2_byte_to_compare) =
        battle_bytes_client.battle(&player1_battle_sig, &player2_battle_sig, &bet_amount);

    print!(
        "Player 1: {:?}\nPlayer 2: {:?}\nPlayer 1 Byte Compared: {}\nPlayer 2 Byte Compared: {}",
        player1_key, player2_key, player1_byte_to_compare, player2_byte_to_compare
    );
    //The byte compared does not matter as both players are ed25519 identifiers
    assert_eq!(
        player1_byte_to_compare,
        (e.ledger().timestamp() % 32) as u32
    );
    if player1_key.get(player1_byte_to_compare.clone()) > player2_key.get(player1_byte_to_compare) {
        assert_eq!(token.balance(&player1_id), bet_amount * 2);
    } else {
        assert_eq!(token.balance(&player2_id), bet_amount * 2);
    }
}

#[test]
#[should_panic]
fn test_battle_different_bet_amount_expect_panic() {
    let e = Env::default();
    e.ledger().set(LedgerInfo {
        timestamp: 1,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });
    //Initialize token contract and bytebattle contract
    let token_admin = e.accounts().generate_and_create();
    let token_contract_address = generate_contract_id(&e);
    let token = create_token_contract(&e, &token_contract_address, &token_admin);

    let battle_bytes_contract_address = generate_contract_id(&e);
    let battle_bytes_id = Identifier::Contract(battle_bytes_contract_address.clone());
    let battle_bytes_client = create_byte_battle_contract(&e, &battle_bytes_contract_address);
    battle_bytes_client.initialize(&token_contract_address);

    let (player1_id, player1_sig) = ed25519::generate(&e);
    let (player2_id, player2_sig) = ed25519::generate(&e);

    let bet_amount = BigInt::from_u32(&e, 100);

    //Mint tokens to each player and approve for transfer
    token.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &player1_id,
        &bet_amount,
    );
    token.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &player2_id,
        &bet_amount,
    );
    let player1_token_nonce = token.nonce(&player1_id);
    let approval_sig = ed25519::sign(
        &e,
        &player1_sig,
        &token_contract_address,
        symbol!("approve"),
        (
            &player1_id,
            &player1_token_nonce,
            &battle_bytes_id,
            &bet_amount,
        ),
    );
    token.approve(
        &approval_sig,
        &player1_token_nonce,
        &battle_bytes_id,
        &bet_amount,
    );
    let player2_token_nonce = token.nonce(&player2_id);
    let approval_sig = ed25519::sign(
        &e,
        &player2_sig,
        &token_contract_address,
        symbol!("approve"),
        (
            &player2_id,
            &player2_token_nonce,
            &battle_bytes_id,
            &bet_amount,
        ),
    );
    token.approve(
        &approval_sig,
        &player2_token_nonce,
        &battle_bytes_id,
        &bet_amount,
    );

    //Generate signatures for each player
    let player1_nonce = battle_bytes_client.nonce(&player1_id);
    //Player 1 agreed to bet amount of 90
    let player1_battle_sig = ed25519::sign(
        &e,
        &player1_sig,
        &battle_bytes_contract_address,
        symbol!("battle"),
        (&player1_id, &player1_nonce, &player2_id, &bet_amount - 10),
    );

    let player2_nonce = battle_bytes_client.nonce(&player2_id);
    let player2_battle_sig = ed25519::sign(
        &e,
        &player2_sig,
        &battle_bytes_contract_address,
        symbol!("battle"),
        (&player2_id, &player2_nonce, &player1_id, &bet_amount),
    );

    battle_bytes_client.battle(&player1_battle_sig, &player2_battle_sig, &bet_amount);
}

#[test]
#[should_panic]
fn test_battle_different_user_expect_panic() {
    let e = Env::default();
    e.ledger().set(LedgerInfo {
        timestamp: 1,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });
    //Initialize token contract and bytebattle contract
    let token_admin = e.accounts().generate_and_create();
    let token_contract_address = generate_contract_id(&e);
    let token = create_token_contract(&e, &token_contract_address, &token_admin);

    let battle_bytes_contract_address = generate_contract_id(&e);
    let battle_bytes_id = Identifier::Contract(battle_bytes_contract_address.clone());
    let battle_bytes_client = create_byte_battle_contract(&e, &battle_bytes_contract_address);
    battle_bytes_client.initialize(&token_contract_address);

    let (player1_id, player1_sig) = ed25519::generate(&e);
    let (player2_id, player2_sig) = ed25519::generate(&e);

    let bet_amount = BigInt::from_u32(&e, 100);

    //Mint tokens to each player and approve for transfer
    token.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &player1_id,
        &bet_amount,
    );
    token.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &player2_id,
        &bet_amount,
    );
    let player1_token_nonce = token.nonce(&player1_id);
    let approval_sig = ed25519::sign(
        &e,
        &player1_sig,
        &token_contract_address,
        symbol!("approve"),
        (
            &player1_id,
            &player1_token_nonce,
            &battle_bytes_id,
            &bet_amount,
        ),
    );
    token.approve(
        &approval_sig,
        &player1_token_nonce,
        &battle_bytes_id,
        &bet_amount,
    );
    let player2_token_nonce = token.nonce(&player2_id);
    let approval_sig = ed25519::sign(
        &e,
        &player2_sig,
        &token_contract_address,
        symbol!("approve"),
        (
            &player2_id,
            &player2_token_nonce,
            &battle_bytes_id,
            &bet_amount,
        ),
    );
    token.approve(
        &approval_sig,
        &player2_token_nonce,
        &battle_bytes_id,
        &bet_amount,
    );

    //Generate signatures for each player
    let player1_nonce = battle_bytes_client.nonce(&player1_id);
    //Player 1 agreed to battle player 3;
    let (player3_id, _) = ed25519::generate(&e);
    let player1_battle_sig = ed25519::sign(
        &e,
        &player1_sig,
        &battle_bytes_contract_address,
        symbol!("battle"),
        (&player1_id, &player1_nonce, &player3_id, &bet_amount - 10),
    );

    let player2_nonce = battle_bytes_client.nonce(&player2_id);
    let player2_battle_sig = ed25519::sign(
        &e,
        &player2_sig,
        &battle_bytes_contract_address,
        symbol!("battle"),
        (&player2_id, &player2_nonce, &player1_id, &bet_amount),
    );

    battle_bytes_client.battle(&player1_battle_sig, &player2_battle_sig, &bet_amount);
}
#[test]
#[should_panic]
fn test_battle_reuse_sigs_expect_panic() {
    let e = Env::default();
    e.ledger().set(LedgerInfo {
        timestamp: 1,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });
    //Initialize token contract and bytebattle contract
    let token_admin = e.accounts().generate_and_create();
    let token_contract_address = generate_contract_id(&e);
    let token = create_token_contract(&e, &token_contract_address, &token_admin);

    let battle_bytes_contract_address = generate_contract_id(&e);
    let battle_bytes_id = Identifier::Contract(battle_bytes_contract_address.clone());
    let battle_bytes_client = create_byte_battle_contract(&e, &battle_bytes_contract_address);
    battle_bytes_client.initialize(&token_contract_address);

    let (player1_id, player1_sig) = ed25519::generate(&e);
    let (player2_id, player2_sig) = ed25519::generate(&e);

    let bet_amount = BigInt::from_u32(&e, 100);

    //Mint tokens to each player and approve for transfer
    token.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &player1_id,
        &bet_amount,
    );
    token.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &player2_id,
        &bet_amount,
    );
    let player1_token_nonce = token.nonce(&player1_id);
    let approval_sig = ed25519::sign(
        &e,
        &player1_sig,
        &token_contract_address,
        symbol!("approve"),
        (
            &player1_id,
            &player1_token_nonce,
            &battle_bytes_id,
            &bet_amount,
        ),
    );
    token.approve(
        &approval_sig,
        &player1_token_nonce,
        &battle_bytes_id,
        &bet_amount,
    );
    let player2_token_nonce = token.nonce(&player2_id);
    let approval_sig = ed25519::sign(
        &e,
        &player2_sig,
        &token_contract_address,
        symbol!("approve"),
        (
            &player2_id,
            &player2_token_nonce,
            &battle_bytes_id,
            &bet_amount,
        ),
    );
    token.approve(
        &approval_sig,
        &player2_token_nonce,
        &battle_bytes_id,
        &bet_amount,
    );

    //Generate signatures for each player
    let player1_nonce = battle_bytes_client.nonce(&player1_id);
    let player1_battle_sig = ed25519::sign(
        &e,
        &player1_sig,
        &battle_bytes_contract_address,
        symbol!("battle"),
        (&player1_id, &player1_nonce, &player2_id, &bet_amount),
    );

    let player2_nonce = battle_bytes_client.nonce(&player2_id);
    let player2_battle_sig = ed25519::sign(
        &e,
        &player2_sig,
        &battle_bytes_contract_address,
        symbol!("battle"),
        (&player2_id, &player2_nonce, &player1_id, &bet_amount),
    );

    battle_bytes_client.battle(&player1_battle_sig, &player2_battle_sig, &bet_amount);
    battle_bytes_client.battle(&player1_battle_sig, &player2_battle_sig, &bet_amount);
}

#[test]
fn test() {
    let e = Env::default();
    let account = e.accounts().generate_and_create();
    println!("{:?}", account.serialize(&e));
}
