use cosmwasm_std::{Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Decimal, Empty, Env, Extern, HandleResponse, HumanAddr, InitResponse, LogAttribute, Querier, QueryResult, StdError, StdResult, Storage, Uint128, WasmMsg, from_binary, to_binary};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::{snip20, storage::{AppendStore, AppendStoreMut}, utils::{HandleCallback, Query}};
use secret_toolkit::snip20::transfer_msg;
use crate::{msg::{AmmAssetInfo, AmmPairReverseSimulationResponse, AmmPairSimulationResponse, AmmSimulationOfferAsset, AmmSimulationQuery, AssetInfo, FactoryHandleMsg, FactoryQueryMsg, HandleAnswer, HandleMsg, InitMsg, IsKeyValidResponse, LimitOrderState, QueryAnswer, QueryMsg, ResponseStatus, Snip20Msg, UserOrderMap}, state::{load, may_load, remove, save}};
use crate::order_queues::OrderQueue;
pub const FACTORY_DATA: &[u8] = b"factory";
pub const AMM_PAIR_DATA: &[u8] = b"ammpair";
pub const TOKEN1_DATA: &[u8] = b"token1";
pub const TOKEN2_DATA: &[u8] = b"token2";
pub const HISTORY_LIMIT_ORDERS: &[u8] = b"historylimitorders";
pub const ACTIVE_LIMIT_ORDERS: &[u8] = b"activelimitorders";
pub const BID_ORDER_QUEUE: &[u8] = b"bidordequeue";
pub const ASK_ORDER_QUEUE: &[u8] = b"askorderqueue";
pub const SWAPPED_LIMIT_ORDER: &[u8] = b"swappedlimitorder";
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

    // NO NATIVE TOKENS FOR NOW
    if msg.token1_info.clone().token == None || msg.token2_info.clone().token == None {
        return Err(StdError::generic_err(
            "Native token not supported for now...",
        ));
    }

    token1_response = Some(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: msg.token1_info.token.clone().unwrap().contract_addr,
        callback_code_hash: msg.token1_info.token.clone().unwrap().token_code_hash,
        msg: snip20_register_msg.clone(),
        send: vec![],
    }));

    token2_response = Some(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: msg.token2_info.token.clone().unwrap().contract_addr,
        callback_code_hash: msg.token2_info.token.clone().unwrap().token_code_hash,
        msg: snip20_register_msg.clone(),
        send: vec![],
    }));
    
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
        HandleMsg::CancelLimitOrder {} => try_cancel_limit_order(deps, env), 
        HandleMsg::TriggerLimitOrders {} => try_trigger_limit_orders(deps, env), 
        _ => Err(StdError::generic_err("Handler not found!"))
    } 
}

pub fn try_receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    if msg != None {
        let msg: HandleMsg = from_binary(&msg.unwrap())?;

        if matches!(msg, HandleMsg::Receive { .. }) {
            return Err(StdError::generic_err(
                "Recursive call to receive() is not allowed",
            ));
        }
    
        if let HandleMsg::CreateLimitOrder {is_bid, price, expected_amount} = msg.clone() {
            let (deposit_token_index,balances,deposit_amount) = prepare_create_limit_order(deps,env.clone(),amount);
            if deposit_token_index == None {
                return Err(StdError::generic_err(format!(
                    "Invalid Token or Amount Sent < Minimum Amount"
                )));
            }
            return create_limit_order(deps, env.clone(), balances, deposit_token_index.unwrap(), deposit_amount, expected_amount, from, is_bid, price)
        } else {
                return Err(StdError::generic_err(format!(
                    "Receive handler not found!"
                )));
        }
    } else {
        let amm_pair_data = ReadonlyPrefixedStorage::new(AMM_PAIR_DATA, &deps.storage);
        let amm_pair_address: HumanAddr = load(&amm_pair_data, b"address").unwrap();
        if from == amm_pair_address {
            return swap_callback(deps, env.clone(), amount);
        } else {
            return Err(StdError::generic_err(format!(
                "Receive handler not found!"
            )));
        }
    }
    
}

