use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetAdminResp)]
    GetAdmin {},

    #[returns(ListTxkIbcDenomResp)]
    ListTxkIbcDenom {},

    #[returns(ListTKXChainWithDenomResp)]
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
pub struct TKXChainWithDenom {
    pub chain_prefix: String,
    pub denom: String,
}

#[cw_serde]
pub struct ListTKXChainWithDenomResp {
    pub data: Vec<TKXChainWithDenom>,
}
