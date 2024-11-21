use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    /// Address of the admin of the contract.
    /// Admin can manage tkx ibc token denoms.
    /// If not specified, default to sender's address.
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Add a tkx ibc token denomination.
    /// Only admin can add a tkx ibc token denomination.
    AddTkxIbcDenom(AddTkxIbcDenomMsg),

    /// Remove a tkx ibc token denomination.
    /// Only admin can remove a tkx ibc token denomination.
    RemoveTkxIbcDenom(RemoveTkxIbcDenomMsg),
}

#[cw_serde]
pub struct AddTkxIbcDenomMsg {
    /// Denom is the tkx ibc token denomination to be added.
    pub denom: String,
}

#[cw_serde]
pub struct RemoveTkxIbcDenomMsg {
    /// Denom is the tkx ibc token denomination to be removed.
    pub denom: String,
}
