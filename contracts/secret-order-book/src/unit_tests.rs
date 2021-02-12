#[cfg(test)]
mod tests {
    use cosmwasm_std::{Coin, Extern, HumanAddr, Querier, QuerierResult, StdResult, Uint128, from_binary, testing::*};
    use crate::{contract::{BID_ORDER_QUEUE, FACTORY_DATA, LIMIT_ORDERS, TOKEN1_DATA, TOKEN2_DATA, handle}, msg::{AssetInfo, HandleMsg, NativeToken, QueryMsg, Token}, state::{load, may_load}};
    use crate::contract::{init};
    use crate::order_queues::OrderQueue;
    use cosmwasm_std::{Api, InitResponse, to_binary};
    use secret_toolkit::utils::{Query};
    use crate::{msg::{InitMsg, 
        LimitOrderState
    }};

    use cosmwasm_storage::{ReadonlyPrefixedStorage};

    fn init_helper(
        factory_address: HumanAddr,
        factory_hash: String,
        factory_key: String,
        token1_info: AssetInfo,
        token2_info: AssetInfo,
        amm_pair_contract_address: HumanAddr,
        amm_pair_contract_hash: String
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
            token1_info,
            token2_info,
            amm_pair_contract_address,
            amm_pair_contract_hash
        };

