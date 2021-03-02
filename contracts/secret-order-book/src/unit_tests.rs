#[cfg(test)]
mod tests {
    use cosmwasm_std::{CanonicalAddr, Binary, Coin, Decimal, Extern, HumanAddr, Querier, QuerierResult, QueryRequest, StdResult, Uint128, WasmQuery, from_binary, testing::*, to_vec};
    use secret_toolkit::storage::AppendStore;
    use crate::{contract::{BID_ORDER_QUEUE, SWAPPED_LIMIT_ORDER, FACTORY_DATA, ACTIVE_LIMIT_ORDERS, HISTORY_LIMIT_ORDERS, TOKEN1_DATA, TOKEN2_DATA, handle}, msg::{QueryAnswer, AssetInfo, HandleMsg, NativeToken, QueryMsg, Token}, state::{load, save, may_load}};
    use crate::contract::{init};
    use crate::order_queues::OrderQueue;
    use cosmwasm_std::{Api, InitResponse, to_binary};
    use crate::contract::query;
    use crate::{msg::{InitMsg, 
        UserOrderMap,
        LimitOrderState,
        AmmSimulationQuery,
        AmmPairSimulationResponse
    }};

    use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};

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

    /* 
    #[test]
    fn test_init() {
        let (init_result, deps) = init_helper(
            HumanAddr("factoryaddress".to_string()),
            "factoryhash".to_string(),
            "factorykey".to_string(),
            AssetInfo {
                is_native_token: true,
                decimal_places: 6,
                base_amount: Uint128(1000000),
                token: None,
                native_token: Some(
                    NativeToken{denom:"uscrt".to_string()}
                )
            },
            AssetInfo {
                is_native_token: false,
                decimal_places: 18,
                base_amount: Uint128(1000000000000000000),
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
            decimal_places: 6,
            base_amount: Uint128(1000000),
            token: None,
            native_token: Some(
                NativeToken{denom:"uscrt".to_string()}
            )
        });

        let token2_info: AssetInfo=load(&deps.storage, TOKEN2_DATA).unwrap();
        assert_eq!(token2_info, AssetInfo {
            is_native_token: false,
            decimal_places: 18,
            base_amount: Uint128(1000000000000000000),
            token: Some(
                Token {
                    contract_addr: HumanAddr("token2address".to_string()),
                    token_code_hash: "token2hash".to_string()
                }
            ),
            native_token: None
        });
    }*/

    #[test]
    fn test_handle_receive_create_n_limit_orders_2_tokens() {
        let (init_result, mut deps) = init_helper(
            HumanAddr("factoryaddress".to_string()),
            "factoryhash".to_string(),
            "factorykey".to_string(),
            AssetInfo {
                is_native_token: false,
                decimal_places: 18,
                base_amount: Uint128(1000000000000000000),
                fee_amount: Uint128(500000000000000000),
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
                decimal_places: 18,
                base_amount: Uint128(1000000000000000000),
                fee_amount: Uint128(500000000000000000),
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
            sender: HumanAddr("token2address".to_string()), 
            from: HumanAddr("bob".to_string()), 
            amount: Uint128(4000000000000000000),
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: true,
                price: Uint128(4000000000000000000),
                expected_amount: Uint128(1000000000000000000)
            }).unwrap())
        };

        let handle_result = handle(&mut deps, mock_env("token2address", &[]), handle_msg.clone());

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 

        // Alice send
        let handle_msg = HandleMsg::Receive {
            sender: HumanAddr("token2address".to_string()), 
            from: HumanAddr("alice".to_string()), 
            amount: Uint128(5000000000000000000),
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: true,
                price: Uint128(5000000000000000000),
                expected_amount: Uint128(1000000000000000000)
            }).unwrap())
        };

        let handle_result = handle(&mut deps, mock_env("token2address", &[]), handle_msg.clone());

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 

        // Check Bob limit order
        let user_address_bob = &deps.api.canonical_address(&HumanAddr("bob".to_string())).unwrap();

        let limit_orders = ReadonlyPrefixedStorage::new(ACTIVE_LIMIT_ORDERS,&deps.storage);
        let load_limit_order: Option<LimitOrderState> = may_load(&limit_orders, &user_address_bob.as_slice()).unwrap();

        assert_eq!(load_limit_order.clone().unwrap().is_bid, true);
        assert_eq!(load_limit_order.clone().unwrap().status, "Active".to_string());
        assert_eq!(load_limit_order.clone().unwrap().price, Uint128(4000000000000000000));
        assert_eq!(load_limit_order.clone().unwrap().balances, vec![Uint128(0),Uint128(4000000000000000000)]);

        // Check Alice limit order
        let user_address_alice = &deps.api.canonical_address(&HumanAddr("alice".to_string())).unwrap();

        let limit_orders = ReadonlyPrefixedStorage::new(ACTIVE_LIMIT_ORDERS,&deps.storage);
        let load_limit_order: Option<LimitOrderState> = may_load(&limit_orders, &user_address_alice.as_slice()).unwrap();

        assert_eq!(load_limit_order.clone().unwrap().is_bid, true);
        assert_eq!(load_limit_order.clone().unwrap().status, "Active".to_string());
        assert_eq!(load_limit_order.clone().unwrap().price, Uint128(5000000000000000000));
        assert_eq!(load_limit_order.clone().unwrap().balances, vec![Uint128(0),Uint128(5000000000000000000)]);

        // Check order queue
        let mut bid_order_book:OrderQueue=load(&deps.storage, BID_ORDER_QUEUE).unwrap();
        assert_eq!(bid_order_book.peek().unwrap().id, HumanAddr("alice".to_string()));
        assert_eq!(bid_order_book.peek().unwrap().price, Uint128(5000000000000000000));
        assert_eq!(bid_order_book.pop().unwrap().id, HumanAddr("alice".to_string()));
        assert_eq!(bid_order_book.pop().unwrap().id, HumanAddr("bob".to_string()));
        assert_eq!(bid_order_book.peek(), None);
        assert_eq!(bid_order_book.pop(), None);

        // get limit orders
        /*let query_msg = QueryMsg::GetLimitOrders {
            user_address: HumanAddr("alice".to_string()),
            user_viewkey: "".to_string()
        };
        let query_result = query(&deps, query_msg);

        let result:OrderBookPairResponse = from_binary(&query_result.unwrap()).unwrap();
        */
        // withdraw
        let handle_msg = HandleMsg::WithdrawLimitOrder {};

        let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg.clone());
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 

        let limit_orders = ReadonlyPrefixedStorage::new(ACTIVE_LIMIT_ORDERS,&deps.storage);
        let load_limit_order: Option<LimitOrderState> = may_load(&limit_orders, &user_address_alice.as_slice()).unwrap();
        assert_eq!(load_limit_order, None);

        // add again
        // Alice send
        let handle_msg = HandleMsg::Receive {
            sender: HumanAddr("token2address".to_string()), 
            from: HumanAddr("alice".to_string()), 
            amount: Uint128(14000000000000000000),
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: true,
                price: Uint128(7000000000000000000),
                expected_amount: Uint128(2000000000000000000)
            }).unwrap())
        };

        let handle_result = handle(&mut deps, mock_env("token2address", &[]), handle_msg.clone());

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 

        let mut bid_order_book:OrderQueue=load(&deps.storage, BID_ORDER_QUEUE).unwrap();
        assert_eq!(bid_order_book.peek().unwrap().id, HumanAddr("alice".to_string()));

        //check limit orders
        let history_limit_orders = ReadonlyPrefixedStorage::multilevel(&[HISTORY_LIMIT_ORDERS, user_address_alice.as_slice()], &deps.storage);
        let store = AppendStore::<LimitOrderState, _>::attach(&history_limit_orders);
        let tx_iter = store.unwrap().unwrap().iter();
        let txs: StdResult<Vec<LimitOrderState>> = tx_iter.collect();
        
        assert_eq!(txs.unwrap().len(), 1);

        //widthdraw again
        let handle_msg = HandleMsg::WithdrawLimitOrder {};

        let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg.clone());
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 

        //check limit orders
        let history_limit_orders = ReadonlyPrefixedStorage::multilevel(&[HISTORY_LIMIT_ORDERS, user_address_alice.as_slice()], &deps.storage);
        let store = AppendStore::<LimitOrderState, _>::attach(&history_limit_orders);
        let tx_iter = store.unwrap().unwrap().iter();
        let txs: StdResult<Vec<LimitOrderState>> = tx_iter.collect();

        assert_eq!(txs.unwrap().len(), 2);
    }

    /*#[test]
    fn test_handle_receive_create_n_limit_order_native_token() {
        let (init_result, mut deps) = init_helper(
            HumanAddr("factoryaddress".to_string()),
            "factoryhash".to_string(),
            "factorykey".to_string(),
            AssetInfo {
                is_native_token: false,
                decimal_places: 18,
                base_amount: Uint128(1000000000000000000),
                token: Some(
                    Token {
                        contract_addr: HumanAddr("token2address".to_string()),
                        token_code_hash: "token2hash".to_string()
                    }
                ),
                native_token: None
            },
            AssetInfo {
                is_native_token: true,
                decimal_places: 6,
                base_amount: Uint128(1000000),
                token: None,
                native_token: Some(
                    NativeToken{denom:"uscrt".to_string()}
                )
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
            &[Coin{amount:Uint128(400),denom:"uscrt".to_string(),}]), 
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
            &[Coin{amount:Uint128(500000000000000),denom:"uscrt".to_string()}]), 
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
        assert_eq!(load_limit_order.clone().unwrap().price, Uint128(400000000000000));
        assert_eq!(load_limit_order.clone().unwrap().balances, vec![Uint128(0),Uint128(400000000000000)]);

        // Check Alice limit order
        let user_address_alice = &deps.api.canonical_address(&HumanAddr("alice".to_string())).unwrap();

        let limit_orders = ReadonlyPrefixedStorage::new(LIMIT_ORDERS,&deps.storage);
        let load_limit_order: Option<LimitOrderState> = may_load(&limit_orders, &user_address_alice.as_slice()).unwrap();

        assert_eq!(load_limit_order.clone().unwrap().is_bid, true);
        assert_eq!(load_limit_order.clone().unwrap().status, "Active".to_string());
        assert_eq!(load_limit_order.clone().unwrap().price, Uint128(50));
        assert_eq!(load_limit_order.clone().unwrap().balances, vec![Uint128(0),Uint128(500000000000000)]);

        let mut bid_order_book:OrderQueue=load(&deps.storage, BID_ORDER_QUEUE).unwrap();
        assert_eq!(bid_order_book.peek().unwrap().id, user_address_alice.clone());
        assert_eq!(bid_order_book.peek().unwrap().price, Uint128(500000000000000));
        assert_eq!(bid_order_book.pop().unwrap().id, user_address_alice.clone());
        assert_eq!(bid_order_book.pop().unwrap().id, user_address_bob.clone());
        assert_eq!(bid_order_book.peek(), None);
        assert_eq!(bid_order_book.pop(), None);

        // Trigerer send
        /*let handle_msg = HandleMsg::TriggerLimitOrders {};
        let handle_result = handle(&mut deps, mock_env("trigerer", &[]), handle_msg.clone());
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 

        let limit_orders = ReadonlyPrefixedStorage::new(LIMIT_ORDERS,&deps.storage);
        let load_limit_order: Option<LimitOrderState> = may_load(&limit_orders, &user_address_alice.as_slice()).unwrap();
        assert_eq!(load_limit_order.clone().unwrap().balances, vec![Uint128(0),Uint128(500000000000000)]);
        */
    }
