use super::*;
use crate::{contract::{PREFIX_VIEW_KEY, query}, msg::ResponseStatus};
use cosmwasm_std::{Extern, HumanAddr, StdResult, testing::*};
use cosmwasm_std::{from_binary, BlockInfo, ContractInfo, MessageInfo, QueryResponse, WasmMsg};
use schemars::_serde_json::to_string;
use std::any::Any;
use crate::state::{save, load, may_load};
use crate::contract::{init, handle,SECRET_ORDER_BOOK_CONTRACT_CODE_ID, FACTORY_KEY, SECRET_ORDER_BOOK_CONTRACT_CODE_HASH};

use cosmwasm_std::{Api, Binary, Env, HandleResponse, HandleResult, InitResponse, Querier, QueryResult, StdError, Storage, to_binary};

use crate::{msg::{HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg}, rand::sha_256};

use crate::viewing_key::{ViewingKey, VIEWING_KEY_SIZE};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};

fn init_helper(
    entropy: String,
    secret_order_book_code_id: u64,
    secret_order_book_code_hash: String,
) -> (
    StdResult<InitResponse>,
    Extern<MockStorage, MockApi, MockQuerier>,
) {
    let mut deps = mock_dependencies(20, &[]);
    let env = mock_env("admin", &[]);

    let init_msg = InitMsg {
        entropy,
        secret_order_book_code_id,
        secret_order_book_code_hash,
    };

    (init(&mut deps, env, init_msg), deps)
}

#[test]
fn test_init() {
    let (init_result, deps) = init_helper(
        "123124".to_string(),
        10,
        "DFADFA123123".to_string()
    );
    assert_eq!(init_result.unwrap(), InitResponse::default());
    
    let arena_contract_code_id: u64 = load(&deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_ID).unwrap();
    let arena_contract_code_hash: String = load(&deps.storage, SECRET_ORDER_BOOK_CONTRACT_CODE_HASH).unwrap();
    let factory_key: String = load(&deps.storage, FACTORY_KEY).unwrap();
    
    assert_eq!(10, arena_contract_code_id);
    assert_eq!("DFADFA123123".to_string(), arena_contract_code_hash);
    assert_eq!("TF9fujurR33f73E4II+o5cLzwuXBMVrT9kpapaqT8GM=".to_string(), factory_key);
}

#[test]
fn test_handle_create_viewkey_and_is_valid() {
    let (init_result, mut deps) = init_helper(
        "123124".to_string(),
        10,
        "DFADFA123123".to_string()
    );
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::CreateViewingKey {
        entropy: "41234123".to_string()
    };

    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    ); 
    let answer: HandleAnswer = from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
    let key = match answer {
        HandleAnswer::ViewingKey { key } => key,
        _ => panic!("NOPE"),
    };
    assert_eq!("6peL/KYDQF7jt8q5+1//aCA0/j/sVNDvzjv3jNNgrx4=".to_string(), key);

    let bob_canonical = deps
    .api
    .canonical_address(&HumanAddr("bob".to_string()))
    .unwrap();

    let read_key = ReadonlyPrefixedStorage::new(PREFIX_VIEW_KEY,&deps.storage);
    let load_key: Option<[u8; VIEWING_KEY_SIZE]> = may_load(&read_key, bob_canonical.as_slice()).unwrap();
    let is_valid = ViewingKey(key).check_viewing_key(&load_key.unwrap());
    assert_eq!(true, is_valid);

    //test query is valid from secret order book contracts (auth queries from users)
    let query_msg = QueryMsg::IsKeyValid {
        address: HumanAddr("bob".to_string()),
        viewing_key: "6peL/KYDQF7jt8q5+1//aCA0/j/sVNDvzjv3jNNgrx4=".to_string(),
        factory_key: "TF9fujurR33f73E4II+o5cLzwuXBMVrT9kpapaqT8GM=".to_string()
    };

    let query_result = query(&deps, query_msg);
    assert!(
        query_result.is_ok(),
        "Init failed: {}",
        query_result.err().unwrap()
    );
    let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
    match query_answer {
        QueryAnswer::IsKeyValid {is_valid} => {
            assert_eq!(is_valid, true);
        }
        _ => panic!("unexpected"),
    }

    // bad factory key
    let query_msg = QueryMsg::IsKeyValid {
        address: HumanAddr("bob".to_string()),
        viewing_key: "6peL/KYDQF7jt8q5+1//aCA0/j/sVNDvzjv3jNNgrx4=".to_string(),
        factory_key: "stuff".to_string()
    };
    let query_result = query(&deps, query_msg);
    assert!(
        query_result.is_err(),
        "Init failed: {}",
        query_result.err().unwrap()
    );

    // bad viewkey
    let query_msg = QueryMsg::IsKeyValid {
        address: HumanAddr("bob".to_string()),
        viewing_key: "stuff".to_string(),
        factory_key: "TF9fujurR33f73E4II+o5cLzwuXBMVrT9kpapaqT8GM=".to_string()
    };
    let query_result = query(&deps, query_msg);
    assert!(
        query_result.is_ok(),
        "Init failed: {}",
        query_result.err().unwrap()
    );
    let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
    match query_answer {
        QueryAnswer::IsKeyValid {is_valid} => {
            assert_eq!(is_valid, false);
        }
        _ => panic!("unexpected"),
    }
}