        (init(&mut deps, env, init_msg), deps)
    }

    #[test]
    fn test_init() {
        let (init_result, deps) = init_helper(
            HumanAddr("factoryaddress".to_string()),
            "factoryhash".to_string(),
            "factorykey".to_string(),
            AssetInfo {
                is_native_token: true,
                token: None,
                native_token: Some(
                    NativeToken{denom:"uscrt".to_string()}
                )
            },
            AssetInfo {
                is_native_token: false,
                token: Some(
                    Token {
                        contract_addr: HumanAddr("token2address".to_string()),
                        token_code_hash: "token2hash".to_string()
                    }
                ),
                native_token: None
            },
            HumanAddr("ammpairaddress".to_string()),
            "ammpairhash".to_string()
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

        let token1_info: AssetInfo=load(&deps.storage, TOKEN1_DATA).unwrap();
        assert_eq!(token1_info, AssetInfo {
            is_native_token: true,
            token: None,
            native_token: Some(
                NativeToken{denom:"uscrt".to_string()}
            )
        });

        let token2_info: AssetInfo=load(&deps.storage, TOKEN2_DATA).unwrap();
        assert_eq!(token2_info, AssetInfo {
            is_native_token: false,
            token: Some(
                Token {
                    contract_addr: HumanAddr("token2address".to_string()),
                    token_code_hash: "token2hash".to_string()
                }
            ),
            native_token: None
        });
    }

    #[test]
    fn test_handle_receive_create_n_limit_orders_2_tokens() {
        let (init_result, mut deps) = init_helper(
            HumanAddr("factoryaddress".to_string()),
            "factoryhash".to_string(),
            "factorykey".to_string(),
            AssetInfo {
                is_native_token: false,
                token: Some(
                    Token {
                        contract_addr: HumanAddr("token1address".to_string()),
                        token_code_hash: "token1hash".to_string()
                    }
                ),
                native_token: None
            },
            AssetInfo {
                is_native_token: false,
                token: Some(
                    Token {
                        contract_addr: HumanAddr("token2address".to_string()),
                        token_code_hash: "token2hash".to_string()
                    }
                ),
                native_token: None
            },
            HumanAddr("ammpairaddress".to_string()),
            "ammpairhash".to_string()
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

        // Trigerer send
        let handle_msg = HandleMsg::TriggerLimitOrders {
            test_amm_price: Uint128(50)
        };
        let handle_result = handle(&mut deps, mock_env("trigerer", &[]), handle_msg.clone());
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 

        let limit_orders = ReadonlyPrefixedStorage::new(LIMIT_ORDERS,&deps.storage);
        let load_limit_order: Option<LimitOrderState> = may_load(&limit_orders, &user_address_alice.as_slice()).unwrap();
        assert_eq!(load_limit_order.clone().unwrap().balances, vec![Uint128(0),Uint128(5)]);
    
        // withdraw
        let handle_msg = HandleMsg::WithdrawLimitOrder {};

        let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg.clone());
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 
    }

    #[test]
    fn test_handle_receive_create_n_limit_order_native_token() {
        let (init_result, mut deps) = init_helper(
            HumanAddr("factoryaddress".to_string()),
            "factoryhash".to_string(),
            "factorykey".to_string(),
            AssetInfo {
                is_native_token: true,
                token: None,
                native_token: Some(
                    NativeToken{denom:"uscrt".to_string()}
                )
            },
            AssetInfo {
                is_native_token: false,
                token: Some(
                    Token {
                        contract_addr: HumanAddr("token2address".to_string()),
                        token_code_hash: "token2hash".to_string()
                    }
                ),
                native_token: None
            },
            HumanAddr("ammpairaddress".to_string()),
            "ammpairhash".to_string()
        );
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // Bob send
        let handle_msg = HandleMsg::ReceiveNativeToken {
            is_bid: true,
            price: Uint128(40)
        };

        let handle_result = handle(&mut deps, mock_env(
            "bob", 
            &[Coin{amount:Uint128(4),denom:"uscrt".to_string(),}]), 
            handle_msg.clone()
        );

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 

        // Alice send
        let handle_msg = HandleMsg::ReceiveNativeToken {
            is_bid: true,
            price: Uint128(50)
        };

        let handle_result = handle(&mut deps, mock_env(
            "alice", 
            &[Coin{amount:Uint128(5),denom:"uscrt".to_string()}]), 
            handle_msg.clone()
        );

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

        let mut bid_order_book:OrderQueue=load(&deps.storage, BID_ORDER_QUEUE).unwrap();
        assert_eq!(bid_order_book.peek().unwrap().id, user_address_alice.clone());
        assert_eq!(bid_order_book.peek().unwrap().price, Uint128(50));
        assert_eq!(bid_order_book.pop().unwrap().id, user_address_alice.clone());
        assert_eq!(bid_order_book.pop().unwrap().id, user_address_bob.clone());
        assert_eq!(bid_order_book.peek(), None);
        assert_eq!(bid_order_book.pop(), None);

        // Trigerer send
        let handle_msg = HandleMsg::TriggerLimitOrders {
            test_amm_price: Uint128(50)
        };
        let handle_result = handle(&mut deps, mock_env("trigerer", &[]), handle_msg.clone());
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 

        let limit_orders = ReadonlyPrefixedStorage::new(LIMIT_ORDERS,&deps.storage);
        let load_limit_order: Option<LimitOrderState> = may_load(&limit_orders, &user_address_alice.as_slice()).unwrap();
        assert_eq!(load_limit_order.clone().unwrap().balances, vec![Uint128(0),Uint128(5)]);
    }

    #[test]
    fn test_get_limit_order_to_trigger() {
        let (init_result, mut deps) = init_helper(
            HumanAddr("factoryaddress".to_string()),
            "factoryhash".to_string(),
            "factorykey".to_string(),
            AssetInfo {
                is_native_token: false,
                token: Some(
                    Token {
                        contract_addr: HumanAddr("token1address".to_string()),
                        token_code_hash: "token1hash".to_string()
                    }
                ),
                native_token: None
            },
            AssetInfo {
                is_native_token: false,
                token: Some(
                    Token {
                        contract_addr: HumanAddr("token2address".to_string()),
                        token_code_hash: "token2hash".to_string()
                    }
                ),
                native_token: None
            },
            HumanAddr("ammpairaddress".to_string()),
            "ammpairhash".to_string()
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
            amount: Uint128(10),
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

        
        use serde::{Deserialize, Serialize};
        #[derive(Debug)]
        struct MyMockQuerier {}
        #[derive(Serialize, Deserialize, Debug)]
        pub struct MockedSimulation {
            pub return_amount: Uint128,
            pub spread_amount: Uint128,
            pub commission_amount: Uint128
        }

        impl Querier for MyMockQuerier {
            fn raw_query(&self, _request: &[u8]) -> QuerierResult {
                Ok(to_binary(&MockedSimulation {
                    return_amount: Uint128(1),
                    spread_amount: Uint128(1),
                    commission_amount: Uint128(1)
                }))
            }
        }

        let mocked_deps = deps.change_querier(|_| MyMockQuerier {});
        
        use crate::contract::query;

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, false);
        
    }
}