use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub entropy: String,
    pub secret_order_book_code_id: u64,
    pub secret_order_book_code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SecretOrderBookContractInitMsg {
    pub factory_address: HumanAddr,
    pub factory_hash: String,
    pub factory_key: String,
    pub token1_code_address: HumanAddr,
    pub token1_code_hash: String,
    pub token2_code_address: HumanAddr,
    pub token2_code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    CreateViewingKey {entropy: String},
    ChangeSecretOrderBookContractCodeId {code_id: u64, code_hash: String},
    NewSecretOrderBookInstanciate {
        token1_code_address: HumanAddr,
        token1_code_hash: String,
        token2_code_address: HumanAddr,
        token2_code_hash: String,
    },
    InitCallBackFromSecretOrderBookToFactory {
        auth_key: String, 
        contract_address: HumanAddr,
        token1_address: HumanAddr,
        token2_address: HumanAddr,
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
    SecretOrderBooks {
        token_address: HumanAddr,
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
    SecretOrderBooks {secret_order_books: Vec<HumanAddr>}
}
