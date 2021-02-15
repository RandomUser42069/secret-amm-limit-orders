use std::u128;

use cosmwasm_std::{Api, Binary, CanonicalAddr, Env, Extern, HandleResponse, HandleResult, HumanAddr, InitResponse, Querier, QueryResult, ReadonlyStorage, StdError, StdResult, Storage, Uint128, to_binary};

use crate::{msg::{AmmAssetInfo, AmmPairResponse, AmmQueryMsg, AssetInfo, HandleAnswer, HandleMsg, InitMsg, NativeToken, QueryAnswer, QueryMsg, ResponseStatus::Success, SecretOrderBookContract, SecretOrderBookContractInitMsg, Token}, rand::sha_256};
use crate::state::{save, load, may_load};
use crate::viewing_key::{ViewingKey, VIEWING_KEY_SIZE};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
 
use secret_toolkit::{snip20::token_info_query, utils::{InitCallback, Query}};

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
pub const PREFIX_SECRET_ORDER_BOOKS: &[u8] = b"secretorderbooks";
/// storage key for the amm factory address
pub const AMM_FACTORY_ADDRESS: &[u8] = b"ammfactoryaddress";
/// storage key for the children contracts 
pub const AMM_FACTORY_HASH: &[u8] = b"ammfactoryhash";
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
    save(&mut deps.storage, AMM_FACTORY_ADDRESS, &msg.amm_factory_contract_address)?;
    save(&mut deps.storage, AMM_FACTORY_HASH, &msg.amm_factory_contract_hash)?;
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
            amm_pair_address,
            amm_pair_hash
        } => try_secret_order_book_instanciate(deps, env, &amm_pair_address, &amm_pair_hash),
        HandleMsg::InitCallBackFromSecretOrderBookToFactory {
            auth_key, 
            amm_pair_address,
            contract_address,  
            token1_info, 
            token2_info
        } => try_secret_order_book_instanciated_callback(deps, env, auth_key, amm_pair_address, contract_address, token1_info, token2_info)
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
    amm_pair_address: &HumanAddr,
    amm_pair_hash: &String
) -> HandleResult {  
    let secret_order_book_contract_code_id: u64 = load(&deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_ID)?;
    let secret_order_book_contract_code_hash: String = load(&deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_HASH)?;
    let factory_key: String = load(&deps.storage, FACTORY_KEY)?;

    // check the info from pair AMM
    let response: AmmPairResponse =
    AmmQueryMsg::Pair {}.query(&deps.querier, amm_pair_hash.to_string(), amm_pair_address.to_owned())?;

    let mut token1_info: AssetInfo = match response.asset_infos[0].clone() {
        AmmAssetInfo::NativeToken { denom } => AssetInfo {
            is_native_token: true,
            decimal_places: 6,
            min_order_amount: Uint128(0),
            token: None,
            native_token: Some(NativeToken {
                denom
            })
        },
        AmmAssetInfo::Token { contract_addr, token_code_hash, viewing_key } => AssetInfo {
            is_native_token: false,
            decimal_places: 0,
            min_order_amount: Uint128(0),
            token: Some(Token {
                contract_addr: HumanAddr(contract_addr),
                token_code_hash
            }),
            native_token: None
        }
    };

    let mut token2_info: AssetInfo = match response.asset_infos[1].clone() {
        AmmAssetInfo::NativeToken { denom } => AssetInfo {
            is_native_token: true,
            token: None,
            decimal_places: 6,
            min_order_amount: Uint128(0),
            native_token: Some(NativeToken {
                denom
            })
        },
        crate::msg::AmmAssetInfo::Token { contract_addr, token_code_hash, viewing_key } => AssetInfo {
            is_native_token: false,
            decimal_places: 0,
            min_order_amount: Uint128(0),
            token: Some(Token {
                contract_addr: HumanAddr(contract_addr),
                token_code_hash
            }),
            native_token: None
        }
    };

    let token1_symbol:String;
    let token2_symbol:String;
    //query tokens info and get symbols from Addresses
    match token1_info.clone().is_native_token {
        true => token1_symbol="SCRT".to_string(),
        false => {
            let response_token1 = token_info_query(&deps.querier,BLOCK_SIZE,token1_info.clone().token.unwrap().token_code_hash, token1_info.clone().token.unwrap().contract_addr).unwrap();
            token1_symbol = response_token1.clone().symbol;
            token1_info.decimal_places = response_token1.clone().decimals;
        }
    }
    match token2_info.clone().is_native_token {
        true => token2_symbol="SCRT".to_string(),
        false => {
            let response_token2 = token_info_query(&deps.querier,BLOCK_SIZE,token2_info.clone().token.unwrap().token_code_hash, token2_info.clone().token.unwrap().contract_addr).unwrap();
            token2_symbol = response_token2.clone().symbol;
            token2_info.decimal_places = response_token2.clone().decimals;
        }
    }

    //Define min order bids, this needs to be done because AMM will on swap amounts that will give > 0 value swapped
    //So if we have 18 decimal places vs 6 decimal places the min bid needs to be 1 and 18-6=12 zeroes
    let mut token1_min_order_zeroes:i32;
    if token1_info.decimal_places > token2_info.decimal_places {
        token1_min_order_zeroes = token1_info.decimal_places as i32 - token2_info.decimal_places as i32;
        token1_min_order_zeroes = token1_min_order_zeroes.abs();
    } else {
        token1_min_order_zeroes = 0
    }
    let mut token2_min_order_zeroes:i32;
    if token2_info.decimal_places > token1_info.decimal_places {
        token2_min_order_zeroes = token2_info.decimal_places as i32 - token1_info.decimal_places as i32;
        token2_min_order_zeroes = token2_min_order_zeroes.abs();
    } else {
        token2_min_order_zeroes = 0
    }
    
    token1_info.min_order_amount = Uint128(u128::pow(10,token1_min_order_zeroes as u32));
    token2_info.min_order_amount = Uint128(u128::pow(10,token2_min_order_zeroes as u32));
    //TODO: Deal with duplicated token symbols
    let initmsg = SecretOrderBookContractInitMsg {
        factory_hash: env.contract_code_hash,
        factory_address: env.contract.address,
        factory_key,
        token1_info: token1_info.clone(),
        token2_info: token2_info.clone(),
        amm_pair_contract_address: amm_pair_address.clone(),
        amm_pair_contract_hash: amm_pair_hash.clone(),
    };

    impl InitCallback for SecretOrderBookContractInitMsg {
        const BLOCK_SIZE: usize = BLOCK_SIZE;
    }

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
    amm_pair_address: HumanAddr,
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

    let secret_order_book_contract:SecretOrderBookContract = SecretOrderBookContract{
        contract_addr: contract_address,
        asset_infos: vec![
            token1_info,
            token2_info
        ]
    };

    let mut secret_order_books = PrefixedStorage::new(PREFIX_SECRET_ORDER_BOOKS, &mut deps.storage);
    save(&mut secret_order_books, &deps.api.canonical_address(&amm_pair_address)?.as_slice(), &secret_order_book_contract)?;

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
        QueryMsg::SecretOrderBooks {contract_address} => secret_order_books(deps,contract_address),
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
    contract_address: HumanAddr
) -> QueryResult {
    let secret_order_books = ReadonlyPrefixedStorage::new(PREFIX_SECRET_ORDER_BOOKS, &deps.storage);
    let load_secret_order_book: Option<SecretOrderBookContract> = may_load(&secret_order_books, &deps.api.canonical_address(&contract_address)?.as_slice())?;

    to_binary(&QueryAnswer::SecretOrderBooks {
        secret_order_book: load_secret_order_book
    })
}