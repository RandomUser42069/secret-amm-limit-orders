use cosmwasm_std::{Api, BankMsg, Binary, Coin, CosmosMsg, Empty, Env, Extern, HandleResponse, HumanAddr, InitResponse, LogAttribute, Querier, StdError, StdResult, Storage, Uint128, WasmMsg, from_binary, to_binary};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::{utils::{HandleCallback, Query}};
use secret_toolkit::snip20::transfer_msg;
use crate::{msg::{AmmAssetInfo, AmmFactoryPairResponse, AmmFactoryQueryMsg, AmmPairSimulationResponse, AmmSimulationOfferAsset, AmmSimulationQuery, AssetInfo, FactoryHandleMsg, FactoryQueryMsg, HandleAnswer, HandleMsg, InitMsg, IsKeyValidResponse, LimitOrderState, QueryMsg, ResponseStatus, Snip20Msg}, state::{load, may_load, remove, save}};
use crate::order_queues::OrderQueue;
pub const FACTORY_DATA: &[u8] = b"factory";
pub const AMM_PAIR_DATA: &[u8] = b"ammpair";
pub const TOKEN1_DATA: &[u8] = b"token1";
pub const TOKEN2_DATA: &[u8] = b"token2";
pub const LIMIT_ORDERS: &[u8] = b"limitorders";
pub const BID_ORDER_QUEUE: &[u8] = b"bidordequeue";
pub const ASK_ORDER_QUEUE: &[u8] = b"askorderqueue";
pub const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let mut factory_data = PrefixedStorage::new(FACTORY_DATA, &mut deps.storage);
    save(&mut factory_data, b"address", &msg.factory_address)?;
    save(&mut factory_data, b"hash", &msg.factory_hash)?;
    save(&mut factory_data, b"key", &msg.factory_key)?;

    save(&mut deps.storage, TOKEN1_DATA, &msg.token1_info)?;
    save(&mut deps.storage, TOKEN2_DATA, &msg.token2_info)?;

    save(&mut deps.storage, BID_ORDER_QUEUE, &OrderQueue::new(true))?;
    save(&mut deps.storage, ASK_ORDER_QUEUE, &OrderQueue::new(false))?;

    let mut amm_pair_data = PrefixedStorage::new(AMM_PAIR_DATA, &mut deps.storage);
    save(&mut amm_pair_data, b"address", &msg.amm_pair_contract_address)?;
    save(&mut amm_pair_data, b"hash", &msg.amm_pair_contract_hash)?;
    
    // send register to snip20
    let snip20_register_msg = to_binary(&Snip20Msg::register_receive(env.contract_code_hash))?;
    
    let mut token1_response: Option<CosmosMsg> = None;
    let mut token2_response: Option<CosmosMsg> = None;

    if !msg.token1_info.clone().is_native_token {
        token1_response = Some(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: msg.token1_info.token.clone().unwrap().contract_addr,
            callback_code_hash: msg.token1_info.token.clone().unwrap().token_code_hash,
            msg: snip20_register_msg.clone(),
            send: vec![],
        }));
    }

    if !msg.token2_info.clone().is_native_token {
        token2_response = Some(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: msg.token2_info.token.clone().unwrap().contract_addr,
            callback_code_hash: msg.token2_info.token.clone().unwrap().token_code_hash,
            msg: snip20_register_msg.clone(),
            send: vec![],
        }));
    }
    
    // send callback to factory
    let callback_msg = FactoryHandleMsg::InitCallBackFromSecretOrderBookToFactory {
        auth_key: msg.factory_key.clone(),
        contract_address: env.contract.address,
        amm_pair_address: msg.amm_pair_contract_address,
        token1_info: msg.token1_info.clone(),
        token2_info: msg.token2_info.clone(),
    };

    let cosmos_msg = callback_msg.to_cosmos_msg(msg.factory_hash.clone(), msg.factory_address.clone(), None)?;

    let mut messages:Vec<CosmosMsg> = vec![cosmos_msg];
    if token2_response != None {messages.insert(0, token2_response.unwrap())}
    if token1_response != None {messages.insert(0, token1_response.unwrap())}

    Ok(InitResponse {
        messages,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        // Receiver to CreateLimitOrder from SNIP20
        HandleMsg::Receive { sender, from, amount, msg } => try_receive(deps, env, sender, from, amount, msg),
        // Receiver to CreateLimitOrder from SCRT
        HandleMsg::ReceiveNativeToken {is_bid, price} => try_receive_native_token(deps, env, is_bid, price),
        HandleMsg::WithdrawLimitOrder {} => try_withdraw_limit_order(deps, env), 
        HandleMsg::TriggerLimitOrders {test_amm_price} => try_trigger_limit_orders(deps, env, test_amm_price), 
        _ => Err(StdError::generic_err("Handler not found!"))
    } 
}

