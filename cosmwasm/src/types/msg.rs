use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the admin of the contract.
    /// Admin can manage tkx ibc token denoms.
    /// If not specified, default to sender's address.
    pub admin: Option<String>,
}
