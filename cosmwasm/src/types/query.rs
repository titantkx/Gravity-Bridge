use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetAdminResp)]
    GetAdmin {},

    #[returns(ListTxkIbcDenomResp)]
    ListTxkIbcDenom {},

    #[returns(ListTKXChainInfoResp)]
    ListTKXChainWithDenom {},

    #[returns(WithdrawInfoResp)]
    GetWithdrawInfo { channel_id: String, sequence: u64 },
}

#[cw_serde]
pub struct GetAdminResp {
    pub address: String,
}

#[cw_serde]
pub struct ListTxkIbcDenomResp {
    pub denoms: Vec<String>,
}

#[cw_serde]
pub struct TKXChainInfo {
    pub chain_prefix: String,
    pub denom: String,
    pub channel_id: String,
}

#[cw_serde]
pub struct ListTKXChainInfoResp {
    pub data: Vec<TKXChainInfo>,
}

#[cw_serde]
pub struct WithdrawInfoResp {
    pub ibc_channel_id: String,
    pub sequence: u64,
    pub chain_prefix: String,
    pub sender: String,
    pub recipient: String,
    pub forwarder: String,
    pub total_amount: Uint128,
    pub amount: Uint128,
    pub bridge_fee: Uint128,
}