pub fn try_receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    from: HumanAddr,
    amount: Uint128,
    msg: Binary,
) -> StdResult<HandleResponse> {
    let msg: HandleMsg = from_binary(&msg)?;

    if matches!(msg, HandleMsg::Receive { .. }) {
        return Err(StdError::generic_err(
            "Recursive call to receive() is not allowed",
        ));
    }

    if let HandleMsg::CreateLimitOrder {is_bid, price} = msg.clone() {
        let (order_token_index,balances,order_token_init_quant) = prepare_create_limit_order(deps,env.clone(),false,amount);
        if order_token_index == None {
            return Err(StdError::generic_err(format!(
                "Invalid Token or Amount Sent < Minimum Amount"
            )));
        }
        return create_limit_order(deps, env.clone(), balances, order_token_index.unwrap(), order_token_init_quant, from, is_bid, price)
    } else {
        return Err(StdError::generic_err(format!(
            "Receive handler not found!"
        )));
    }
}

pub fn try_receive_native_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    is_bid:bool,
    price: Uint128
) -> StdResult<HandleResponse> { 
    let (order_token_index,balances,order_token_init_quant) = prepare_create_limit_order(deps,env.clone(),true, env.clone().message.sent_funds[0].amount);
    if order_token_index == None {
        return Err(StdError::generic_err(format!(
            "Invalid Token or Amount Sent < Minimum Amount"
        )));
    }
    return create_limit_order(deps, env.clone(), balances, order_token_index.unwrap(), order_token_init_quant, env.clone().message.sender, is_bid, price)
}

fn prepare_create_limit_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    is_native_token: bool,
    amount:Uint128,
) -> (
    Option<i8>,
    Vec<Uint128>,
    Uint128
) {
    let token1_info: AssetInfo = load(&deps.storage, TOKEN1_DATA).unwrap();
    let token2_info: AssetInfo = load(&deps.storage, TOKEN2_DATA).unwrap();

    let mut order_token_index:Option<i8> = None;
    let mut balances = vec![Uint128(0), Uint128(0)];
    let order_token_init_quant: Uint128 = amount;

    match token1_info.is_native_token {
        true => {
            if is_native_token && amount >= token1_info.min_order_amount {
                balances[0] = amount;
                order_token_index = Some(0);
            }
        },
        false => {
            if !is_native_token && token1_info.token.unwrap().contract_addr == env.message.sender && amount >= token1_info.min_order_amount{
                balances[0] = amount;
                order_token_index = Some(0);
            }
        }
    }

    match token2_info.is_native_token {
        true => {
            if is_native_token && amount >= token2_info.min_order_amount {
                balances[1] = amount;
                order_token_index = Some(1);
            }
        },
        false => {
            if !is_native_token && token2_info.token.unwrap().contract_addr == env.message.sender && amount >= token2_info.min_order_amount{
                balances[1] = amount;
                order_token_index = Some(1);
            }
        }
    }

    return (order_token_index,balances,order_token_init_quant);
}

pub fn create_limit_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    balances: Vec<Uint128>,
    order_token_index: i8,
    order_token_init_quant: Uint128,
    from: HumanAddr,
    is_bid: bool,
    price: Uint128
) -> StdResult<HandleResponse> {
    // Create new user limit order
    let user_address = &deps.api.canonical_address(&from)?;

    let limit_order = LimitOrderState {
        is_bid,
        status: "Active".to_string(),
        price,
        order_token_index,
        order_token_init_quant,
        timestamp: env.block.time,
        balances
    };
    let mut key_store = PrefixedStorage::new(LIMIT_ORDERS, &mut deps.storage);
    save(&mut key_store, user_address.as_slice(), &limit_order)?;

    // Update Order Book
    if is_bid {
        let mut bid_order_book:OrderQueue = load(&deps.storage, BID_ORDER_QUEUE).unwrap();
        bid_order_book.insert(
            user_address.clone(),
            price,
            env.block.time
        );
        save(&mut deps.storage, BID_ORDER_QUEUE, &bid_order_book)?;
    } else {
        let mut ask_order_book:OrderQueue = load(&deps.storage, ASK_ORDER_QUEUE).unwrap();
        ask_order_book.insert(
            user_address.clone(),
            price,
            env.block.time
        );
        save(&mut deps.storage, ASK_ORDER_QUEUE, &ask_order_book)?;
    }

    Ok(HandleResponse::default())
}

