use super::helpers::compute_voucher_token_id;
use crate::{
    error::ContractError,
    msg::{CollectionExecuteMsg, IbcPacketMessage},
    state::VOUCHERS_ADDR,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, to_json_binary, DepsMut, Env, Event, IbcBasicResponse, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, StdAck, WasmMsg,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let ibc_msg = from_json::<IbcPacketMessage>(msg.packet.data)?;
    let response = match ibc_msg {
        IbcPacketMessage::TransferName {
            collection,
            token_id,
            sender_addr: _sender_addr,
            receiver_addr,
        } => ibc_receive_transfer_name(
            deps,
            env,
            msg.packet.dest.channel_id,
            collection,
            token_id,
            receiver_addr,
        ),
        IbcPacketMessage::ReturnName {
            collection,
            token_id,
            sender_addr: _sender_addr,
            receiver_addr,
        } => ibc_receive_return_name(deps, env, collection, token_id, receiver_addr),
    };
    match response {
        Ok(response) => Ok(response),
        Err(error) => Ok(IbcReceiveResponse::new()
            .add_attribute("method", "ibc_packet_receive")
            .add_attribute("error", error.to_string())
            .set_ack(StdAck::Error(error.to_string()))),
    }
}

fn ibc_receive_transfer_name(
    deps: DepsMut,
    _env: Env,
    channel_id: String,
    collection: String,
    token_id: String,
    receiver_addr: String,
) -> Result<IbcReceiveResponse, ContractError> {
    let voucher_collection = VOUCHERS_ADDR.load(deps.storage)?;
    let voucher_token_id = compute_voucher_token_id(&channel_id, &collection, &token_id);
    let mint_msg = CollectionExecuteMsg::Mint {
        token_id: voucher_token_id,
        owner: receiver_addr,
        token_uri: None,
        extension: None,
    };
    let mint_exec_msg = WasmMsg::Execute {
        contract_addr: voucher_collection,
        msg: to_json_binary(&mint_msg)?,
        funds: vec![],
    };
    let mint_event = Event::new("my-ics-name-voucher-mint")
        .add_attribute("channel", channel_id)
        .add_attribute("original-collection", collection)
        .add_attribute("token-id", token_id);
    Ok(IbcReceiveResponse::default()
        .add_message(mint_exec_msg)
        .add_event(mint_event))
}

