use std::u128;

use cosmwasm_std::{Api, Binary, CanonicalAddr, Env, Extern, HandleResponse, HandleResult, HumanAddr, InitResponse, Querier, QueryResult, ReadonlyStorage, StdError, StdResult, Storage, Uint128, to_binary};

use crate::{msg::{AmmAssetInfo, AmmPairResponse, AmmQueryMsg, AssetInfo, HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg, ResponseStatus::Success, SecretOrderBookContract, SecretOrderBookContractInitMsg, Token}, rand::sha_256};
use crate::state::{save, load, may_load};
use crate::viewing_key::{ViewingKey, VIEWING_KEY_SIZE};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
 
use secret_toolkit::{snip20::token_info_query, storage::{AppendStore, AppendStoreMut}, utils::{InitCallback, Query}};

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
/// storage key for the secret order books
pub const PREFIX_SECRET_ORDER_BOOK: &[u8] = b"secretorderbook";
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
            amm_pair_hash,
            token1_fee,
            token2_fee
        } => try_secret_order_book_instanciate(deps, env, &amm_pair_address, &amm_pair_hash, &token1_fee, &token2_fee),
        HandleMsg::InitCallBackFromSecretOrderBookToFactory {
            auth_key, 
            amm_pair_address,
            contract_address,  
            token1_info, 
            token2_info
        } => try_secret_order_book_instanciated_callback(deps, env, auth_key, amm_pair_address, contract_address, token1_info, token2_info),
        HandleMsg::ChangeAssetFee {
            amm_pairs_address,
            asset_contract_address,
            new_asset_fee
        } => try_change_asset_fee(deps, env, amm_pairs_address, asset_contract_address, new_asset_fee)
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
    amm_pair_hash: &String,
    token1_fee: &Uint128,
    token2_fee: &Uint128
) -> HandleResult {  
    let secret_order_book_contract_code_id: u64 = load(&deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_ID)?;
    let secret_order_book_contract_code_hash: String = load(&deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_HASH)?;
    let factory_key: String = load(&deps.storage, FACTORY_KEY)?;
    let admin: HumanAddr = load(&deps.storage, ADMIN_KEY)?;
    if env.message.sender != admin {
        return Err(StdError::generic_err(
            "Permission Denied.",
        ));
    }

    // check the info from pair AMM
    let response: AmmPairResponse =
    AmmQueryMsg::Pair {}.query(&deps.querier, amm_pair_hash.to_string(), amm_pair_address.to_owned())?;

    let mut token1_info: AssetInfo = match response.asset_infos[0].clone() {
        AmmAssetInfo::NativeToken { .. } => {
            return Err(StdError::generic_err(
                "Native token not supported!",
            ));
        },
        AmmAssetInfo::Token { contract_addr, token_code_hash, viewing_key } => AssetInfo {
            decimal_places: 0,
            base_amount: Uint128(0),
            fee_amount: token1_fee.clone(),
            min_amount: token1_fee.clone().multiply_ratio(Uint128(2),Uint128(1)),
            token: Some(Token {
                contract_addr: HumanAddr(contract_addr),
                token_code_hash
            })
        }
    };

    let mut token2_info: AssetInfo = match response.asset_infos[1].clone() {
        AmmAssetInfo::NativeToken { .. } => {
            return Err(StdError::generic_err(
                "Native token not supported!",
            ));
        },
        crate::msg::AmmAssetInfo::Token { contract_addr, token_code_hash, viewing_key } => AssetInfo {
            decimal_places: 0,
            base_amount: Uint128(0),
            fee_amount: token2_fee.clone(),
            min_amount: token2_fee.clone().multiply_ratio(Uint128(2),Uint128(1)),
            token: Some(Token {
                contract_addr: HumanAddr(contract_addr),
                token_code_hash
            })
        }
    };

    let token1_symbol:String;
    let token2_symbol:String;
    //query tokens info and get symbols from Addresses
    let response_token1 = token_info_query(&deps.querier,BLOCK_SIZE,token1_info.clone().token.unwrap().token_code_hash, token1_info.clone().token.unwrap().contract_addr).unwrap();
    token1_symbol = response_token1.clone().symbol;
    token1_info.decimal_places = response_token1.clone().decimals;

    let response_token2 = token_info_query(&deps.querier,BLOCK_SIZE,token2_info.clone().token.unwrap().token_code_hash, token2_info.clone().token.unwrap().contract_addr).unwrap();
    token2_symbol = response_token2.clone().symbol;
    token2_info.decimal_places = response_token2.clone().decimals;
   
    token1_info.base_amount = Uint128(u128::pow(10,token1_info.decimal_places as u32));
    token2_info.base_amount = Uint128(u128::pow(10,token2_info.decimal_places as u32));

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
        amm_pair_contract_addr: amm_pair_address.clone(),
        contract_addr: contract_address,
        asset_infos: vec![
            token1_info,
            token2_info
        ]
    };

    // Store this contract
    let mut secret_order_books = PrefixedStorage::new(PREFIX_SECRET_ORDER_BOOKS, &mut deps.storage);
    let mut secret_order_books = AppendStoreMut::attach_or_create(&mut secret_order_books)?;
    secret_order_books.push(&secret_order_book_contract)?;

    let mut secret_order_book = PrefixedStorage::new(PREFIX_SECRET_ORDER_BOOK, &mut deps.storage);
    save(&mut secret_order_book, &deps.api.canonical_address(&amm_pair_address.clone())?.as_slice(), &secret_order_book_contract)?;

    Ok(HandleResponse::default())
}

