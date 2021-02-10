use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, Query};
use serde::{Deserialize, Serialize};

use crate::{contract::BLOCK_SIZE};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub factory_address: HumanAddr,
    pub factory_hash: String,
    pub factory_key: String,
    pub token1_info: AssetInfo,
    pub token2_info: AssetInfo,
    pub amm_pair_contract_address: HumanAddr,
    pub amm_pair_contract_hash: String
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
        amm_pair_address: HumanAddr,
        token1_info: AssetInfo,
        token2_info: AssetInfo,
    },
}

impl HandleCallback for FactoryHandleMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive{ sender: HumanAddr, from: HumanAddr, amount: Uint128, msg: Binary },
    ReceiveNativeToken {
        is_bid: bool,
        price: Uint128
    },
    CreateLimitOrder {
        is_bid: bool,
        price: Uint128
    },
    WithdrawLimitOrder {},
    TriggerLimitOrders {
        test_amm_price: Uint128
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    /// generic status response
    Status {
        /// success or failure
        status: ResponseStatus,
        /// execution description
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
}
/// success or failure response
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub enum ResponseStatus {
    Success,
    Failure,
}

/// Queries
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetLimitOrder {
        user_address: HumanAddr,
        user_viewkey: String
    },
    CheckOrderBookTrigger {
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

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AmmFactoryQueryMsg {
    Pair {
        asset_infos: [AmmAssetInfo; 2]
    }
}
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AmmAssetInfo {
    Token {
        contract_addr: HumanAddr,
        token_code_hash: String,
        viewing_key: String,
    },
    NativeToken {
        denom: String,
    },
}

impl Query for AmmFactoryQueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize)]
pub struct AmmFactoryPairResponse {
    pub asset_infos: [AmmAssetInfo; 2],
    pub contract_addr: HumanAddr,
    pub liquidity_token: HumanAddr,
    pub token_code_hash: String
}

#[derive(Serialize, Deserialize)]
pub enum AmmSimulationQuery {
    simulation {
        offer_asset: AmmSimulationOfferAsset,
    },
    reverseSimulation {
        ask_asset: AmmSimulationOfferAsset,
    }
}
#[derive(Serialize, Deserialize)]
pub struct AmmSimulationOfferAsset {
    pub info: AmmAssetInfo,
    pub amount: Uint128
}

impl Query for AmmSimulationQuery {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize)]
pub struct AmmPairSimulationResponse {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
    pub commission_amount: Uint128
}


// State
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LimitOrderState {
    pub is_bid: bool,
    pub status: String, //Active, PartiallyFilled, Filled
    pub price: Uint128,
    pub order_token_index: i8,
    pub order_token_init_quant: Uint128,
    pub balances: Vec<Uint128>,
    pub timestamp: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AssetInfo {
    pub is_native_token: bool,
    pub token: Option<Token>,
    pub native_token: Option<NativeToken>
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Token {
    pub contract_addr: HumanAddr,
    pub token_code_hash: String
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct NativeToken {
    pub denom: String,
}