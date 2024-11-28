use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

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
    AddTKXIbcDenom(AddTKXIbcDenomMsg),

    /// Remove a tkx ibc token denomination.
    /// Only admin can remove a tkx ibc token denomination.
    RemoveTKXIbcDenom(RemoveTKXChainMsg),

    /// contract receive a tkx ibc token convert it to native TKX token and send it to recipient.
    Deposit(DepositMsg),

    /// contract receive a native TKX token convert it to ibc token and send it to recipient on other chain through gravity chain.
    Withdraw(WithdrawMsg),
}

#[cw_serde]
pub struct AddTKXIbcDenomMsg {
    pub chain_prefix: String,
    /// Denom is the tkx ibc token denomination to be added.
    pub denom: String,
    /// IBC channel id of the tkx ibc token.
    pub channel_id: String,
}

#[cw_serde]
pub struct RemoveTKXChainMsg {
    pub chain_prefix: String,
}

#[cw_serde]
pub struct SetAdminMsg {
    pub admin: String,
}

#[cw_serde]
pub struct DepositMsg {
    /// this id is string set by sender from source chain (e.g. ethereum chain)
    /// it format should be "[chain_prefix]/[event_nonce]"
    pub deposit_id: String,
    /// bech32 string recipient address on the titan chain (chain that have this contract)
    pub recipient: String,
}

#[cw_serde]
pub struct WithdrawMsg {
    /// address that be forwarder in gravity chain
    pub forwarder: String,
    /// chain prefix of the destination evm chain
    pub chain_prefix: String,
    /// recipient address on the other chain
    pub recipient: String,
    /// amount that recipient will receive
    pub amount: Uint128,
    /// fee that will be paid for eth relayer
    pub bridge_fee: Uint128,
}

#[cw_serde]
pub struct IbcAutoSendEth {
    pub evm_chain_prefix: String,
    pub eth_dest: String,
    pub amount: Uint128,
    pub bridge_fee: Uint128,
}

#[cw_serde]
pub struct IbcAutoSendEthMemo {
    pub send_to_eth: IbcAutoSendEth,
    /// contract address to receive ibc callback
    pub ibc_callback: String,
}
