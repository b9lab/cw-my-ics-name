use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Unsupported ibc version on channel: {actual}. Expected: {expected}")]
    InvalidIbcVersion { actual: String, expected: String },
    #[error("Only supports unordered channels")]
    OrderedChannel,
    #[error("Channel {channel_id} already exists")]
    ChannelAlreadyExists { channel_id: String },
    #[error("The channel cant be closed")]
    CantCloseChannel,
}
