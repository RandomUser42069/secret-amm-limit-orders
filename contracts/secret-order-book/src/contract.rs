use cosmwasm_std::{Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, StdError, StdResult, Storage, Uint128, WasmMsg, from_binary, to_binary};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::{utils::{HandleCallback, Query}};
use secret_toolkit::snip20::transfer_msg;
use crate::{msg::{FactoryHandleMsg, FactoryQueryMsg, HandleMsg, InitMsg, IsKeyValidResponse, 
    LimitOrderState, 
    QueryMsg, Snip20Msg}, state::{load, may_load, remove, save}};
use crate::order_queues::OrderQueue;
pub const FACTORY_DATA: &[u8] = b"factory"; // address, hash, key
pub const TOKEN1_DATA: &[u8] = b"token1"; // address, hash
pub const TOKEN2_DATA: &[u8] = b"token2"; // address, hash
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

    let mut token1_data = PrefixedStorage::new(TOKEN1_DATA, &mut deps.storage);
    save(&mut token1_data, b"address", &msg.token1_code_address)?;
    save(&mut token1_data, b"hash", &msg.token1_code_hash)?;

    let mut token2_data = PrefixedStorage::new(TOKEN2_DATA, &mut deps.storage);
    save(&mut token2_data, b"address", &msg.token2_code_address)?;
    save(&mut token2_data, b"hash", &msg.token2_code_hash)?;

    save(&mut deps.storage, BID_ORDER_QUEUE, &OrderQueue::new(true))?;
    save(&mut deps.storage, ASK_ORDER_QUEUE, &OrderQueue::new(false))?;

    // send register to snip20
    let snip20_register_msg = to_binary(&Snip20Msg::register_receive(env.contract_code_hash))?;
    let token1_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: msg.token1_code_address.clone(),
        callback_code_hash: msg.token1_code_hash,
        msg: snip20_register_msg.clone(),
        send: vec![],
    });
    let token2_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: msg.token2_code_address.clone(),
        callback_code_hash: msg.token2_code_hash,
        msg: snip20_register_msg.clone(),
        send: vec![],
    });
    
    // send callback to factory
    let callback_msg = FactoryHandleMsg::InitCallBackFromSecretOrderBookToFactory {
        auth_key: msg.factory_key.clone(),
        contract_address: env.contract.address,
        token1_address: msg.token1_code_address.clone(),
        token2_address: msg.token2_code_address.clone(),
    };

    let cosmos_msg = callback_msg.to_cosmos_msg(msg.factory_hash.clone(), msg.factory_address.clone(), None)?;

    Ok(InitResponse {
        messages: vec![
            token1_msg,
            token2_msg,
            cosmos_msg,
        ],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        // Receiver to CreateLimitOrder
        HandleMsg::Receive { sender, from, amount, msg } => try_receive(deps, env, sender, from, amount, msg),
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

    let token1_data = ReadonlyPrefixedStorage::new(TOKEN1_DATA, &deps.storage);
    let token2_data = ReadonlyPrefixedStorage::new(TOKEN2_DATA, &deps.storage);
    let load_token1_address: HumanAddr = load(&token1_data, b"address")?;
    let load_token2_address: HumanAddr = load(&token2_data, b"address")?;

    let mut balances = vec![Uint128(0), Uint128(0)];
    let order_token_index: usize;
    let order_token_init_quant: Uint128 = amount;

    if load_token1_address == env.message.sender {
        balances[0] = amount;
        order_token_index = 0;
    } else if load_token2_address == env.message.sender { 
        balances[1] = amount;
        order_token_index = 1;
    } else {
        return Err(StdError::generic_err(format!(
            "{} is not a known SNIP-20 coin that this contract registered to",
            env.message.sender
        )));
    }
    
    if let HandleMsg::CreateLimitOrder {is_bid, price} = msg.clone() {
        return create_limit_order(deps, env, balances, order_token_index, order_token_init_quant,from, is_bid, price)
    } else {
        return Err(StdError::generic_err(format!(
            "Receive handler not found!"
        )));
    }
}

pub fn create_limit_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    balances: Vec<Uint128>,
    order_token_index: usize,
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
    // send transfer from this contract to the token contract
    if limit_order_data.clone().unwrap().balances[0] > Uint128(0) {
        let token1_data = ReadonlyPrefixedStorage::new(TOKEN1_DATA, &deps.storage);
        let load_token1_address: HumanAddr = load(&token1_data, b"address")?;
        let load_token1_hash: String = load(&token1_data, b"hash")?;

        let _transfer_result: StdResult<CosmosMsg> = transfer_msg(
            env.message.sender.clone(),
            limit_order_data.clone().unwrap().balances[0],
            None,
            BLOCK_SIZE,
            load_token1_hash,
            load_token1_address
        );
    }
    if limit_order_data.clone().unwrap().balances[1] > Uint128(0) {
        let token2_data = ReadonlyPrefixedStorage::new(TOKEN2_DATA, &deps.storage);
        let load_token2_address: HumanAddr = load(&token2_data, b"address")?;
        let load_token2_hash: String = load(&token2_data, b"hash")?;

        let _transfer_result: StdResult<CosmosMsg> = transfer_msg(
            env.message.sender.clone(),
            limit_order_data.clone().unwrap().balances[1],
            None,
            BLOCK_SIZE,
            load_token2_hash,
            load_token2_address
        );
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
    Ok(HandleResponse::default())
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
                order_total_quantity = order_total_quantity + limit_order_data.clone().unwrap().balances[limit_order_data.clone().unwrap().order_token_index];
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
        QueryMsg::GetLimitOrder {user_address, user_viewkey} => to_binary(&get_limit_order(deps, user_address, user_viewkey)?),
        QueryMsg::CheckOrderBookTrigger {user_address, user_viewkey} => to_binary(&check_order_book_trigger(deps, user_address, user_viewkey)?),
        _ => Err(StdError::generic_err("Handler not found!"))
    }
}

fn get_limit_order<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    user_address: HumanAddr,
    user_viewkey: String
) -> StdResult<LimitOrderState> {
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
            return Ok(limit_order_data)
        } else {
            return Err(StdError::generic_err(format!(
                "No limit order found for this user."
            ))); 
        }
    } else {
        return Err(StdError::generic_err(format!(
            "Invalid address - viewkey pair!"
        ))); 
    }
}

fn check_order_book_trigger<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    user_address: HumanAddr,
    user_viewkey: String
) -> StdResult<bool> {
        let mut bid_order_book:OrderQueue = load(&deps.storage, BID_ORDER_QUEUE).unwrap();
        let mut bid_price: Option<Uint128> = None;
        let mut ask_price: Option<Uint128> = None;

        // Peek order books for the price and check the AMM price for this asset
        // Respond true if needs to trigger and false if not

        //if let Some(bid_order_book_peek) = bid_order_book.peek() {
        //    bid_price = Some(bid_order_book_peek.price);
        //}

        return Ok(true)
}