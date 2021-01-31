#[cfg(test)]
mod tests {
    use cosmwasm_std::{Extern, HumanAddr, StdResult, Uint128, testing::*};
    use crate::{contract::{BID_ORDER_QUEUE, FACTORY_DATA, LIMIT_ORDERS, TOKEN1_DATA, TOKEN2_DATA, handle}, msg::{HandleMsg,
    }, state::{load, may_load}};
    use crate::contract::{init};
    use crate::order_queues::OrderQueue;
    use cosmwasm_std::{Api, InitResponse, to_binary};

    use crate::{msg::{InitMsg, 
        LimitOrderState
    }};


    use cosmwasm_storage::{ReadonlyPrefixedStorage};

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

    #[test]
    fn test_handle_receive_create_n_limit_orders() {
        let (init_result, mut deps) = init_helper(
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

        // Bob send
        let handle_msg = HandleMsg::Receive {
            sender: HumanAddr("token1address".to_string()), 
            from: HumanAddr("bob".to_string()), 
            amount: Uint128(4),
            msg: to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: true,
                price: Uint128(40)
            }).unwrap()
        };

        let handle_result = handle(&mut deps, mock_env("token1address", &[]), handle_msg.clone());

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 

        // Alice send
        let handle_msg = HandleMsg::Receive {
            sender: HumanAddr("token1address".to_string()), 
            from: HumanAddr("alice".to_string()), 
            amount: Uint128(5),
            msg: to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: true,
                price: Uint128(50)
            }).unwrap()
        };

        let handle_result = handle(&mut deps, mock_env("token1address", &[]), handle_msg.clone());

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 

        // Check Bob limit order
        let user_address_bob = &deps.api.canonical_address(&HumanAddr("bob".to_string())).unwrap();

        let limit_orders = ReadonlyPrefixedStorage::new(LIMIT_ORDERS,&deps.storage);
        let load_limit_order: Option<LimitOrderState> = may_load(&limit_orders, &user_address_bob.as_slice()).unwrap();

        assert_eq!(load_limit_order.clone().unwrap().is_bid, true);
        assert_eq!(load_limit_order.clone().unwrap().status, "Active".to_string());
        assert_eq!(load_limit_order.clone().unwrap().price, Uint128(40));
        assert_eq!(load_limit_order.clone().unwrap().balances, vec![Uint128(4),Uint128(0)]);

        // Check Alice limit order
        let user_address_alice = &deps.api.canonical_address(&HumanAddr("alice".to_string())).unwrap();

        let limit_orders = ReadonlyPrefixedStorage::new(LIMIT_ORDERS,&deps.storage);
        let load_limit_order: Option<LimitOrderState> = may_load(&limit_orders, &user_address_alice.as_slice()).unwrap();

        assert_eq!(load_limit_order.clone().unwrap().is_bid, true);
        assert_eq!(load_limit_order.clone().unwrap().status, "Active".to_string());
        assert_eq!(load_limit_order.clone().unwrap().price, Uint128(50));
        assert_eq!(load_limit_order.clone().unwrap().balances, vec![Uint128(5),Uint128(0)]);

        // Check order queue
        let mut bid_order_book:OrderQueue=load(&deps.storage, BID_ORDER_QUEUE).unwrap();
        assert_eq!(bid_order_book.peek().unwrap().id, user_address_alice.clone());
        assert_eq!(bid_order_book.peek().unwrap().price, Uint128(50));
        assert_eq!(bid_order_book.pop().unwrap().id, user_address_alice.clone());
        assert_eq!(bid_order_book.pop().unwrap().id, user_address_bob.clone());
        assert_eq!(bid_order_book.peek(), None);
        assert_eq!(bid_order_book.pop(), None);
    }
}