pub fn try_withdraw_limit_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env
) -> StdResult<HandleResponse>{
    // load limit order state of the user
    let user_address = &deps.api.canonical_address(&env.message.sender)?;
    let limit_orders_data = ReadonlyPrefixedStorage::new(LIMIT_ORDERS, &deps.storage);
    let limit_order_data: Option<LimitOrderState> = may_load(&limit_orders_data, user_address.as_slice())?;
    if limit_order_data == None {
        return Err(StdError::generic_err(format!(
            "No limit order found for this user."
        ))); 
    }

    let mut transfer_result: CosmosMsg = CosmosMsg::Custom(Empty {});

    // send transfer from this contract to the token contract
    if limit_order_data.clone().unwrap().balances[0] > Uint128(0) {
        let token1_info: AssetInfo = load(&deps.storage, TOKEN1_DATA).unwrap();
        match token1_info.is_native_token {
            true => {
                transfer_result = CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: env.message.sender.clone(),
                    amount: vec![Coin {
                        denom: token1_info.native_token.unwrap().denom,
                        amount: limit_order_data.clone().unwrap().balances[0],
                    }],
                });
            }
            false => {
                transfer_result = transfer_msg(
                    env.message.sender.clone(),
                    limit_order_data.clone().unwrap().balances[0],
                    None,
                    BLOCK_SIZE,
                    token1_info.token.clone().unwrap().token_code_hash,
                    token1_info.token.clone().unwrap().contract_addr
                ).unwrap();
            }
        }        
    }

    if limit_order_data.clone().unwrap().balances[1] > Uint128(0) {
        let token2_info: AssetInfo = load(&deps.storage, TOKEN2_DATA).unwrap();
        match token2_info.is_native_token {
            true => {
                transfer_result = CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: env.message.sender.clone(),
                    amount: vec![Coin {
                        denom: token2_info.native_token.unwrap().denom,
                        amount: limit_order_data.clone().unwrap().balances[1],
                    }],
                });
            }
            false => {
                transfer_result = transfer_msg(
                    env.message.sender.clone(),
                    limit_order_data.clone().unwrap().balances[1],
                    None,
                    BLOCK_SIZE,
                    token2_info.token.clone().unwrap().token_code_hash,
                    token2_info.token.clone().unwrap().contract_addr
                ).unwrap();
            }
        }      
    }
    // remove the limit order 
    let mut limit_orders_data = PrefixedStorage::new(LIMIT_ORDERS, &mut deps.storage);
    remove(&mut limit_orders_data, user_address.as_slice());
    // remove the order on the queue
    let mut bid_order_book:OrderQueue = load(&deps.storage, BID_ORDER_QUEUE).unwrap();
    bid_order_book.remove(
        user_address.clone()
    );
    save(&mut deps.storage, BID_ORDER_QUEUE, &bid_order_book)?;
    // Response
    Ok(HandleResponse {
        messages: vec![
            transfer_result
        ],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Status {
            status: ResponseStatus::Success,
            message: None,
        })?),
    })
}

pub fn try_trigger_limit_orders<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    test_amm_price: Uint128
) -> StdResult<HandleResponse>{
    // 1. get AMM price for this pair
    let amm_price = test_amm_price;
    // 2. Bid Order Book
    execute_trigger(deps, env.clone(), amm_price, true);
    // 3. Ask Order Book
    execute_trigger(deps, env.clone(), amm_price, false);
   
    /*return Err(StdError::generic_err(format!(
        "{:?},{}",
        triggered_limit_orders,
        order_total_quantity
    )));*/

    Ok(HandleResponse::default())
}

