use cosmwasm_std::{Api, Binary, CanonicalAddr, Env, Extern, HandleResponse, HandleResult, HumanAddr, InitResponse, Querier, QueryResult, ReadonlyStorage, StdError, StdResult, Storage, to_binary};

use crate::{msg::{AssetInfo, HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg, ResponseStatus::Success, SecretOrderBookContractInitMsg}, rand::sha_256};
use crate::state::{save, load, may_load};
use crate::viewing_key::{ViewingKey, VIEWING_KEY_SIZE};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
 
use secret_toolkit::{snip20::token_info_query, utils::{InitCallback}};

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
/// storage key for the secret order books
pub const PREFIX_TOKEN_SECRET_ORDER_BOOKS: &[u8] = b"tokensecretorderbooks";
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
        HandleMsg::ChangeSecretOrderBookContractCodeId { code_id, code_hash } => try_change_secret_order_book_contract_code_id(deps, env, &code_id, &code_hash),
        HandleMsg::NewSecretOrderBookInstanciate {
            token1_info,
            token2_info
        } => try_secret_order_book_instanciate(deps, env, &token1_info, &token2_info),
        HandleMsg::InitCallBackFromSecretOrderBookToFactory {
            auth_key, 
            contract_address,  
            token1_info, 
            token2_info
        } => try_secret_order_book_instanciated_callback(deps, env, auth_key, contract_address, token1_info, token2_info)
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

fn try_change_secret_order_book_contract_code_id<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    code_id: &u64,
    code_hash: &String
) -> HandleResult {
    let admin: HumanAddr = load(&deps.storage, ADMIN_KEY)?;
    if env.message.sender != admin {
        return Err(StdError::generic_err(
            "Permission Denied.",
        ));
    }
    
    save(&mut deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_ID, &code_id)?;
    save(&mut deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_HASH, &code_hash)?;
    
    Ok(HandleResponse::default())
}

fn try_secret_order_book_instanciate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    token1_info: &AssetInfo,
    token2_info: &AssetInfo
) -> HandleResult {  
    let secret_order_book_contract_code_id: u64 = load(&deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_ID)?;
    let secret_order_book_contract_code_hash: String = load(&deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_HASH)?;
    let factory_key: String = load(&deps.storage, FACTORY_KEY)?;
    
    let initmsg = SecretOrderBookContractInitMsg {
        factory_hash: env.contract_code_hash,
        factory_address: env.contract.address,
        factory_key,
        token1_info: token1_info.clone(),
        token2_info: token2_info.clone()
    };
    impl InitCallback for SecretOrderBookContractInitMsg {
        const BLOCK_SIZE: usize = BLOCK_SIZE;
    }

    let token1_symbol:String;
    let token2_symbol:String;
    //query tokens info and get symbols from Addresses
    match token1_info {
        AssetInfo::NativeToken { .. } => token1_symbol="SCRT".to_string(),
        AssetInfo::Token { contract_addr, token_code_hash } => {
            let response_token1 = token_info_query(&deps.querier,BLOCK_SIZE,token_code_hash.to_owned(), contract_addr.to_owned());
            token1_symbol = response_token1.unwrap().symbol;
        }
    }
    match token2_info {
        AssetInfo::NativeToken { .. } => token2_symbol="SCRT".to_string(),
        AssetInfo::Token { contract_addr, token_code_hash } => {
            let response_token2 = token_info_query(&deps.querier,BLOCK_SIZE,token_code_hash.to_owned(), contract_addr.to_owned());
            token2_symbol = response_token2.unwrap().symbol;
        }
    }
    //TODO: Deal with duplicated token symbols
    let cosmosmsg =
        initmsg.to_cosmos_msg(format!("({}) Secret Order Book - {}/{}",secret_order_book_contract_code_id,token1_symbol,token2_symbol).to_string(), secret_order_book_contract_code_id, secret_order_book_contract_code_hash, None)?;

    Ok(HandleResponse {
        messages: vec![cosmosmsg],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Status {
            status: Success,
            message: None,
        })?),
    })
}

