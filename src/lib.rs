#![no_std]
#[cfg(any(test, feature = "testutils"))]
extern crate std;

use auth::{get_nonce, verify_and_consume_nonce};
use soroban_auth::{verify, Identifier, Signature};
use soroban_sdk::{
    contractimpl, contracttype, serde::Serialize, symbol, BigInt, Bytes, BytesN, Env,
};
mod auth;
pub mod token {
    soroban_sdk::contractimport!(file = "token/soroban_token_spec.wasm");
}
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Nonce(Identifier),
    Token,
}
pub struct ByteBattle;

pub trait ByteBattleTraits {
    fn initialize(e: Env, token_address: BytesN<32>);
    fn battle(
        e: Env,
        player1_sig: Signature,
        player2_sig: Signature,
        amount: BigInt,
    ) -> (Bytes, Bytes, u32, u32);
    fn nonce(e: Env, user: Identifier) -> i64;
    fn token(e: Env) -> BytesN<32>;
}
#[contractimpl]
impl ByteBattleTraits for ByteBattle {
    fn initialize(e: Env, token_address: BytesN<32>) {
        if e.data().has(DataKey::Token) {
            panic!("Error: Already initialized");
        }
        e.data().set(DataKey::Token, token_address);
    }

    fn battle(
        e: Env,
        player1_sig: Signature,
        player2_sig: Signature,
        amount: BigInt,
    ) -> (Bytes, Bytes, u32, u32) {
        let signer1_id = player1_sig.identifier(&e);
        let signer2_id = player2_sig.identifier(&e);
        let p1_nonce = get_nonce(&e, &signer1_id);
        let p2_nonce = get_nonce(&e, &signer2_id);

        verify(
            &e,
            &player1_sig,
            symbol!("battle"),
            (&signer1_id, &p1_nonce, &signer2_id, &amount),
        );
        verify_and_consume_nonce(&e, &player1_sig, &p1_nonce);
        verify(
            &e,
            &player2_sig,
            symbol!("battle"),
            (&signer2_id, &p2_nonce, &signer1_id, &amount),
        );
        verify_and_consume_nonce(&e, &player2_sig, &p2_nonce);

        let token = token::Client::new(&e, &get_token(&e));
        token.xfer_from(
            &Signature::Invoker,
            &BigInt::zero(&e),
            &signer1_id,
            &Identifier::Contract(e.current_contract()),
            &amount,
        );
        token.xfer_from(
            &Signature::Invoker,
            &BigInt::zero(&e),
            &signer2_id,
            &Identifier::Contract(e.current_contract()),
            &amount,
        );

        let mut player1_byte_to_compare = (e.ledger().timestamp() % 32) as u32;
        let mut player2_byte_to_compare = (e.ledger().timestamp() % 32) as u32;
        let player1_key: Bytes = match signer1_id.clone() {
            Identifier::Account(a) => {
                player1_byte_to_compare += 16;
                a.serialize(&e)
            }
            Identifier::Contract(k) => Bytes::from_array(&e, &k.to_array()),
            Identifier::Ed25519(k) => Bytes::from_array(&e, &k.to_array()),
        };
        let player2_key: Bytes = match signer1_id.clone() {
            Identifier::Account(a) => {
                player2_byte_to_compare += 16;
                a.serialize(&e)
            }
            Identifier::Contract(k) => Bytes::from_array(&e, &k.to_array()),
            Identifier::Ed25519(k) => Bytes::from_array(&e, &k.to_array()),
        };

        if player1_key.get(player1_byte_to_compare).unwrap()
            > player2_key.get(player2_byte_to_compare).unwrap()
        {
            token.xfer(
                &Signature::Invoker,
                &BigInt::zero(&e),
                &signer1_id,
                &(amount * 2),
            );
        } else {
            token.xfer(
                &Signature::Invoker,
                &BigInt::zero(&e),
                &signer2_id,
                &(amount * 2),
            );
        }
        return (
            player1_key,
            player2_key,
            player1_byte_to_compare,
            player2_byte_to_compare,
        );
    }

    fn nonce(e: Env, user: Identifier) -> i64 {
        get_nonce(&e, &user)
    }
    fn token(e: Env) -> BytesN<32> {
        get_token(&e)
    }
}
/*--------Helpers--------*/
pub fn get_token(e: &Env) -> BytesN<32> {
    e.data().get(DataKey::Token).unwrap().unwrap()
}
// fn add_win(user: Identifier) {}
