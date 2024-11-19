use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Empty;
use cw721::msg::{Cw721ExecuteMsg, Cw721QueryMsg};

pub type CollectionExecuteMsg = Cw721ExecuteMsg<Option<Empty>, Option<Empty>, Empty>;
pub type CollectionQueryMsg = Cw721QueryMsg<Option<Empty>, Option<Empty>, Empty>;

#[cw_serde]
pub struct InstantiateMsg {
    pub vouchers_addr: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    IbcTransferName {
        channel_id: String,
        collection: String,
        token_id: String,
        receiver_addr: String,
    },
    IbcReturnName {
        channel_id: String,
        collection: String,
        token_id: String,
        receiver_addr: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[cw_serde]
pub enum SudoMsg {
    UpdateVouchersAddr(Option<String>),
}

// #[derive(Debug)]
#[cw_serde]
pub enum IbcPacketMessage {
    TransferName {
        collection: String,
        token_id: String,
        sender_addr: String,
        receiver_addr: String,
    },
    ReturnName {
        collection: String,
        token_id: String,
        sender_addr: String,
        receiver_addr: String,
    },
}
