use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetAdminResp)]
    GetAdmin {},

    #[returns(ListTxkIbcDenomResp)]
    ListTxkIbcDenom {},

    #[returns(ListTKXChainInfoResp)]
    ListTKXChainWithDenom {},
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
