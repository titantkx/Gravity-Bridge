use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetAdminResp)]
    GetAdmin {},

    #[returns(ListTxkIbcDenomResp)]
    ListTxkIbcDenom {},
}

#[cw_serde]
pub struct GetAdminResp {
    pub address: String,
}

#[cw_serde]
pub struct ListTxkIbcDenomResp {
    pub denoms: Vec<String>,
}