pub fn execute_trigger<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amm_price: Uint128,
    is_bid: bool
) -> StdResult<bool> {
    // 2. init loop
    let limit_orders_data = ReadonlyPrefixedStorage::new(LIMIT_ORDERS, &deps.storage);
    let mut order_book:OrderQueue;
    if is_bid {
        order_book = load(&deps.storage, BID_ORDER_QUEUE).unwrap();
    } else {
        order_book = load(&deps.storage, ASK_ORDER_QUEUE).unwrap();
    }
    let mut triggered_limit_orders = vec![];
    let mut order_total_quantity:Uint128 = Uint128(0);
    loop {
        // 2.1. Peek secret order book
        let peek_bid = order_book.peek();
        if peek_bid == None {break;}
        // 2.2. if peek price >= AMM price => pop and save it to variable
        let compare_prices: bool;
        if is_bid { compare_prices = peek_bid.unwrap().price >= amm_price }
        else { compare_prices = peek_bid.unwrap().price <= amm_price }

        if compare_prices {
            let triggered_order = order_book.pop().unwrap();
            let limit_order_data:Option<LimitOrderState> = may_load(&limit_orders_data, triggered_order.id.as_slice())?;
            if limit_order_data != None {
                order_total_quantity = order_total_quantity + limit_order_data.clone().unwrap().balances[limit_order_data.clone().unwrap().order_token_index as usize];
                triggered_limit_orders.push(triggered_order.id);
            }
        } else {
            // 2.3. end loop when peek price fails to be > AMM price
            break;
        }
    };

    // 3. Send a single transaction to swap with all the peeked quantity on the variable

    
    // 4. If success, update all the limit orders triggered balance and status
    let mut limit_orders_data = PrefixedStorage::new(LIMIT_ORDERS, &mut deps.storage);
    for limit_order_id in &triggered_limit_orders {
        let mut limit_order_data:LimitOrderState = load(&limit_orders_data, limit_order_id.as_slice()).unwrap();
        limit_order_data.status = "Filled".to_string();
        limit_order_data.balances = vec![
            limit_order_data.balances[1], 
            limit_order_data.balances[0]
        ];
        save(&mut limit_orders_data, limit_order_id.as_slice(), &limit_order_data)?;
    }
    Ok(true)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::OrderBookPairInfo {} => to_binary(&get_order_book_pair_info(deps)?),
        QueryMsg::GetLimitOrder {user_address, user_viewkey} => to_binary(&get_limit_order(deps, user_address, user_viewkey)?),
        QueryMsg::CheckOrderBookTrigger {} => to_binary(&check_order_book_trigger(deps)?),
        _ => Err(StdError::generic_err("Handler not found!"))
    }
}

fn get_order_book_pair_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<[AssetInfo;2]> {
    let token1_data:AssetInfo = load(&deps.storage, TOKEN1_DATA).unwrap();
    let token2_data:AssetInfo = load(&deps.storage, TOKEN2_DATA).unwrap();

    return Ok([token1_data,token2_data]);
}

fn get_limit_order<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    user_address: HumanAddr,
    user_viewkey: String
) -> StdResult<Option<LimitOrderState>> {
    let factory_data = ReadonlyPrefixedStorage::new(FACTORY_DATA, &deps.storage);
    let factory_contract_address: HumanAddr = load(&factory_data, b"address")?;
    let factory_contract_hash: String = load(&factory_data, b"hash")?;
    let factory_key: String = load(&factory_data, b"key")?;

    let response: IsKeyValidResponse =
    FactoryQueryMsg::IsKeyValid {
        factory_key,
        viewing_key: user_viewkey.clone(),
        address: user_address.clone()
    }.query(&deps.querier, factory_contract_hash, factory_contract_address)?;

    if response.is_key_valid.is_valid {
        let user_address = &deps.api.canonical_address(&user_address)?;
        let limit_orders_data = ReadonlyPrefixedStorage::new(LIMIT_ORDERS, &deps.storage);
        let limit_order_data:Option<LimitOrderState> = may_load(&limit_orders_data, user_address.as_slice())?;
        if let Some(limit_order_data) = limit_order_data {
            return Ok(Some(limit_order_data))
        } else {
            return Ok(None)
        }
    } else {
        return Err(StdError::generic_err(format!(
            "Invalid address - viewkey pair!"
        ))); 
    }
}