#[test]
fn test_handle_query_try_change_secret_order_book_contract_code_id() {
    let (init_result, mut deps) = init_helper(
        "123124".to_string(),
        10,
        "DFADFA123123".to_string()
    );
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::ChangeArenaContractCodeId {
        code_id: 5,
        code_hash: "ADSDASDASDA".to_string()
    };

    let handle_result1 = handle(&mut deps, mock_env("bob", &[]), handle_msg.clone());
    assert!(
        handle_result1.is_err(),
        "handle() failed: {}",
        handle_result1.err().unwrap()
    ); 

    let handle_result2 = handle(&mut deps, mock_env("admin", &[]), handle_msg.clone());
    assert!(
        handle_result2.is_ok(),
        "handle() failed: {}",
        handle_result2.err().unwrap()
    ); 
    
    let query_msg = QueryMsg::ArenaContractCodeId {};

    let query_result = query(&deps, query_msg);
    assert!(
        query_result.is_ok(),
        "Init failed: {}",
        query_result.err().unwrap()
    );
    let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
    match query_answer {
        QueryAnswer::ArenaContractCodeID {
            code_id,
            code_hash
        } => {
            assert_eq!(code_id, 5);
            assert_eq!(code_hash, "ADSDASDASDA".to_string());
        }
        _ => panic!("unexpected"),
    }
}

#[test]
fn test_handle_init_callback_from_secret_order_book_to_factory_and_query() {
    let (init_result, mut deps) = init_helper(
        "123124".to_string(),
        10,
        "DFADFA123123".to_string()
    );
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::InitCallBackFromSecretOrderBookToFactory {
        auth_key:"TF9fujurR33f73E4II+o5cLzwuXBMVrT9kpapaqT8GM=".to_string(),
        contract_address: HumanAddr("contract1".to_string()),
        token1_address: HumanAddr("token1".to_string()),
        token2_address: HumanAddr("token2".to_string())
    };

    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg.clone());
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    ); 

    let handle_msg = HandleMsg::InitCallBackFromSecretOrderBookToFactory {
        auth_key:"TF9fujurR33f73E4II+o5cLzwuXBMVrT9kpapaqT8GM=".to_string(),
        contract_address: HumanAddr("contract2".to_string()),
        token1_address: HumanAddr("token1".to_string()),
        token2_address: HumanAddr("token3".to_string())
    };

    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg.clone());
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    ); 
    
    let query_msg = QueryMsg::SecretOrderBooks {
        token_address: HumanAddr("token1".to_string())
    };

    let query_result = query(&deps, query_msg);
    assert!(
        query_result.is_ok(),
        "Init failed: {}",
        query_result.err().unwrap()
    );
    let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
    match query_answer {
        QueryAnswer::SecretOrderBooks {
            secret_order_books
        } => {
            assert_eq!(secret_order_books[0], HumanAddr("contract2".to_string()));
            assert_eq!(secret_order_books[1], HumanAddr("contract1".to_string()));
        }
        _ => panic!("unexpected"),
    }
    
    let query_msg = QueryMsg::SecretOrderBooks {
        token_address: HumanAddr("token2".to_string())
    };

    let query_result = query(&deps, query_msg);
    assert!(
        query_result.is_ok(),
        "Init failed: {}",
        query_result.err().unwrap()
    );
    let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
    match query_answer {
        QueryAnswer::SecretOrderBooks {
            secret_order_books
        } => {
            assert_eq!(secret_order_books[0], HumanAddr("contract1".to_string()))
        }
        _ => panic!("unexpected"),
    }

    let query_msg = QueryMsg::SecretOrderBooks {
        token_address: HumanAddr("token3".to_string())
    };

    let query_result = query(&deps, query_msg);
    assert!(
        query_result.is_ok(),
        "Init failed: {}",
        query_result.err().unwrap()
    );
    let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
    match query_answer {
        QueryAnswer::SecretOrderBooks {
            secret_order_books
        } => {
            assert_eq!(secret_order_books[0], HumanAddr("contract2".to_string()))
        }
        _ => panic!("unexpected"),
    }
}