fn prepare_create_limit_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount:Uint128
) -> (
    Option<i8>,
    Vec<Uint128>,
    Uint128
) {
    let token1_info: AssetInfo = load(&deps.storage, TOKEN1_DATA).unwrap();
    let token2_info: AssetInfo = load(&deps.storage, TOKEN2_DATA).unwrap();

    let mut deposit_token_index:Option<i8> = None;
    let mut balances = vec![Uint128(0), Uint128(0)];
    let deposit_amount: Uint128 = amount;

    if token1_info.token.unwrap().contract_addr == env.message.sender {
        balances[0] = amount;
        deposit_token_index = Some(0);
    };

    if token2_info.token.unwrap().contract_addr == env.message.sender {
        balances[1] = amount;
        deposit_token_index = Some(1);
    }

    return (deposit_token_index,balances,deposit_amount);
}

pub fn create_limit_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    balances: Vec<Uint128>,
    deposit_token_index: i8,
    deposit_amount: Uint128,
    expected_amount: Uint128,
    from: HumanAddr,
    is_bid: bool,
    price: Uint128
) -> StdResult<HandleResponse> {
    // Create new user limit order
    let user_address = deps.api.canonical_address(&from)?;

    // check if this user already has a limit order here
    let active_limit_orders = ReadonlyPrefixedStorage::new(ACTIVE_LIMIT_ORDERS, &deps.storage);
    let limit_order_data: Option<LimitOrderState> = may_load(&active_limit_orders, user_address.as_slice())?;
    if limit_order_data != None {
        return Err(StdError::generic_err(format!(
            "User already has a limit order for this pair. To create a new one withdraw the other one!"
        ))); 
    }

    // check if valid price and quantity
    if deposit_amount <= Uint128(0) || expected_amount <= Uint128(0) || price <= Uint128(0) {
        return Err(StdError::generic_err(format!(
            "Bad Amount or Price!"
        ))); 
    }

    // check if correct flag on is_bid!
    // is_bid = true => sell token 2 for token 1
    // is_bid = false =>  sell token 1 for token 2
    if (is_bid == true && balances[0] > Uint128(0)) || (is_bid == false && balances[1] > Uint128(0)) {
        return Err(StdError::generic_err(format!(
            "Incorrect is_bid flag! is_bid = true => sell token 2 for token 1 || is_bid = false =>  sell token 1 for token 2"
        ))); 
    }

    //Create Limit order
    let limit_order = LimitOrderState {
        is_bid,
        status: "Active".to_string(),
        price,
        deposit_token_index,
        deposit_amount,
        expected_amount,
        timestamp: env.block.time,
        balances,
        withdrew_balance: None
    };
    
    let mut key_store = PrefixedStorage::new(ACTIVE_LIMIT_ORDERS, &mut deps.storage);
    save(&mut key_store, &user_address.as_slice(), &limit_order)?;

    // Update Order Book
    if is_bid {
        let mut bid_order_book:OrderQueue = load(&deps.storage, BID_ORDER_QUEUE).unwrap();
        bid_order_book.insert(
            from.clone(),
            price,
            env.block.time
        );
        save(&mut deps.storage, BID_ORDER_QUEUE, &bid_order_book)?;
    } else {
        let mut ask_order_book:OrderQueue = load(&deps.storage, ASK_ORDER_QUEUE).unwrap();
        ask_order_book.insert(
            from.clone(),
            price,
            env.block.time
        );
        save(&mut deps.storage, ASK_ORDER_QUEUE, &ask_order_book)?;
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Status {
            status: ResponseStatus::Success,
            message: None,
        })?),
    })
}

