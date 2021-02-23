#[cfg(test)]
mod tests {
    use cosmwasm_std::{CanonicalAddr, Binary, Coin, Decimal, Extern, HumanAddr, Querier, QuerierResult, QueryRequest, StdResult, Uint128, WasmQuery, from_binary, testing::*, to_vec};
    use crate::{contract::{BID_ORDER_QUEUE, SWAPPED_LIMIT_ORDER, FACTORY_DATA, LIMIT_ORDERS, TOKEN1_DATA, TOKEN2_DATA, handle}, msg::{OrderBookPairResponse, AmmAssetInfo, AssetInfo, HandleMsg, NativeToken, QueryMsg, Token}, state::{load, save, may_load}};
    use crate::contract::{init};
    use crate::order_queues::OrderQueue;
    use cosmwasm_std::{Api, InitResponse, to_binary};
    use crate::contract::query;
    use crate::{msg::{InitMsg, 
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
    }

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
            amount: Uint128(400000000000000),
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: true,
                price: Uint128(40)
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
            amount: Uint128(500000000000000),
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: true,
                price: Uint128(50)
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

        let limit_orders = ReadonlyPrefixedStorage::new(LIMIT_ORDERS,&deps.storage);
        let load_limit_order: Option<LimitOrderState> = may_load(&limit_orders, &user_address_bob.as_slice()).unwrap();

        assert_eq!(load_limit_order.clone().unwrap().is_bid, true);
        assert_eq!(load_limit_order.clone().unwrap().status, "Active".to_string());
        assert_eq!(load_limit_order.clone().unwrap().price, Uint128(40));
        assert_eq!(load_limit_order.clone().unwrap().balances, vec![Uint128(0),Uint128(400000000000000)]);

        // Check Alice limit order
        let user_address_alice = &deps.api.canonical_address(&HumanAddr("alice".to_string())).unwrap();

        let limit_orders = ReadonlyPrefixedStorage::new(LIMIT_ORDERS,&deps.storage);
        let load_limit_order: Option<LimitOrderState> = may_load(&limit_orders, &user_address_alice.as_slice()).unwrap();

        assert_eq!(load_limit_order.clone().unwrap().is_bid, true);
        assert_eq!(load_limit_order.clone().unwrap().status, "Active".to_string());
        assert_eq!(load_limit_order.clone().unwrap().price, Uint128(50));
        assert_eq!(load_limit_order.clone().unwrap().balances, vec![Uint128(0),Uint128(500000000000000)]);

        // Check order queue
        let mut bid_order_book:OrderQueue=load(&deps.storage, BID_ORDER_QUEUE).unwrap();
        assert_eq!(bid_order_book.peek().unwrap().id, user_address_alice.clone());
        assert_eq!(bid_order_book.peek().unwrap().price, Uint128(50));
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
        assert_eq!(load_limit_order.clone().unwrap().price, Uint128(40));
        assert_eq!(load_limit_order.clone().unwrap().balances, vec![Uint128(0),Uint128(400)]);

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
        assert_eq!(bid_order_book.peek().unwrap().price, Uint128(50));
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
            amount: Uint128(2000000000000000000),
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: true,
                price: Uint128(9000000)
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
            amount: Uint128(10000000000000000000),
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: true,
                price: Uint128(10000000)
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
            expected_ask_base_request: Vec<u8>,
            expected_ask_base_response: AmmPairSimulationResponse,
            expected_ask_amount_request: Option<Vec<u8>>,
            expected_ask_amount_response: Option<AmmPairSimulationResponse>,
            expected_ask_amount2_request: Option<Vec<u8>>,
            expected_ask_amount2_response: Option<AmmPairSimulationResponse>,
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
                 
                let ask_base_req = check_mock_request(self.expected_ask_base_request.clone());
                if request == ask_base_req {
                    return Ok(to_binary(&MockedSimulation {
                        return_amount: self.expected_ask_base_response.return_amount,
                        spread_amount: self.expected_ask_base_response.spread_amount,
                        commission_amount: self.expected_ask_base_response.commission_amount
                    }));
                }

                if self.expected_ask_amount_request.clone() != None {
                    let ask_amount_req = check_mock_request(self.expected_ask_amount_request.clone().unwrap());
                    if request == ask_amount_req {
                        return Ok(to_binary(&MockedSimulation {
                            return_amount: self.expected_ask_amount_response.as_ref().unwrap().return_amount,
                            spread_amount: self.expected_ask_amount_response.as_ref().unwrap().spread_amount,
                            commission_amount: self.expected_ask_amount_response.as_ref().unwrap().commission_amount
                        }));
                    }  
                }

                if self.expected_ask_amount2_request.clone() != None { 
                    let ask_amount2_req = check_mock_request(self.expected_ask_amount2_request.clone().unwrap());
                    if request == ask_amount2_req {
                        return Ok(to_binary(&MockedSimulation {
                            return_amount: self.expected_ask_amount2_response.as_ref().unwrap().return_amount,
                            spread_amount: self.expected_ask_amount2_response.as_ref().unwrap().spread_amount,
                            commission_amount: self.expected_ask_amount2_response.as_ref().unwrap().commission_amount
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
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"1000000000000000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(11000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: None,
            expected_bid_amount_response: None,
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
            expected_ask_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_ask_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(0),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_ask_amount_request: None,
            expected_ask_amount_response: None,
            expected_ask_amount2_request: None,
            expected_ask_amount2_response: None,
        });
        
        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, false);

 
        // Test 2 => AMM Base Price = 10 && AMM Amount = 11 && Expected False

        //amount = 7000000000000000000
        //return = 110000000000000000000
        //limit price  = 10000000000000000000

        // min amount - unit price
        // amount     - returned price
        
        // 7000000000000000000   -    110000000000000000000
        // 1000000000000000000    -    x
    
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"1000000000000000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(10000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"10000000000000000000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(110000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
            expected_ask_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_ask_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(0),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_ask_amount_request: None,
            expected_ask_amount_response: None,
            expected_ask_amount2_request: None,
            expected_ask_amount2_response: None,
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, false);
  
        // Test 3 => AMM Base Price = 9 && AMM Amount = 10 && Expected True
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"1000000000000000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(9000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"10000000000000000000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(100000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
            expected_ask_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_ask_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(0),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_ask_amount_request: None,
            expected_ask_amount_response: None,
            expected_ask_amount2_request: None,
            expected_ask_amount2_response: None,
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, true);
  
        // Test 4 => AMM Base Price = 7 && AMM Amount = 11 && AMM Amount2 = 8 && Expected True
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"1000000000000000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(7000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"10000000000000000000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(110000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_bid_amount2_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"2000000000000000000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount2_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(16000000), 
                spread_amount: Uint128(0),  
                commission_amount: Uint128(0)
            }), // 7*2 + 2 of spread
            expected_ask_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_ask_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(0),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_ask_amount_request: None,
            expected_ask_amount_response: None,
            expected_ask_amount2_request: None,
            expected_ask_amount2_response: None,
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, true);

       // Test 5 => AMM Base Price = 9 && AMM Amount = 11 && AMM Amount2 = 10 && Expected False
        let mut mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"1000000000000000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(9000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"10000000000000000000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(110000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_bid_amount2_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"2000000000000000000"}}}"#.as_bytes().to_vec()),
            expected_bid_amount2_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(20000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_ask_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_ask_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(0),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_ask_amount_request: None,
            expected_ask_amount_response: None,
            expected_ask_amount2_request: None,
            expected_ask_amount2_response: None,
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, false);
   
        // SEND ASKS
        // sell token 1 for token 2
        // Charlie send
        let handle_msg = HandleMsg::Receive {
            sender: HumanAddr("token1address".to_string()), 
            from: HumanAddr("charlie".to_string()), 
            amount: Uint128(110000), //0.11 
            msg: Some(to_binary(&HandleMsg::CreateLimitOrder {
                is_bid: false,
                price: Uint128(110000000000000000) //0.11 token 2
            }).unwrap())
        };

        let handle_result = handle(&mut mocked_deps, mock_env("token1address", &[]), handle_msg.clone());

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
                price: Uint128(120000000000000000) //0.12 token 2
            }).unwrap())
        };

        let handle_result = handle(&mut mocked_deps, mock_env("token1address", &[]), handle_msg.clone());

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );   

        // ASK Test 1 => AMM Base Price = 0.13 && Expected False
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"1000000000000000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(99999999999999999999999999999),
                spread_amount: Uint128(99999999999999999999999999999),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: None,
            expected_bid_amount_response: None,
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
            expected_ask_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_ask_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(130000000000000000),
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_ask_amount_request: None,
            expected_ask_amount_response: None,
            expected_ask_amount2_request: None,
            expected_ask_amount2_response: None,
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, false);

        // Test 2 => AMM Base Price = 0.12 && AMM Amount = 0.13 && Expected False
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request:  r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"1000000000000000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(9999999999999999999999999),
                spread_amount: Uint128(9999999999999999999999999),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: None,
            expected_bid_amount_response: None,
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
            expected_ask_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_ask_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(120000000000000000), //0.12
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_ask_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"20000"}}}"#.as_bytes().to_vec()),
            expected_ask_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(2600000000000000), //0.02 * 0.13 (base price * amm price simulated)
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_ask_amount2_request: None,
            expected_ask_amount2_response: None,
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, false);
 
        // Test 3 => AMM Base Price = 0.11 && AMM Amount = 0.12 && Expected True
        
        //base = 1
        //amount = 0.11 * 0.11
       
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"1000000000000000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(9999999999999999999999999),
                spread_amount: Uint128(9999999999999999999999999),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: None,
            expected_bid_amount_response: None,
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
            expected_ask_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_ask_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(110000000000000000), //0.11
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_ask_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"20000"}}}"#.as_bytes().to_vec()),
            expected_ask_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(2400000000000000), //0.02 * 0.12  (base price * amm price simulated)
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_ask_amount2_request: None,
            expected_ask_amount2_response: None,
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, true);

        // Test 4 => AMM Base Price = 0.11 && AMM Amount = 0.13 && AMM Amount2 = 0.11 && Expected True
        let mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"1000000000000000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(9999999999999999999999999),
                spread_amount: Uint128(9999999999999999999999999),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: None,
            expected_bid_amount_response: None,
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
            expected_ask_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_ask_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(110000000000000000), //0.11
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_ask_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"20000"}}}"#.as_bytes().to_vec()),
            expected_ask_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(2600000000000000), //0.02 * 0.13 (base price * amm price simulated)
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_ask_amount2_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"110000"}}}"#.as_bytes().to_vec()),
            expected_ask_amount2_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(12100000000000000), //0.11 * 0.11 (base price * amm price simulated)
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
        });

        let query_msg = QueryMsg::CheckOrderBookTrigger {};
        let query_result = query(&mocked_deps, query_msg);

        let needs_trigger:bool = from_binary(&query_result.unwrap()).unwrap();

        assert_eq!(needs_trigger, true);
 
        // Test 5 => AMM Base Price = 0.11 && AMM Amount = 0.13 && AMM Amount2 = 0.12 && Expected False
        let mut mocked_deps = mocked_deps.change_querier(|_| MyMockQuerier {
            expected_bid_base_request:  r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token2address","token_code_hash":"token2hash","viewing_key":""}},"amount":"1000000000000000000"}}}"#.as_bytes().to_vec(),
            expected_bid_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(9999999999999999999999999),
                spread_amount: Uint128(9999999999999999999999999),
                commission_amount: Uint128(0)
            },
            expected_bid_amount_request: None,
            expected_bid_amount_response: None,
            expected_bid_amount2_request: None,
            expected_bid_amount2_response: None,
            expected_ask_base_request: r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"1000000"}}}"#.as_bytes().to_vec(),
            expected_ask_base_response: AmmPairSimulationResponse {
                return_amount: Uint128(110000000000000000), //0.11
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            },
            expected_ask_amount_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"20000"}}}"#.as_bytes().to_vec()),
            expected_ask_amount_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(2600000000000000), //0.02 * 0.13 (base price * amm price simulated)
                spread_amount: Uint128(0),
                commission_amount: Uint128(0)
            }),
            expected_ask_amount2_request: Some(r#"{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"token1address","token_code_hash":"token1hash","viewing_key":""}},"amount":"110000"}}}"#.as_bytes().to_vec()),
            expected_ask_amount2_response: Some(AmmPairSimulationResponse {
                return_amount: Uint128(13200000000000000), //0.11 * 0.12 (base price * amm price simulated)
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
                price: Uint128(9000000000000000000)
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

        let order_id_can: CanonicalAddr = deps.api.canonical_address(&HumanAddr("bob".to_string())).unwrap();
        save(&mut deps.storage, SWAPPED_LIMIT_ORDER, &order_id_can);

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

        let query_msg = QueryMsg::OrderBookPairInfo {};
        let query_result = query(&deps, query_msg);

        let result:OrderBookPairResponse = from_binary(&query_result.unwrap()).unwrap();
        assert_eq!(result.amm_pair_address, HumanAddr("ammpairaddress".to_string()));
        assert_eq!(result.assets_info[0],AssetInfo {
            is_native_token: true,
            decimal_places: 6,
            base_amount: Uint128(1000000),
            token: None,
            native_token: Some(
                NativeToken{denom:"uscrt".to_string()}
            )
        });
        assert_eq!(result.assets_info[1],AssetInfo {
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
    }
}