fn ibc_receive_return_name(
    _deps: DepsMut,
    _env: Env,
    collection: String,
    token_id: String,
    receiver_addr: String,
) -> Result<IbcReceiveResponse, ContractError> {
    let unescrow_msg = CollectionExecuteMsg::TransferNft {
        token_id,
        recipient: receiver_addr,
    };
    let unescrow_wasm_msg = WasmMsg::Execute {
        contract_addr: collection,
        msg: to_json_binary(&unescrow_msg)?,
        funds: vec![],
    };
    Ok(IbcReceiveResponse::new().add_message(unescrow_wasm_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    ack: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let ack_data = from_json::<StdAck>(&ack.acknowledgement.data)
        .unwrap_or_else(|_| StdAck::Error(ack.acknowledgement.data.to_base64()));
    let original_msg = from_json::<IbcPacketMessage>(ack.original_packet.data)?;
    match original_msg {
        IbcPacketMessage::TransferName {
            collection,
            token_id,
            sender_addr,
            receiver_addr: _receiver_addr,
        } => match ack_data {
            StdAck::Error(_) => unescrow_name(&deps, &env, &collection, &token_id, &sender_addr),
            StdAck::Success(_) => Ok(IbcBasicResponse::default()),
        },
        IbcPacketMessage::ReturnName {
            collection,
            token_id,
            sender_addr,
            receiver_addr: _receiver_addr,
        } => match ack_data {
            StdAck::Error(_) => unescrow_voucher(
                &deps,
                &env,
                &ack.original_packet.src.channel_id,
                &collection,
                &token_id,
                &sender_addr,
            ),
            StdAck::Success(_) => burn_voucher(
                &deps,
                &env,
                &ack.original_packet.src.channel_id,
                &collection,
                &token_id,
            ),
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    match from_json::<IbcPacketMessage>(msg.packet.data)? {
        IbcPacketMessage::TransferName {
            collection,
            token_id,
            sender_addr,
            receiver_addr: _receiver_addr,
        } => unescrow_name(&deps, &env, &collection, &sender_addr, &token_id),
        IbcPacketMessage::ReturnName {
            collection,
            token_id,
            sender_addr,
            receiver_addr: _receiver_addr,
        } => unescrow_voucher(
            &deps,
            &env,
            &msg.packet.src.channel_id,
            &collection,
            &token_id,
            &sender_addr,
        ),
    }
}

fn unescrow_name(
    _deps: &DepsMut,
    _env: &Env,
    collection: &String,
    token_id: &String,
    original_sender_addr: &String,
) -> Result<IbcBasicResponse, ContractError> {
    let unescrow_msg = CollectionExecuteMsg::TransferNft {
        token_id: token_id.to_string(),
        recipient: original_sender_addr.to_string(),
    };
    let exec_msg = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_json_binary(&unescrow_msg)?,
        funds: vec![],
    };
    Ok(IbcBasicResponse::new().add_message(exec_msg))
}

fn burn_voucher(
    deps: &DepsMut,
    _env: &Env,
    channel_id: &String,
    collection: &String,
    token_id: &String,
) -> Result<IbcBasicResponse, ContractError> {
    let vouchers: String = VOUCHERS_ADDR.load(deps.storage)?;
    let voucher_token_id = compute_voucher_token_id(&channel_id, &collection, &token_id);
    let burn_msg = CollectionExecuteMsg::Burn {
        token_id: voucher_token_id,
    };
    let burn_exec_msg = WasmMsg::Execute {
        contract_addr: vouchers,
        msg: to_json_binary(&burn_msg)?,
        funds: vec![],
    };
    let burn_event = Event::new("ibc-voucher-burn")
        .add_attribute("channel", channel_id.to_owned())
        .add_attribute("original-collection", collection.to_owned())
        .add_attribute("token_id", token_id.to_owned());
    Ok(IbcBasicResponse::default()
        .add_message(burn_exec_msg)
        .add_event(burn_event))
}

fn unescrow_voucher(
    deps: &DepsMut,
    _env: &Env,
    channel_id: &String,
    collection: &String,
    token_id: &String,
    original_sender_addr: &String,
) -> Result<IbcBasicResponse, ContractError> {
    let vouchers = VOUCHERS_ADDR.load(deps.storage)?;
    let voucher_token_id = compute_voucher_token_id(&channel_id, &collection, &token_id);
    let unescrow_msg = CollectionExecuteMsg::TransferNft {
        token_id: voucher_token_id,
        recipient: original_sender_addr.to_string(),
    };
    let unescrow_exec_msg = WasmMsg::Execute {
        contract_addr: vouchers,
        msg: to_json_binary(&unescrow_msg)?,
        funds: vec![],
    };
    Ok(IbcBasicResponse::new().add_message(unescrow_exec_msg))
}

#[cfg(test)]
mod tests {
    use crate::{
        contract::instantiate,
        ibc::packet::ibc_packet_receive,
        msg::{CollectionExecuteMsg, IbcPacketMessage, InstantiateMsg},
        state::VOUCHERS_ADDR,
    };
    use cosmwasm_std::{testing, to_json_binary, Addr, Event, IbcReceiveResponse, WasmMsg};

    #[test]
    fn test_ibc_receive_transfer_name() {
        // Arrange
        let mut mocked_deps_mut = testing::mock_dependencies();
        let mocked_env = testing::mock_env();
        let deployer = Addr::unchecked("deployer");
        let vouchers = Addr::unchecked("vouchers");
        let mocked_msg_info = testing::mock_info(deployer.as_ref(), &[]);
        let instantiate_msg = InstantiateMsg {
            vouchers_addr: Some(vouchers.to_string()),
        };
        let _ = instantiate(
            mocked_deps_mut.as_mut(),
            mocked_env.to_owned(),
            mocked_msg_info,
            instantiate_msg.to_owned(),
        )
        .expect("Failed to instantiate ics name");
        let transfer_msg = IbcPacketMessage::TransferName {
            collection: "original".to_owned(),
            token_id: "3".to_owned(),
            sender_addr: "sender".to_owned(),
            receiver_addr: "receiver".to_owned(),
        };
        let mocked_receive_packet =
            testing::mock_ibc_packet_recv("20", &transfer_msg).expect("Failed to mock packet");

        // Act
        let result = ibc_packet_receive(
            mocked_deps_mut.as_mut(),
            mocked_env.to_owned(),
            mocked_receive_packet,
        );

        // Assert
        assert!(result.is_ok(), "Failed to receive packet");
        let received_response = result.unwrap();
        let expected_mint_msg = CollectionExecuteMsg::Mint {
            // sha256 of "transfer_name/ibc/20/original/3"
            token_id: "a6b4a0065ec52c2b0b296e0f7e006a85800688897737ac2a182e50d9015c998e".to_owned(),
            owner: "receiver".to_owned(),
            token_uri: None,
            extension: None,
        };
        let expected_mint_exec_msg = WasmMsg::Execute {
            contract_addr: "vouchers".to_owned(),
            msg: to_json_binary(&expected_mint_msg).expect("Failed to serialize mint msg"),
            funds: vec![],
        };
        let expected_event = Event::new("my-ics-name-voucher-mint")
            .add_attribute("channel", "20".to_string())
            .add_attribute("original-collection", "original".to_string())
            .add_attribute("token-id", "3".to_string());
        let expected_response = IbcReceiveResponse::default()
            .add_message(expected_mint_exec_msg)
            .add_event(expected_event);
        assert_eq!(received_response, expected_response);
        let saved_vouchers = VOUCHERS_ADDR
            .load(&mocked_deps_mut.storage)
            .expect("Failed to load vouchers address");
        assert_eq!(saved_vouchers, vouchers.to_string());
    }

    #[test]
    fn test_ibc_receive_return_name() {
        // Arrange
        let mut mocked_deps_mut = testing::mock_dependencies();
        let mocked_env = testing::mock_env();
        let deployer = Addr::unchecked("deployer");
        let mocked_msg_info = testing::mock_info(deployer.as_ref(), &[]);
        let instantiate_msg = InstantiateMsg {
            vouchers_addr: None,
        };
        let _ = instantiate(
            mocked_deps_mut.as_mut(),
            mocked_env.to_owned(),
            mocked_msg_info,
            instantiate_msg.to_owned(),
        )
        .expect("Failed to instantiate ics name");
        let return_msg = IbcPacketMessage::ReturnName {
            collection: "original".to_owned(),
            token_id: "3".to_owned(),
            sender_addr: "sender".to_owned(),
            receiver_addr: "receiver".to_owned(),
        };
        let mocked_receive_packet =
            testing::mock_ibc_packet_recv("20", &return_msg).expect("Failed to mock packet");

        // Act
        let result = ibc_packet_receive(
            mocked_deps_mut.as_mut(),
            mocked_env.to_owned(),
            mocked_receive_packet,
        );

        // Assert
        assert!(result.is_ok(), "Failed to receive packet");
        println!("{:?}", result);
        let received_response = result.unwrap();
        let expected_unescrow_msg = CollectionExecuteMsg::TransferNft {
            recipient: "receiver".to_owned(),
            token_id: "3".to_owned(),
        };
        let expected_unescrow_exec_msg = WasmMsg::Execute {
            contract_addr: "original".to_owned(),
            msg: to_json_binary(&expected_unescrow_msg).expect("Failed to serialize unescrow msg"),
            funds: vec![],
        };
        let expected_response =
            IbcReceiveResponse::default().add_message(expected_unescrow_exec_msg);
        assert_eq!(received_response, expected_response);
    }
}
