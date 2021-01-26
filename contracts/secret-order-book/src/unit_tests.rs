#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{Extern, HumanAddr, StdResult, testing::*};
    use cosmwasm_std::{from_binary, BlockInfo, ContractInfo, MessageInfo, QueryResponse, WasmMsg};
    use schemars::_serde_json::to_string;
    use std::any::Any;
    use crate::{contract::{TOKEN1_DATA, FACTORY_DATA, TOKEN2_DATA}, state::{save, load, may_load}};
    use crate::contract::{init};

    use cosmwasm_std::{Api, Binary, Env, HandleResponse, HandleResult, InitResponse, Querier, QueryResult, StdError, Storage, to_binary};

    use crate::{msg::{InitMsg}};


    use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};

    fn init_helper(
        factory_address: HumanAddr,
        factory_hash: String,
        factory_key: String,
        token1_code_address: HumanAddr,
        token1_code_hash: String,
        token2_code_address: HumanAddr,
        token2_code_hash: String
    ) -> (
        StdResult<InitResponse>,
        Extern<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env("admin", &[]);

        let init_msg = InitMsg {
            factory_address,
            factory_hash,
            factory_key,
            token1_code_address,
            token1_code_hash,
            token2_code_address,
            token2_code_hash
        };

        (init(&mut deps, env, init_msg), deps)
    }

    #[test]
    fn test_init() {
        let (init_result, deps) = init_helper(
            HumanAddr("factoryaddress".to_string()),
            "factoryhash".to_string(),
            "factorykey".to_string(),
            HumanAddr("token1address".to_string()),
            "token1hash".to_string(),
            HumanAddr("token2address".to_string()),
            "token2hash".to_string(),
        );
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let factory_data = ReadonlyPrefixedStorage::new(FACTORY_DATA,&deps.storage);
        let load_factory_data: Option<HumanAddr> = may_load(&factory_data, b"address").unwrap();
        let load_factory_hash: Option<String> = may_load(&factory_data, b"hash").unwrap();
        
        assert_eq!(load_factory_data.unwrap(), HumanAddr("factoryaddress".to_string()));
        assert_eq!(load_factory_hash.unwrap(), "factoryhash".to_string());

        let token1_data = ReadonlyPrefixedStorage::new(TOKEN1_DATA,&deps.storage);
        let load_token1_data: Option<HumanAddr> = may_load(&token1_data, b"address").unwrap();
        let load_token1_hash: Option<String> = may_load(&token1_data, b"hash").unwrap();
        
        assert_eq!(load_token1_data.unwrap(), HumanAddr("token1address".to_string()));
        assert_eq!(load_token1_hash.unwrap(), "token1hash".to_string());

        let token2_data = ReadonlyPrefixedStorage::new(TOKEN2_DATA,&deps.storage);
        let load_token2_data: Option<HumanAddr> = may_load(&token2_data, b"address").unwrap();
        let load_token2_hash: Option<String> = may_load(&token2_data, b"hash").unwrap();
        
        assert_eq!(load_token2_data.unwrap(), HumanAddr("token2address".to_string()));
        assert_eq!(load_token2_hash.unwrap(), "token2hash".to_string());
    }
}