pub fn try_change_asset_fee<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amm_pairs_address: Vec<HumanAddr>,
    asset_contract_address: HumanAddr,
    new_asset_fee: Uint128
) -> HandleResult {   
    let admin: HumanAddr = load(&deps.storage, ADMIN_KEY)?;
    if env.message.sender != admin {
        return Err(StdError::generic_err(
            "Permission Denied.",
        ));
    }
    // 1. Get each secret order book associated with each amm_pair_address indicated
    let mut secret_order_books = PrefixedStorage::new(PREFIX_SECRET_ORDER_BOOK, &mut deps.storage);
    
    for i in 0..amm_pairs_address.len() { 
        let secret_order_book: SecretOrderBookContract = may_load(&secret_order_books, &deps.api.canonical_address(&amm_pairs_address[i])?.as_slice())?.unwrap();
        let mut modified_secret_order_book:SecretOrderBookContract = secret_order_book.clone();
        let token_index: usize;

        // 2. Search the asset info that have the asset_contract_address indicated
        if modified_secret_order_book.asset_infos[0].token.clone().unwrap().contract_addr == asset_contract_address { token_index = 0 }
        else if modified_secret_order_book.asset_infos[1].token.clone().unwrap().contract_addr == asset_contract_address { token_index = 1 }
        else {
            return Err(StdError::generic_err(format!(
                "Error on: {:?}", amm_pairs_address[i]
            ))); 
        }

        // TODO
        // 3. Modify the asset_info with the new fee and send to the secret order book this change
        // 3.1 PREFIX_SECRET_ORDER_BOOK
        // 3.2 PREFIX_SECRET_ORDER_BOOKS
        // 3.3 SEND TO SECRET ORDER BOOK CONTRACTS
        modified_secret_order_book.asset_infos[token_index].fee_amount = new_asset_fee;

        save(&mut secret_order_books, &deps.api.canonical_address(&amm_pairs_address[i])?.as_slice(), &modified_secret_order_book)?;
    }

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
        QueryMsg::SecretOrderBook {amm_pair_contract_addr} => secret_order_book(deps,amm_pair_contract_addr),
        QueryMsg::SecretOrderBooks {page_size, page} => secret_order_books(deps, page_size, page)
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
    page_size: Option<u32>,
    page: Option<u32>
) -> QueryResult {
    let secret_order_books = ReadonlyPrefixedStorage::new(PREFIX_SECRET_ORDER_BOOKS, &deps.storage);
    
    let store = if let Some(result) = AppendStore::<SecretOrderBookContract, _>::attach(&secret_order_books) {
        result?
    } else {
        return to_binary(&QueryAnswer::SecretOrderBooks {
            secret_order_books: vec![]
        });
    };

    let response:Vec<SecretOrderBookContract>;
    if page_size != None && page != None {
        let tx_iter = store
        .iter()
        .skip((page.unwrap() * page_size.unwrap()) as _)
        .take(page_size.unwrap() as _);

        let txs: StdResult<Vec<SecretOrderBookContract>> = tx_iter.collect();
        response = txs.unwrap()
    } else {
        let tx_iter = store.iter();
        let txs: StdResult<Vec<SecretOrderBookContract>> = tx_iter.collect();
        response = txs.unwrap()
    }

    return to_binary(&QueryAnswer::SecretOrderBooks {
        secret_order_books: response
    });
}

fn secret_order_book <S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    amm_pair_contract_addr: HumanAddr
) -> QueryResult {
    let secret_order_books = ReadonlyPrefixedStorage::new(PREFIX_SECRET_ORDER_BOOK, &deps.storage);
    let load_secret_order_book: Option<SecretOrderBookContract> = may_load(&secret_order_books, &deps.api.canonical_address(&amm_pair_contract_addr)?.as_slice())?;

    to_binary(&QueryAnswer::SecretOrderBook {
        secret_order_book: load_secret_order_book
    }) 
}