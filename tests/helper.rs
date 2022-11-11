use rand::{thread_rng, RngCore};
use soroban_auth::Identifier;
use soroban_byte_battle::{token, ByteBattle, ByteBattleClient};
use soroban_sdk::{AccountId, BytesN, Env, IntoVal};

pub fn generate_contract_id(e: &Env) -> BytesN<32> {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id);
    BytesN::from_array(e, &id)
}

pub fn create_token_contract(
    e: &Env,
    contract_id: &BytesN<32>,
    admin: &AccountId,
) -> token::Client {
    e.register_contract_token(contract_id);

    let token = token::Client::new(e, contract_id);
    token.init(
        &Identifier::Account(admin.clone()),
        &token::TokenMetadata {
            name: "unit".into_val(e),
            symbol: "test".into_val(e),
            decimals: 7,
        },
    );
    token
}

pub fn create_byte_battle_contract(e: &Env, contract_id: &BytesN<32>) -> ByteBattleClient {
    e.register_contract(contract_id, ByteBattle {});
    return ByteBattleClient::new(e, contract_id);
}
