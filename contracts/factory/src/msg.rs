use cosmwasm_std::{CanonicalAddr, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, Query};
use serde::{Deserialize, Serialize};
use crate::{contract::BLOCK_SIZE};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub entropy: String,
    pub secret_order_book_code_id: u64,
    pub secret_order_book_code_hash: String,
    pub amm_factory_contract_address: HumanAddr,
    pub amm_factory_contract_hash: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SecretOrderBookContractInitMsg {
    pub factory_address: HumanAddr,
    pub factory_hash: String,
    pub factory_key: String,
    pub token1_info: AssetInfo,
    pub token2_info: AssetInfo,
    pub amm_pair_contract_address: HumanAddr,
    pub amm_pair_contract_hash: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChangeFeeMsg {
    ChangeFee {
        token_index: i8,
        min_amount: Uint128,
        fee_amount: Uint128
    }
}

impl HandleCallback for ChangeFeeMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    CreateViewingKey {entropy: String},
    ChangeSecretOrderBookContractCodeId {code_id: u64, code_hash: String},
    NewSecretOrderBookInstanciate {
        amm_pair_address: HumanAddr,
        amm_pair_hash: String,
        token1_fee: Uint128,
        token2_fee: Uint128
    },
    InitCallBackFromSecretOrderBookToFactory {
        auth_key: String, 
        amm_pair_address: HumanAddr,
        contract_address: HumanAddr,
        contract_hash: String,
        token1_info: AssetInfo,
        token2_info: AssetInfo,
    },
    ChangeAssetFee {
        amm_pairs_address: Vec<HumanAddr>,
        asset_contract_address: HumanAddr,
        new_asset_fee: Uint128
    }
}
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    /// response from creating a viewing key
    ViewingKey { key: String },
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
    /// authenticates the supplied address/viewing key.  This should only be called by arenas
    IsKeyValid {
        /// address whose viewing key is being authenticated
        address: HumanAddr,
        /// viewing key
        viewing_key: String,
        //authentication on factory functions
        factory_key: String
    },
    SecretOrderBookContractCodeId {},
    SecretOrderBook {
        amm_pair_contract_addr: HumanAddr,
    },
    SecretOrderBooks {
        page_size: Option<u32>,
        page: Option<u32>
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    /// Viewing Key Error
    //ViewingKeyError { error: String },
    /// result of authenticating address/key pair
    IsKeyValid { is_valid: bool },
    SecretOrderBookContractCodeID {code_id: u64, code_hash: String},
    SecretOrderBooks {secret_order_books: Vec<SecretOrderBookContract>},
    SecretOrderBook {secret_order_book: Option<SecretOrderBookContract>},
    Error {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SecretOrderBookContract {
    pub amm_pair_contract_addr: HumanAddr,
    pub contract_addr: HumanAddr,
    pub contract_hash: String,
    pub asset_infos: Vec<AssetInfo>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AssetInfo {
    pub decimal_places: u8,
    pub base_amount: Uint128,
    pub fee_amount: Uint128,
    pub min_amount: Uint128,
    pub token: Option<Token>
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AmmQueryMsg {
    Pair {}
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AmmAssetInfo {
    Token {
        contract_addr: String,
        token_code_hash: String,
        viewing_key: String,
    },
    NativeToken {
        denom: String,
    },
}

impl Query for AmmQueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[derive(Serialize, Deserialize)]
pub struct AmmPairResponse {
    pub asset_infos: [AmmAssetInfo; 2],
    pub contract_addr: HumanAddr,
    pub liquidity_token: HumanAddr,
    pub token_code_hash: String
}