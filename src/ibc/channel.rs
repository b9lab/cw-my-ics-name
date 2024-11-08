use crate::error::ContractError;
use cosmwasm_std::{
    entry_point, DepsMut, Env, IbcBasicResponse, IbcChannel, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcOrder,
};

pub const IBC_CUSTOM_PROTOCOL_VERSION: &str = "ibc-name-transfer-1.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(None)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_channel_connect")
        .add_attribute("channel", &msg.channel().endpoint.channel_id)
        .add_attribute("port", &msg.channel().endpoint.port_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    match msg {
        IbcChannelCloseMsg::CloseInit { channel: _ } => Err(ContractError::CantCloseChannel {}),
        IbcChannelCloseMsg::CloseConfirm { channel: _ } => Ok(IbcBasicResponse::default()),
    }
}

fn validate_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    if channel.order != IbcOrder::Unordered {
        return Err(ContractError::OrderedChannel);
    }
    if channel.version != IBC_CUSTOM_PROTOCOL_VERSION {
        return Err(ContractError::InvalidIbcVersion {
            actual: channel.version.clone(),
            expected: IBC_CUSTOM_PROTOCOL_VERSION.to_string(),
        });
    }
    if let Some(counterparty_version) = counterparty_version {
        if counterparty_version != IBC_CUSTOM_PROTOCOL_VERSION {
            return Err(ContractError::InvalidIbcVersion {
                actual: counterparty_version.to_string(),
                expected: IBC_CUSTOM_PROTOCOL_VERSION.to_string(),
            });
        }
    }
    Ok(())
}
