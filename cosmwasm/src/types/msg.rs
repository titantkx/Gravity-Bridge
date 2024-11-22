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
    /// Set the admin of the contract.
    /// Only admin can set the admin of the contract.
    SetAdmin(SetAdminMsg),

    /// Add a tkx ibc token denomination.
    /// Only admin can add a tkx ibc token denomination.
    AddTkxIbcDenom(AddTkxIbcDenomMsg),

    /// Remove a tkx ibc token denomination.
    /// Only admin can remove a tkx ibc token denomination.
    RemoveTkxIbcDenom(RemoveTkxChainMsg),
}

#[cw_serde]
pub struct AddTkxIbcDenomMsg {
    pub chain_prefix: String,
    /// Denom is the tkx ibc token denomination to be added.
    pub denom: String,
}

#[cw_serde]
pub struct RemoveTkxChainMsg {
    pub chain_prefix: String,
}

#[cw_serde]
pub struct SetAdminMsg {
    pub admin: String,
}