pub fn swap_callback<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128
) -> StdResult<HandleResponse>{
    let order_id: HumanAddr = load(&mut deps.storage, SWAPPED_LIMIT_ORDER)?;
    let order_id_canonical = deps.api.canonical_address(&order_id)?;
    let token1_info: AssetInfo = load(&deps.storage, TOKEN1_DATA).unwrap();
    let token2_info: AssetInfo = load(&deps.storage, TOKEN2_DATA).unwrap();

    let mut active_limit_orders_data = PrefixedStorage::new(ACTIVE_LIMIT_ORDERS, &mut deps.storage);
    let limit_order_data: Option<LimitOrderState> = may_load(&active_limit_orders_data, &order_id_canonical.as_slice()).unwrap();
    if limit_order_data == None {
        return Err(StdError::generic_err(format!(
            "No limit order found."
        ))); 
    }

    // Calculate Fees and separate the amount the user needs to receive from the fees
    let user_amount: Uint128 = amount;
    
    // Transfer the amount received to the user
    let token_contract_address: HumanAddr;
    let token_contract_hash: String;

    if limit_order_data.clone().unwrap().is_bid {
        token_contract_address = token1_info.token.clone().unwrap().contract_addr;
        token_contract_hash = token1_info.token.clone().unwrap().token_code_hash;
    } else {
        token_contract_address = token2_info.token.clone().unwrap().contract_addr;
        token_contract_hash = token2_info.token.clone().unwrap().token_code_hash;
    }

    let mut transfer_result: CosmosMsg = transfer_msg(
        order_id.clone(),
        user_amount,
        None,
        BLOCK_SIZE,
        token_contract_hash,
        token_contract_address
    ).unwrap();

    // Get limit order from active and modify
    let mut modify_limit_order = limit_order_data.clone().unwrap();
    modify_limit_order.status = "Filled".to_string();
    modify_limit_order.balances = vec![Uint128(0),Uint128(0)];
    if modify_limit_order.is_bid == true {
        modify_limit_order.withdrew_balance = Some(vec![
            amount,
            Uint128(0)
        ]);
    } else {
        modify_limit_order.withdrew_balance = Some(vec![
            Uint128(0),
            amount
        ]);
    }

    // Remove from active limit order and queue
    remove(&mut active_limit_orders_data,&order_id_canonical.as_slice());

    if modify_limit_order.is_bid == true {
        let mut bid_order_book:OrderQueue = load(&mut deps.storage, BID_ORDER_QUEUE).unwrap();
        bid_order_book.remove(
            order_id.clone()
        );
        save(&mut deps.storage, BID_ORDER_QUEUE, &bid_order_book)?;
    } else {
        let mut ask_order_book:OrderQueue = load(&mut deps.storage, ASK_ORDER_QUEUE).unwrap();
        ask_order_book.remove(
            order_id.clone()
        );
        save(&mut deps.storage, ASK_ORDER_QUEUE, &ask_order_book)?;
    }

    // Add to History Limit Orders
    let mut history_limit_orders = PrefixedStorage::multilevel(&[HISTORY_LIMIT_ORDERS, &order_id_canonical.as_slice()], &mut deps.storage);
    let mut user_history_orders = AppendStoreMut::attach_or_create(&mut history_limit_orders)?;
    user_history_orders.push(&modify_limit_order)?;
        
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

pub fn try_cancel_limit_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env
) -> StdResult<HandleResponse>{
    // load limit order state of the user
    let user_address = &deps.api.canonical_address(&env.message.sender)?;

    let limit_orders_data = ReadonlyPrefixedStorage::new(ACTIVE_LIMIT_ORDERS, &deps.storage);
    let limit_order_data: Option<LimitOrderState> = may_load(&limit_orders_data, &user_address.as_slice())?;
    if limit_order_data == None {
        return Err(StdError::generic_err(format!(
            "No limit order found."
        ))); 
    }

    let mut transfer_result: CosmosMsg = CosmosMsg::Custom(Empty {});

    // send transfer from this contract to the token contract
    if limit_order_data.clone().unwrap().balances[0] > Uint128(0) {
        let token1_info: AssetInfo = load(&deps.storage, TOKEN1_DATA).unwrap();
        
        transfer_result = transfer_msg(
            env.message.sender.clone(),
            limit_order_data.clone().unwrap().balances[0],
            None,
            BLOCK_SIZE,
            token1_info.token.clone().unwrap().token_code_hash,
            token1_info.token.clone().unwrap().contract_addr
        ).unwrap();       
    }

    if limit_order_data.clone().unwrap().balances[1] > Uint128(0) {
        let token2_info: AssetInfo = load(&deps.storage, TOKEN2_DATA).unwrap();
        
        transfer_result = transfer_msg(
            env.message.sender.clone(),
            limit_order_data.clone().unwrap().balances[1],
            None,
            BLOCK_SIZE,
            token2_info.token.clone().unwrap().token_code_hash,
            token2_info.token.clone().unwrap().contract_addr
        ).unwrap();
    }

    // Add modified limit order to this user history and remove it from active
    let mut updated_limit_order: LimitOrderState = limit_order_data.clone().unwrap();
    updated_limit_order.status = "Canceled".to_string();
    updated_limit_order.withdrew_balance = Some(updated_limit_order.balances);
    updated_limit_order.balances = vec![Uint128(0),Uint128(0)];

    // Remove limit order from active
    let mut active_limit_orders_data = PrefixedStorage::new(ACTIVE_LIMIT_ORDERS, &mut deps.storage);
    remove(&mut active_limit_orders_data,user_address.as_slice());

    // Remove from queue
    if updated_limit_order.is_bid == true {
        let mut bid_order_book:OrderQueue = load(&mut deps.storage, BID_ORDER_QUEUE).unwrap();
        bid_order_book.remove(
            env.message.sender
        );
        save(&mut deps.storage, BID_ORDER_QUEUE, &bid_order_book)?;
    } else {
        let mut ask_order_book:OrderQueue = load(&mut deps.storage, ASK_ORDER_QUEUE).unwrap();
        ask_order_book.remove(
            env.message.sender
        );
        save(&mut deps.storage, ASK_ORDER_QUEUE, &ask_order_book)?;
    }

    // Add Order to history
    let mut history_limit_orders = PrefixedStorage::multilevel(&[HISTORY_LIMIT_ORDERS, user_address.as_slice()], &mut deps.storage);
    let mut user_history_orders = AppendStoreMut::attach_or_create(&mut history_limit_orders)?;
    user_history_orders.push(&updated_limit_order)?;
        
    // Response
    Ok(HandleResponse {
        messages: vec![
            transfer_result,
            //factory_response
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
    env: Env
) -> StdResult<HandleResponse>{
    // 1. Check Swappable Limit Orders Order Books
    let (order_id, limit_order_state) = get_limit_order_to_trigger(deps, true);
    if order_id != None {
        let token2_data:AssetInfo = load(&deps.storage, TOKEN2_DATA).unwrap();
        let amm_pair_data = ReadonlyPrefixedStorage::new(AMM_PAIR_DATA, &deps.storage);
        let amm_pair_address: HumanAddr = load(&amm_pair_data, b"address").unwrap();
    
        // Set the swapped limit order
        save(&mut deps.storage, SWAPPED_LIMIT_ORDER, &order_id.unwrap())?;

        let swap_response = snip20::send_msg(
            amm_pair_address, 
            limit_order_state.clone().unwrap().balances[1], 
            Some(Binary::from(r#"{ "swap": { } }"#.as_bytes())), 
            None, 
            256, 
            token2_data.clone().token.unwrap().token_code_hash, 
            token2_data.clone().token.unwrap().contract_addr
        ); 
        return Ok(HandleResponse {
            messages: vec![
                swap_response.unwrap()
            ],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::Status {
                status: ResponseStatus::Success,
                message: None,
            })?),
        })       
    }
    let (order_id, limit_order_state) = get_limit_order_to_trigger(deps, false);
    if order_id != None {
        let token1_data:AssetInfo = load(&deps.storage, TOKEN1_DATA).unwrap();
        let amm_pair_data = ReadonlyPrefixedStorage::new(AMM_PAIR_DATA, &deps.storage);
        let amm_pair_address: HumanAddr = load(&amm_pair_data, b"address").unwrap();
    
        // Set the swapped limit order
        save(&mut deps.storage, SWAPPED_LIMIT_ORDER, &order_id.unwrap())?;

        let swap_response = snip20::send_msg(
            amm_pair_address, 
            limit_order_state.clone().unwrap().balances[0], 
            Some(Binary::from(r#"{ "swap": { } }"#.as_bytes())), 
            None, 
            256, 
            token1_data.clone().token.unwrap().token_code_hash, 
            token1_data.clone().token.unwrap().contract_addr
        ); 
        return Ok(HandleResponse {
            messages: vec![
                swap_response.unwrap()
            ],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::Status {
                status: ResponseStatus::Success,
                message: None,
            })?),
        })   
    }

    return Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::OrderBookPairInfo {} => get_order_book_pair_info(deps),
        QueryMsg::GetActiveLimitOrder {user_address, user_viewkey} => get_active_limit_order(deps, user_address, user_viewkey),
        QueryMsg::GetHistoryLimitOrders {user_address, user_viewkey, page_size, page} => get_history_limit_orders(deps, user_address, user_viewkey, page_size, page),
        QueryMsg::CheckOrderBookTrigger {} => to_binary(&check_order_book_trigger(deps)?),
        _ => Err(StdError::generic_err("Handler not found!"))
    }
}


fn get_order_book_pair_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> QueryResult {
    let token1_data:AssetInfo = load(&deps.storage, TOKEN1_DATA).unwrap();
    let token2_data:AssetInfo = load(&deps.storage, TOKEN2_DATA).unwrap();
    let amm_pair_data = ReadonlyPrefixedStorage::new(AMM_PAIR_DATA, &deps.storage);
    let amm_pair_address: HumanAddr = load(&amm_pair_data, b"address").unwrap();

    to_binary(&QueryAnswer::OrderBookPair {
        amm_pair_address,
        assets_info: [token1_data,token2_data]
    })
}

fn get_active_limit_order<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    user_address: HumanAddr,
    user_viewkey: String
) -> QueryResult {
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
        let user_address_canonical = &deps.api.canonical_address(&user_address)?;

        let limit_orders_data = ReadonlyPrefixedStorage::new(ACTIVE_LIMIT_ORDERS, &deps.storage);
        let limit_order_data: Option<LimitOrderState> = may_load(&limit_orders_data, &user_address_canonical.as_slice())?;
        
        return to_binary(&QueryAnswer::ActiveLimitOrder {
            active_limit_order: limit_order_data 
        });
    } else {
        return Err(StdError::generic_err(format!(
            "Invalid address - viewkey pair!"
        ))); 
    }
}

