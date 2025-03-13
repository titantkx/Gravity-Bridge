use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct TKXIbcInfo {
    pub address_regex: String,
    pub denom: String,
    pub decimals: u8,
    pub channel_id: String,
}

#[cw_serde]
pub struct SendingWithdrawInfo {
    pub ibc_channel_id: String,
    pub chain_prefix: String,
    pub sender: Addr,
    pub recipient: String,
    pub forwarder: String,
    pub total_amount: Uint128,
    pub amount: Uint128,
    pub bridge_fee: Uint128,
}

#[cw_serde]
pub struct WithdrawInfo {
    pub ibc_channel_id: String,
    pub sequence: u64,
    pub chain_prefix: String,
    pub sender: Addr,
    pub recipient: String,
    pub forwarder: String,
    pub total_amount: Uint128,
    pub amount: Uint128,
    pub bridge_fee: Uint128,
}
