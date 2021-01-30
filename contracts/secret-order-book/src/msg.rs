use cosmwasm_std::{Binary, CanonicalAddr, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, Query};
use serde::{Deserialize, Serialize};

use crate::{contract::BLOCK_SIZE, order_queues::OrderSide};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub factory_address: HumanAddr,
    pub factory_hash: String,
    pub factory_key: String,
    pub token1_code_address: HumanAddr,
    pub token1_code_hash: String,
    pub token2_code_address: HumanAddr,
    pub token2_code_hash: String,
}

// Messages sent to SNIP-20 contracts
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Snip20Msg {
    RegisterReceive {
        code_hash: String,
        padding: Option<String>,
    },
    Redeem {
        amount: Uint128,
        padding: Option<String>,
    },
}

impl Snip20Msg {
    pub fn register_receive(code_hash: String) -> Self {
        Snip20Msg::RegisterReceive {
            code_hash,
            padding: None, // TODO add padding calculation
        }
    }

    pub fn redeem(amount: Uint128) -> Self {
        Snip20Msg::Redeem {
            amount,
            padding: None, // TODO add padding calculation
        }
    }
}

/// the factory's handle messages this auction will call
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FactoryHandleMsg {
    InitCallBackFromSecretOrderBookToFactory  {
        auth_key: String,
        contract_address: HumanAddr,
        token1_address: HumanAddr,
        token2_address: HumanAddr,
    },
}

impl HandleCallback for FactoryHandleMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive{ sender: HumanAddr, from: HumanAddr, amount: Uint128, msg: Binary },
    CreateLimitOrder {
        side: OrderSide, // bid||ask
        price: Uint128
    }
}

/// Queries
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetLimitOrder {
        user_address: HumanAddr,
        user_viewkey: String
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    LimitOrders {}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsKeyValidResponse {
    pub is_key_valid: IsKeyValid  
} 

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsKeyValid {
    pub is_valid: bool
}
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FactoryQueryMsg {
    IsKeyValid {
        factory_key: String,
        viewing_key: String,
        address: HumanAddr
    }
}
impl Query for FactoryQueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}
// State
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, JsonSchema)]
pub enum LimitOrderStatus {
    Active,
    PartiallyFilled,
    Filled,
    Cancelled,
    Completed
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LimitOrderState {
    pub side: OrderSide,
    pub status: LimitOrderStatus,
    pub price: Uint128,
    pub balances: Vec<Uint128>,
    pub timestamp: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OrderBookState {
    pub operation: String,
    pub price: Uint128,
    pub total_quantity: Uint128,
    pub limit_orders: Vec<CanonicalAddr>
}