*/
    #[test]
    fn test_get_limit_order_to_trigger() {
        // Test Limit Orders
        // 1 Order - price 10, quantity 10
        // 2 Order - price 9, quantity 2 
        let (init_result, mut deps) = init_helper(
            HumanAddr("factoryaddress".to_string()),
            "factoryhash".to_string(),
            "factorykey".to_string(),
            AssetInfo {
                is_native_token: false,
                decimal_places: 6,
                base_amount: Uint128(1000000),
                fee_amount: Uint128(500000),
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
                decimal_places: 18,
                base_amount: Uint128(1000000000000000000),
                fee_amount: Uint128(500000000000000000),
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
            sender: HumanAddr("token2address".to_string()), 
            from: HumanAddr("bob".to_string()), 
            amount: Uint128(2000000000000000000), // 2e18
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: true,
                price: Uint128(8000000000000000000), // 8e18
                expected_amount: Uint128(250000) //0.25e6
            }).unwrap())
        };

        let handle_result = handle(&mut deps, mock_env("token2address", &[]), handle_msg.clone());

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );

        // Alice send
        let handle_msg = HandleMsg::Receive {
            sender: HumanAddr("token2address".to_string()), 
            from: HumanAddr("alice".to_string()), 
            amount: Uint128(10000000000000000000), // 10e18
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: true,
                price: Uint128(9000000000000000000), // 9e18
                expected_amount: Uint128(1111111) // 1.11e6
            }).unwrap())
        };

        let handle_result = handle(&mut deps, mock_env("token2address", &[]), handle_msg.clone());

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );      

        use serde::{Deserialize, Serialize};
        #[derive(Debug)]
        struct MyMockQuerier {
            expected_bid_base_request: Vec<u8>,
            expected_bid_base_response: AmmPairSimulationResponse,
            expected_bid_amount_request: Option<Vec<u8>>,
            expected_bid_amount_response: Option<AmmPairSimulationResponse>,
            expected_bid_amount2_request: Option<Vec<u8>>,
            expected_bid_amount2_response: Option<AmmPairSimulationResponse>,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct MockedSimulation {
            pub return_amount: Uint128,
            pub spread_amount: Uint128,
            pub commission_amount: Uint128
        }
        
        pub fn space_pad(message: &mut Vec<u8>, block_size: usize) -> &mut Vec<u8> {
            let len = message.len();
            let surplus = len % block_size;
            if surplus == 0 {
                return message;
            }
        
            let missing = block_size - surplus;
            message.reserve(missing);
            message.extend(std::iter::repeat(b' ').take(missing));
            message
        }

        pub fn check_mock_request(msg: Vec<u8>) -> Vec<u8>{
            let mut expected_msg = msg.clone();
            space_pad(&mut expected_msg, 256);
            let expected_request: QueryRequest<AmmSimulationQuery> =
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: HumanAddr("ammpairaddress".to_string()),
                callback_code_hash: "ammpairhash".to_string(),
                msg: Binary(expected_msg),
            });
            to_vec(&expected_request).unwrap()       
        }

        impl Querier for MyMockQuerier {
            fn raw_query(&self, request: &[u8] ) -> QuerierResult {
                let bid_base_req = check_mock_request(self.expected_bid_base_request.clone());
                if request == bid_base_req {
                    return Ok(to_binary(&MockedSimulation {
                        return_amount: self.expected_bid_base_response.return_amount,
                        spread_amount: self.expected_bid_base_response.spread_amount,
                        commission_amount: self.expected_bid_base_response.commission_amount
                    }));
                }
                if self.expected_bid_amount_request.clone() != None {
                    let bid_amount_req = check_mock_request(self.expected_bid_amount_request.clone().unwrap());
                    if request == bid_amount_req {
                        return Ok(to_binary(&MockedSimulation {
                            return_amount: self.expected_bid_amount_response.as_ref().unwrap().return_amount,
                            spread_amount: self.expected_bid_amount_response.as_ref().unwrap().spread_amount,
                            commission_amount: self.expected_bid_amount_response.as_ref().unwrap().commission_amount
                        }));
                    }  
                }
                
                if self.expected_bid_amount2_request.clone() != None { 
                    let bid_amount2_req = check_mock_request(self.expected_bid_amount2_request.clone().unwrap());
                    if request == bid_amount2_req {
                        return Ok(to_binary(&MockedSimulation {
                            return_amount: self.expected_bid_amount2_response.as_ref().unwrap().return_amount,
                            spread_amount: self.expected_bid_amount2_response.as_ref().unwrap().spread_amount,
                            commission_amount: self.expected_bid_amount2_response.as_ref().unwrap().commission_amount
                        }));
                    } 
                }
                return Ok(to_binary(&request))
            }
        }
        // BIDS
        // sell token 2 for token 1
        // Test 1 => AMM Base Price = 11 && Expected False
        let mocked_deps = deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(11000000000000000000), // 11e18
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: None,
            expected_bid_amount_response: None,
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
        });
        
        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, false);

 
        // Test 2 => AMM Base Price = 9 && AMM Amount = 11 && Expected False
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(9000000000000000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1111111"}}}"#.as_bytes().to_vec()),
            expected_bid_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(13000000000000000000), //1.111111 * 11 (rounded)
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, false);
   
        // Test 3 => AMM Base Price = 8 && AMM Amount = 9 && Expected True
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(8000000000000000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1111111"}}}"#.as_bytes().to_vec()),
            expected_bid_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(10000000000000000000), // 1.1111*9 (rounded)
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, true);
  
        // Test 4 => AMM Base Price = 7 && AMM Amount = 11 && AMM Amount2 = 8 && Expected True
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(7000000000000000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1111111"}}}"#.as_bytes().to_vec()),
            expected_bid_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(13000000000000000000), // 1.1111*11 (rounded)  
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_bid_amount2_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"250000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount2_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(2000000000000000000), //0.25*(simulated_price) => 0.25*8
                spread_amount: Uint128(0),  
                commission_amount: Uint128(0)
            }), // 7*2 + 2 of spread
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, true);

       // Test 5 => AMM Base Price = 8 && AMM Amount = 10 && AMM Amount2 = 9 && Expected False
        let mut mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(8000000000000000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1111111"}}}"#.as_bytes().to_vec()),
            expected_bid_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(12000000000000000000), // 1.1111*10 (rounded) => 12
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_bid_amount2_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"250000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount2_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(2250000000000000000), //0.25*(simulated_price) => 0.25*9 = 2.25
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, false);
    }

    #[test]
    fn test_get_limit_order_to_trigger_ask() {
        let (init_result, mut deps) = init_helper(
            HumanAddr("factoryaddress".to_string()),
            "factoryhash".to_string(),
            "factorykey".to_string(),
            AssetInfo {
                is_native_token: false,
                decimal_places: 6,
                base_amount: Uint128(1000000),
                fee_amount: Uint128(500000),
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
                decimal_places: 18,
                base_amount: Uint128(1000000000000000000),
                fee_amount: Uint128(500000000000000000),
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

        use serde::{Deserialize, Serialize};
        #[derive(Debug)]
        struct MyMockQuerier {
            expected_bid_base_request: Vec<u8>,
            expected_bid_base_response: AmmPairSimulationResponse,
            expected_bid_amount_request: Option<Vec<u8>>,
            expected_bid_amount_response: Option<AmmPairSimulationResponse>,
            expected_bid_amount2_request: Option<Vec<u8>>,
            expected_bid_amount2_response: Option<AmmPairSimulationResponse>,
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct MockedSimulation {
            pub return_amount: Uint128,
            pub spread_amount: Uint128,
            pub commission_amount: Uint128
        }
        
        pub fn space_pad(message: &mut Vec<u8>, block_size: usize) -> &mut Vec<u8> {
            let len = message.len();
            let surplus = len % block_size;
            if surplus == 0 {
                return message;
            }
        
            let missing = block_size - surplus;
            message.reserve(missing);
            message.extend(std::iter::repeat(b' ').take(missing));
            message
        }

        pub fn check_mock_request(msg: Vec<u8>) -> Vec<u8>{
            let mut expected_msg = msg.clone();
            space_pad(&mut expected_msg, 256);
            let expected_request: QueryRequest<AmmSimulationQuery> =
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: HumanAddr("ammpairaddress".to_string()),
                callback_code_hash: "ammpairhash".to_string(),
                msg: Binary(expected_msg),
            });
            to_vec(&expected_request).unwrap()       
        }

        impl Querier for MyMockQuerier {
            fn raw_query(&self, request: &[u8] ) -> QuerierResult {
                let bid_base_req = check_mock_request(self.expected_bid_base_request.clone());
                if request == bid_base_req {
                    return Ok(to_binary(&MockedSimulation {
                        return_amount: self.expected_bid_base_response.return_amount,
                        spread_amount: self.expected_bid_base_response.spread_amount,
                        commission_amount: self.expected_bid_base_response.commission_amount
                    }));
                }
                if self.expected_bid_amount_request.clone() != None {
                    let bid_amount_req = check_mock_request(self.expected_bid_amount_request.clone().unwrap());
                    if request == bid_amount_req {
                        return Ok(to_binary(&MockedSimulation {
                            return_amount: self.expected_bid_amount_response.as_ref().unwrap().return_amount,
                            spread_amount: self.expected_bid_amount_response.as_ref().unwrap().spread_amount,
                            commission_amount: self.expected_bid_amount_response.as_ref().unwrap().commission_amount
                        }));
                    }  
                }
                
                if self.expected_bid_amount2_request.clone() != None { 
                    let bid_amount2_req = check_mock_request(self.expected_bid_amount2_request.clone().unwrap());
                    if request == bid_amount2_req {
                        return Ok(to_binary(&MockedSimulation {
                            return_amount: self.expected_bid_amount2_response.as_ref().unwrap().return_amount,
                            spread_amount: self.expected_bid_amount2_response.as_ref().unwrap().spread_amount,
                            commission_amount: self.expected_bid_amount2_response.as_ref().unwrap().commission_amount
                        }));
                    } 
                }
                
                return Ok(to_binary(&request))
            }
        }

        // SEND ASKS
        // sell token 1 for token 2
        // Charlie send
        let handle_msg = HandleMsg::Receive {
            sender: HumanAddr("token1address".to_string()), 
            from: HumanAddr("charlie".to_string()), 
            amount: Uint128(120000), //0.12 
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: false,
                price: Uint128(110000000000000000), //0.11
                expected_amount: Uint128(13200000000000000)  //0.0132 (0.12*0.11)
            }).unwrap())
        };

        let handle_result = handle(&mut deps, mock_env("token1address", &[]), handle_msg.clone());

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );        

        // Rob send
        let handle_msg = HandleMsg::Receive {
            sender: HumanAddr("token1address".to_string()), 
            from: HumanAddr("rob".to_string()), 
            amount: Uint128(20000), //0.02
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: false,
                price: Uint128(120000000000000000), //0.12
                expected_amount: Uint128(2400000000000000)  //0.0024 (0.12*0.02)
            }).unwrap())
        };

        let handle_result = handle(&mut deps, mock_env("token1address", &[]), handle_msg.clone());

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );   
   
        // ASK Test 1 => AMM Base Price = 0.1 && Expected False
        let mocked_deps = deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(100000000000000000), //0.1
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: None,
            expected_bid_amount_response: None,
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, false);

        // Test 2 => AMM Base Price = 0.11 && AMM Amount = 0.1 && Expected False
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request:  r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(110000000000000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"120000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(12000000000000000), // 0.12 * 0.1
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, false);
 
        // Test 3 => AMM Base Price = 0.12 && AMM Amount = 0.11 && Expected True     
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(120000000000000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"120000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(13200000000000000), // 0.12 * 0.11
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, true);

        // Test 4 => AMM Base Price = 0.13 && AMM Amount = 0.1 && AMM Amount2 = 0.12 && Expected True
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(130000000000000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"120000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(12000000000000000), // 0.12 * 0.1
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_bid_amount2_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"20000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount2_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(2400000000000000), // 0.02 * 0.12
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, true);
 
        // Test 5 => AMM Base Price = 0.13 && AMM Amount = 0.1 && AMM Amount2 = 0.11 && Expected False
        let mut mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(130000000000000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"120000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(12000000000000000), // 0.12 * 0.1
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_bid_amount2_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"20000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount2_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(2200000000000000), // 0.02 * 0.11
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, false);   
    }

    #[test]
    fn test_handle_receive_from_swap() {
        let (init_result, mut deps) = init_helper(
            HumanAddr("factoryaddress".to_string()),
            "factoryhash".to_string(),
            "factorykey".to_string(),
            AssetInfo {
                is_native_token: false,
                decimal_places: 6,
                base_amount: Uint128(1000000),
                fee_amount: Uint128(500000),
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
                decimal_places: 18,
                base_amount: Uint128(1000000000000000000),
                fee_amount: Uint128(500000000000000000),
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
            amount: Uint128(2000000),
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: false,
                price: Uint128(9000000000000000000),
                expected_amount: Uint128(9000000000000000000),
            }).unwrap())
        };

        let handle_result = handle(&mut deps, mock_env("token1address", &[]), handle_msg.clone());

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );

        let handle_msg = HandleMsg::Receive {
            sender: HumanAddr("token2address".to_string()), 
            from: HumanAddr("ammpairaddress".to_string()), 
            amount: Uint128(400000000000000),
            msg: None
        };

        let user_address_bob = &deps.api.canonical_address(&HumanAddr("bob".to_string())).unwrap();

        let order_id: HumanAddr = HumanAddr("bob".to_string());
        save(&mut deps.storage, SWAPPED_LIMIT_ORDER, &order_id);

        let handle_result = handle(&mut deps, mock_env("token2address", &[]), handle_msg.clone());
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        ); 
    }

    #[test]
    fn test_query_info_pair() {
        let (init_result, mut deps) = init_helper(
            HumanAddr("factoryaddress".to_string()),
            "factoryhash".to_string(),
            "factorykey".to_string(),
            AssetInfo {
                is_native_token: false,
                decimal_places: 6,
                base_amount: Uint128(1000000),
                fee_amount: Uint128(500000),
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
                decimal_places: 18,
                base_amount: Uint128(1000000000000000000),
                fee_amount: Uint128(500000000000000000),
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

        let query_msg = QueryMsg::OrderBookPairInfo {};
        let query_result = query(&deps, query_msg);

        let result:QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
        match result {
            QueryAnswer::OrderBookPair {amm_pair_address, assets_info} => {
                assert_eq!(amm_pair_address, HumanAddr("ammpairaddress".to_string()));
                assert_eq!(assets_info[0],AssetInfo {
                    is_native_token: false,
                    decimal_places: 6,
                    base_amount: Uint128(1000000),
                    fee_amount: Uint128(500000),
                    token: Some(
                        Token {
                            contract_addr: HumanAddr("token1address".to_string()),
                            token_code_hash: "token1hash".to_string()
                        }
                    ),
                    native_token: None
                });
                assert_eq!(assets_info[1],AssetInfo {
                    is_native_token: false,
                        decimal_places: 18,
                        base_amount: Uint128(1000000000000000000),
                        fee_amount: Uint128(500000000000000000),
                        token: Some(
                            Token {
                                contract_addr: HumanAddr("token2address".to_string()),
                                token_code_hash: "token2hash".to_string()
                            }
                        ),
                        native_token: None
                });
            },
            _ => assert_eq!(true,false)
        }
    }
}