fn check_order_book_trigger<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<bool> {

        let bid_to_trigger = get_limit_order_to_trigger(deps, true);
        if bid_to_trigger != None {
            return Ok(true)
        }
        let ask_to_trigger = get_limit_order_to_trigger(deps, false);
        if ask_to_trigger != None {
            return Ok(true)
        }

        return Ok(false)
}


pub fn get_limit_order_to_trigger<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    is_bid: bool
) -> Option<LimitOrderState> {
    let mut order_book: OrderQueue;
    let token_data: AssetInfo;
    let amm_pair_data = ReadonlyPrefixedStorage::new(AMM_PAIR_DATA, &deps.storage);
    let amm_pair_address: HumanAddr = load(&amm_pair_data, b"address").unwrap();
    let amm_pair_hash: String = load(&amm_pair_data, b"hash").unwrap();
    let limit_orders_data = ReadonlyPrefixedStorage::new(LIMIT_ORDERS, &deps.storage);

    if is_bid {
        order_book = load(&deps.storage, BID_ORDER_QUEUE).unwrap();
        token_data = load(&deps.storage, TOKEN1_DATA).unwrap();
    } else {
        order_book = load(&deps.storage, ASK_ORDER_QUEUE).unwrap();
        token_data = load(&deps.storage, TOKEN2_DATA).unwrap();
    }
    
    let asset:AmmAssetInfo = 
    match token_data.is_native_token {
        true => AmmAssetInfo::NativeToken {
            denom: token_data.native_token.unwrap().denom
        },
        false => AmmAssetInfo::Token {
            contract_addr: token_data.clone().token.unwrap().contract_addr,
            token_code_hash: token_data.clone().token.unwrap().token_code_hash,
            viewing_key: "".to_string()
        }
    };

    // Get Base Price => Lowest Amount possible
    let response_amm_base_simulation: AmmPairSimulationResponse =
    AmmSimulationQuery::simulation {
        offer_asset: AmmSimulationOfferAsset{
            info: asset.clone(),
            amount: token_data.min_order_amount
        }
    }.query(&deps.querier, amm_pair_hash.clone(), amm_pair_address.clone()).unwrap();

    for _ in 1..10 { // Max limit of 10 limit orders to check
        // Peek order, compare order price with base price + spread
        if let Some(order_book_peek) = order_book.peek() {
            let would_trigger_base_price: bool;
            if is_bid {
                would_trigger_base_price = order_book_peek.price >= response_amm_base_simulation.return_amount + response_amm_base_simulation.spread_amount
            } else {
                would_trigger_base_price = order_book_peek.price <= response_amm_base_simulation.return_amount + response_amm_base_simulation.spread_amount
            }

            if would_trigger_base_price {
                // Now we know that this order is a candidate to trigger but need to simulate again with his amount 
                let limit_order_data: Option<LimitOrderState> = may_load(&limit_orders_data, order_book_peek.id.as_slice()).unwrap();
               
                let response_amm_order_simulation: AmmPairSimulationResponse =
                AmmSimulationQuery::simulation {
                    offer_asset: AmmSimulationOfferAsset{
                        info: asset.clone(),
                        amount: limit_order_data.clone().unwrap().balances[limit_order_data.clone().unwrap().order_token_index as usize]
                    }
                }.query(&deps.querier, amm_pair_hash.clone(), amm_pair_address.clone()).unwrap();
                
                // Here we have the final simulation for this, and check if it can be triggered with 0 slippage
                let would_trigger_total_amount: bool;
                if is_bid {
                    would_trigger_total_amount = order_book_peek.price >= response_amm_order_simulation.return_amount + response_amm_order_simulation.spread_amount
                } else {
                    would_trigger_total_amount = order_book_peek.price <= response_amm_order_simulation.return_amount + response_amm_order_simulation.spread_amount
                }

                if would_trigger_total_amount {
                    //This order is elligible for a trigger so return it
                    return Some(limit_order_data.clone().unwrap())
                } else {
                    // pop current order from the orderbook as it's not elligible for triggering
                    order_book.pop();
                    // continues the loop as lower amount limit orders can be on the order book
                }
            } else {
                // Breaks the cycle because the orderbook is ordered and if the base price would not trigger this order, the others would not trigger too
                break;
            }
        }
    }

    return None;
}