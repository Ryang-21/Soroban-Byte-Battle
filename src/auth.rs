use crate::DataKey;
use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contracterror, panic_with_error, BigInt, Env};

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    IncorrectNonceForInvoker = 1,
    IncorrectNonce = 2,
}

pub fn verify_and_consume_nonce(env: &Env, sig: &Signature, nonce: &i64) {
    match sig {
        Signature::Invoker => {
            if BigInt::zero(env) != nonce {
                panic_with_error!(env, Error::IncorrectNonceForInvoker);
            }
        }
        Signature::Ed25519(_) | Signature::Account(_) => {
            let id = sig.identifier(env);
            if nonce != &get_nonce(env, &id) {
                panic_with_error!(env, Error::IncorrectNonce);
            }
            set_nonce(env, &id, nonce + 1);
        }
    }
}

pub fn get_nonce(env: &Env, id: &Identifier) -> i64 {
    let key = DataKey::Nonce(id.clone());
    env.data()
        .get::<DataKey, i64>(key)
        .unwrap_or_else(|| Ok(0))
        .unwrap()
}

pub fn set_nonce(env: &Env, id: &Identifier, nonce: i64) {
    let key = DataKey::Nonce(id.clone());
    env.data().set(key, nonce);
}
