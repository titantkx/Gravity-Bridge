use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct TKXIbcInfo {
    pub denom: String,
    pub channel_id: String,
}