pub fn try_secret_order_book_instanciated_callback<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    auth_key: String,
    contract_address: HumanAddr,
    token1_info: AssetInfo,
    token2_info: AssetInfo,
) -> HandleResult {   
    let factory_key: String = load(&deps.storage, FACTORY_KEY)?;
    let input_key: String = auth_key;
    
    if factory_key != input_key {
        return Err(StdError::generic_err(
            "Permission Denied.",
        ));
    }

    let token1_address_raw: CanonicalAddr;
    let token2_address_raw: CanonicalAddr;

    match token1_info {
        AssetInfo::NativeToken { .. } => token1_address_raw = deps.api.canonical_address(&HumanAddr("scrt".to_string()))?,
        AssetInfo::Token { contract_addr, .. } => {token1_address_raw = deps.api.canonical_address(&contract_addr)?}
    }
    match token2_info {
        AssetInfo::NativeToken { .. } => token2_address_raw = deps.api.canonical_address(&HumanAddr("scrt".to_string()))?,
        AssetInfo::Token { contract_addr, .. } => {token2_address_raw = deps.api.canonical_address(&contract_addr)?}
    }

    let mut token_secret_order_books = PrefixedStorage::new(PREFIX_TOKEN_SECRET_ORDER_BOOKS, &mut deps.storage);
    let load_token1_secret_order_books: Option<Vec<HumanAddr>> = may_load(&token_secret_order_books, token1_address_raw.as_slice())?;
    let load_token2_secret_order_books: Option<Vec<HumanAddr>> = may_load(&token_secret_order_books, token2_address_raw.as_slice())?;

    let mut token1_secret_order_books = load_token1_secret_order_books.unwrap_or_default();
    let mut token2_secret_order_books = load_token2_secret_order_books.unwrap_or_default();

    token1_secret_order_books.insert(0,contract_address.clone());
    token2_secret_order_books.insert(0,contract_address.clone());

    save(&mut token_secret_order_books, token1_address_raw.as_slice(), &token1_secret_order_books)?;
    save(&mut token_secret_order_books, token2_address_raw.as_slice(), &token2_secret_order_books)?;

    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsKeyValid {
            address,
            viewing_key,
            factory_key
        } => try_validate_key(deps, &address, viewing_key, factory_key),
        QueryMsg::SecretOrderBookContractCodeId {} => secret_order_book_contract_code_id(deps),
        QueryMsg::SecretOrderBooks {token_address} => secret_order_books(deps, token_address)
    }
}

fn try_validate_key<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
    viewing_key: String,
    factory_key: String
) -> QueryResult {
    let addr_raw = &deps.api.canonical_address(address)?;
    let state_factory_key: String = load(&deps.storage, FACTORY_KEY)?;
    if factory_key != state_factory_key {
        return Err(StdError::generic_err(
            "Permission Denied.",
        ));
    }

    to_binary(&QueryAnswer::IsKeyValid {
        is_valid: is_key_valid(&deps.storage, addr_raw, viewing_key)?,
    })
}

fn is_key_valid<S: ReadonlyStorage>(
    storage: &S,
    address: &CanonicalAddr,
    viewing_key: String,
) -> StdResult<bool> {
    // load the address' key
    let read_key = ReadonlyPrefixedStorage::new(PREFIX_VIEW_KEY, storage);
    let load_key: Option<[u8; VIEWING_KEY_SIZE]> = may_load(&read_key, address.as_slice())?;
    let input_key = ViewingKey(viewing_key);
    // if a key was set
    if let Some(expected_key) = load_key {
        // and it matches
        if input_key.check_viewing_key(&expected_key) {
            return Ok(true);
        }
    } else {
        // Checking the key will take significant time. We don't want to exit immediately if it isn't set
        // in a way which will allow to time the command and determine if a viewing key doesn't exist
        input_key.check_viewing_key(&[0u8; VIEWING_KEY_SIZE]);
    }
    Ok(false)
}

fn secret_order_book_contract_code_id <S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> QueryResult {
    let secret_order_book_contract_code_id: u64 = load(&deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_ID)?;
    let secret_order_book_contract_code_hash: String = load(&deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_HASH)?;
    to_binary(&QueryAnswer::SecretOrderBookContractCodeID {
        code_id: secret_order_book_contract_code_id,
        code_hash: secret_order_book_contract_code_hash
    })
}

fn secret_order_books <S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token_address: HumanAddr
) -> QueryResult {
    let token_address_raw = &deps.api.canonical_address(&token_address)?;

    let token_secret_order_books = ReadonlyPrefixedStorage::new(PREFIX_TOKEN_SECRET_ORDER_BOOKS, &deps.storage);
    let load_token_secret_order_books: Option<Vec<HumanAddr>> = may_load(&token_secret_order_books, token_address_raw.as_slice())?;

    to_binary(&QueryAnswer::SecretOrderBooks {
        secret_order_books: load_token_secret_order_books.unwrap_or_default()
    })
}