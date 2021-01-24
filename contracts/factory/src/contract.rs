use cosmwasm_std::{Api, Binary, Env, Extern, HandleResponse, HandleResult, InitResponse, Querier, QueryResult, StdError, StdResult, Storage, to_binary};

use crate::{msg::{HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg}, rand::sha_256};
use crate::state::{save, load, may_load};
use crate::viewing_key::{ViewingKey, VIEWING_KEY_SIZE};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
 
/// prefix for viewing keys
pub const PREFIX_VIEW_KEY: &[u8] = b"viewingkey";
/// storage key for prng seed
pub const PRNG_SEED_KEY: &[u8] = b"prngseed";
/// storage key for the factory admin
pub const ADMIN_KEY: &[u8] = b"admin";
/// storage key for the children contracts 
pub const SECRET_ORDER_BOOK_CONTRACT_CODE_ID: &[u8] = b"secretorderbookcontractcodeid";
/// storage key for the children contracts 
pub const SECRET_ORDER_BOOK_CONTRACT_CODE_HASH: &[u8] = b"secretorderbookcontractcodehash";
/// storage key for the factory admin
pub const FACTORY_KEY: &[u8] = b"factorykey";
/// response size
pub const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let prng_seed: Vec<u8> = sha_256(base64::encode(msg.entropy.clone()).as_bytes()).to_vec();
    let key = ViewingKey::new(&env, &prng_seed, msg.entropy.clone().as_ref());
    save(&mut deps.storage, FACTORY_KEY, &format!("{}", key))?;
    save(&mut deps.storage, PRNG_SEED_KEY, &prng_seed)?;
    save(&mut deps.storage, ADMIN_KEY, &env.message.sender)?;
    save(&mut deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_ID, &msg.secret_order_book_code_id)?;
    save(&mut deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_HASH, &msg.secret_order_book_code_hash)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::CreateViewingKey { entropy } => try_create_key(deps, env, &entropy),
        //HandleMsg::ChangeArenaContractCodeId { code_id, code_hash } => try_change_arena_contract_code_id(deps, env, &code_id, &code_hash),
        //HandleMsg::NewArenaInstanciate {name, entropy} => try_arena_instanciate(deps, env, &name, &entropy),
        //HandleMsg::InitCallBackFromArenaToFactory {auth_key, contract_address} => try_arena_instanciated_callback(deps, env, auth_key, contract_address)
    }
}

fn try_create_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: &str,
) -> HandleResult {
    // create and store the key
    let prng_seed: Vec<u8> = load(&deps.storage, PRNG_SEED_KEY)?;
    let key = ViewingKey::new(&env, &prng_seed, entropy.as_ref());
    let message_sender = &deps.api.canonical_address(&env.message.sender)?;
    let mut key_store = PrefixedStorage::new(PREFIX_VIEW_KEY, &mut deps.storage);
    save(&mut key_store, message_sender.as_slice(), &key.to_hashed())?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ViewingKey {
            key: format!("{}", key),
        })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        /*QueryMsg::IsKeyValid {
            address,
            viewing_key,
            factory_key
        } => try_validate_key(deps, &address, viewing_key, factory_key),
        */
        QueryMsg::ArenaContractCodeId {} => secret_order_book_contract_code_id(deps),
        //QueryMsg::Arenas {} => arenas(deps)
    }
}

fn secret_order_book_contract_code_id <S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> QueryResult {
    let arena_contract_code_id: u64 = load(&deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_ID)?;
    let arena_contract_code_hash: String = load(&deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_HASH)?;
    to_binary(&QueryAnswer::ArenaContractCodeID {
        code_id: arena_contract_code_id,
        code_hash: arena_contract_code_hash
    })
}