fn get_history_limit_orders<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    user_address: HumanAddr,
    user_viewkey: String,
    page_size: Option<u32>,
    page: Option<u32>
) -> QueryResult {
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
        let history_limit_orders = ReadonlyPrefixedStorage::multilevel(&[HISTORY_LIMIT_ORDERS, user_address.as_slice()], &deps.storage);
        
        let store = if let Some(result) = AppendStore::<LimitOrderState, _>::attach(&history_limit_orders) {
            result?
        } else {
            return to_binary(&QueryAnswer::HistoryLimitOrders {
                history_limit_orders: vec![]
            });
        };

        let response:Vec<LimitOrderState>;
        if page_size != None && page != None {
            let tx_iter = store
            .iter()
            .skip((page.unwrap() * page_size.unwrap()) as _)
            .take(page_size.unwrap() as _);
    
            let txs: StdResult<Vec<LimitOrderState>> = tx_iter.collect();
            response = txs.unwrap()
        } else {
            let tx_iter = store.iter();
            let txs: StdResult<Vec<LimitOrderState>> = tx_iter.collect();
            response = txs.unwrap()
        }
    
        return to_binary(&QueryAnswer::HistoryLimitOrders {
            history_limit_orders: response
        });

    } else {
        return Err(StdError::generic_err(format!(
            "Invalid address - viewkey pair!"
        ))); 
    }
}

fn check_order_book_trigger<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<bool> {
        let (order_id, limit_order_state) = get_limit_order_to_trigger(deps, true);
        if order_id != None {
            return Ok(true)
        }
        let (order_id, limit_order_state) = get_limit_order_to_trigger(deps, false);
        if order_id != None {
            return Ok(true)
        }
        return Ok(false)
}


pub fn get_limit_order_to_trigger<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    is_bid: bool
) -> (Option<HumanAddr>, Option<LimitOrderState>) {
    let mut order_book: OrderQueue;
    let amm_pair_data = ReadonlyPrefixedStorage::new(AMM_PAIR_DATA, &deps.storage);
    let amm_pair_address: HumanAddr = load(&amm_pair_data, b"address").unwrap();
    let amm_pair_hash: String = load(&amm_pair_data, b"hash").unwrap();
    let limit_orders_data = ReadonlyPrefixedStorage::new(ACTIVE_LIMIT_ORDERS, &deps.storage);
    let token1_data:AssetInfo = load(&deps.storage, TOKEN1_DATA).unwrap();
    let token2_data:AssetInfo = load(&deps.storage, TOKEN2_DATA).unwrap();

    if is_bid {
        order_book = load(&deps.storage, BID_ORDER_QUEUE).unwrap();
    } else {
        order_book = load(&deps.storage, ASK_ORDER_QUEUE).unwrap();
    }
    
    let asset1:AmmAssetInfo = AmmAssetInfo::Token {
        contract_addr: token1_data.clone().token.unwrap().contract_addr,
        token_code_hash: token1_data.clone().token.unwrap().token_code_hash,
        viewing_key: "".to_string()
    };

    let asset2:AmmAssetInfo = AmmAssetInfo::Token {
        contract_addr: token2_data.clone().token.unwrap().contract_addr,
        token_code_hash: token2_data.clone().token.unwrap().token_code_hash,
        viewing_key: "".to_string()
    };

    // Simulate offering Token 1 with base unit of 1
    // Getting => X Token 2 per Token1 Price
    let response_amm_base_simulation: AmmPairSimulationResponse =
    AmmSimulationQuery::simulation {
        offer_asset: AmmSimulationOfferAsset{
            info: asset1.clone(),
            amount: token1_data.base_amount,
        }
    }.query(&deps.querier, amm_pair_hash.clone(), amm_pair_address.clone()).unwrap();

    for _ in 1..10 { // Max limit of 10 limit orders to check
        // Peek order, compare price of the limit order with the simulated one
        if let Some(order_book_peek) = order_book.peek() {
            let would_trigger_base_price: bool;
            if is_bid {
                would_trigger_base_price = order_book_peek.price >= response_amm_base_simulation.return_amount;
            } else {
                would_trigger_base_price = order_book_peek.price <= response_amm_base_simulation.return_amount;
            }
            
            if would_trigger_base_price {
                // Now we know that this order is a candidate to trigger but need to simulate again with his amount 
                // Simulate offering N amount of Token1
                // Getting => X Token2 per N Token1
                let order_book_id_canonical = &deps.api.canonical_address(&order_book_peek.id).unwrap();
                let limit_order_data: Option<LimitOrderState> = may_load(&limit_orders_data, order_book_id_canonical.as_slice()).unwrap();
                //let amount: Uint128 = limit_order_data.clone().unwrap().deposit_amount;
                let simulated: Uint128;
                let asset: AmmAssetInfo;

                if is_bid { asset = asset2.clone()} 
                else { asset = asset1.clone()} 

                // Here we have the final simulation for this with slippage
                // Check if deposited amount is <= simulated amount that comes from the swap
                let response_amm_order_simulation: AmmPairSimulationResponse =
                    AmmSimulationQuery::simulation {
                        offer_asset: AmmSimulationOfferAsset{
                            info: asset.clone(),
                            amount: limit_order_data.clone().unwrap().deposit_amount
                        }
                    }.query(&deps.querier, amm_pair_hash.clone(), amm_pair_address.clone()).unwrap();

                simulated = response_amm_order_simulation.return_amount;
                let would_trigger_total_amount = limit_order_data.clone().unwrap().expected_amount <= simulated;
               
                if would_trigger_total_amount {
                    //This order is elligible for a trigger so return it
                    return (Some(order_book_peek.clone().id), Some(limit_order_data.clone().unwrap()))
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

    return (None, None);
} 