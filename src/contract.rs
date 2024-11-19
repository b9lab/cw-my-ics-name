use crate::{
    error::ContractError,
    msg::{
        CollectionExecuteMsg, CollectionQueryMsg, ExecuteMsg, IbcPacketMessage, InstantiateMsg,
        QueryMsg, SudoMsg,
    },
    state::VOUCHERS_ADDR,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Deps, DepsMut, Env, Event, IbcMsg, IbcTimeout, MessageInfo, QueryRequest,
    QueryResponse, Response, WasmMsg, WasmQuery,
};
use cw721::msg::OwnerOfResponse;

type ContractResult = Result<Response, ContractError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult {
    let instantiate_event = Event::new("my-ics-name");
    if let Some(addr) = &msg.vouchers_addr {
        VOUCHERS_ADDR.save(deps.storage, &addr)?;
    }
    let instantiate_event = append_vouchers_attributes(instantiate_event, &msg.vouchers_addr);
    Ok(Response::default().add_event(instantiate_event))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResult {
    match msg {
        ExecuteMsg::IbcTransferName {
            collection,
            receiver_addr,
            token_id,
            channel_id,
        } => execute_ibc_tranfer(
            deps,
            env,
            info,
            collection,
            receiver_addr,
            token_id,
            channel_id,
        ),
    }
}

fn execute_ibc_tranfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: String,
    receiver_addr: String,
    token_id: String,
    channel_id: String,
) -> ContractResult {
    validate_sender_is_owner(&deps, &info, &collection, &token_id)?;
    let escrow_msg = CollectionExecuteMsg::TransferNft {
        recipient: env.contract.address.to_string(),
        token_id: token_id.to_owned(),
    };
    let escrow_exec_msg = WasmMsg::Execute {
        contract_addr: collection.to_owned(),
        msg: to_json_binary(&escrow_msg)?,
        funds: vec![],
    };
    let transfer_msg = IbcPacketMessage::TransferName {
        collection: collection.to_owned(),
        token_id,
        sender_addr: info.sender.to_string(),
        receiver_addr,
    };
    let transfer_packet = IbcMsg::SendPacket {
        channel_id,
        data: to_json_binary(&transfer_msg)?,
        timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(120)),
    };
    Ok(Response::default()
        .add_message(escrow_exec_msg)
        .add_message(transfer_packet))
}

pub fn validate_sender_is_owner(
    deps: &DepsMut,
    info: &MessageInfo,
    collection: &String,
    token_id: &String,
) -> Result<(), ContractError> {
    let owner = deps
        .querier
        .query::<OwnerOfResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: collection.to_string(),
            msg: to_json_binary(&CollectionQueryMsg::OwnerOf {
                token_id: token_id.to_string(),
                include_expired: None,
            })?,
        }))?
        .owner;
    if owner != info.sender.to_string() {
        Err(ContractError::OnlyOwner)
    } else {
        Ok(())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> Result<QueryResponse, ContractError> {
    todo!("query");
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> ContractResult {
    match msg {
        SudoMsg::UpdateVouchersAddr(vouchers_addr) => {
            sudo_update_vouchers_addr(deps, &vouchers_addr)
        }
    }
}

fn sudo_update_vouchers_addr(deps: DepsMut, vouchers_addr: &Option<String>) -> ContractResult {
    if let Some(addr) = vouchers_addr {
        VOUCHERS_ADDR.save(deps.storage, &addr)?;
    }
    let sudo_event = Event::new("my-ics-name");
    let sudo_event = append_vouchers_attributes(sudo_event, vouchers_addr);
    Ok(Response::default().add_event(sudo_event))
}

fn append_vouchers_attributes(my_event: Event, vouchers_addr: &Option<String>) -> Event {
    if let Some(addr) = vouchers_addr {
        my_event.add_attribute("update-vouchers-address", addr)
    } else {
        my_event
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use cosmwasm_std::{
        from_json,
        testing::{self, MockApi, MockQuerier, MockStorage},
        to_json_binary, Addr, ContractResult, Empty, Event, IbcMsg, IbcTimeout, OwnedDeps, Querier,
        QuerierResult, QueryRequest, Response, SystemError, SystemResult, WasmMsg, WasmQuery,
    };
    use cw721::msg::OwnerOfResponse;

    use crate::{
        msg::{
            CollectionExecuteMsg, CollectionQueryMsg, ExecuteMsg, IbcPacketMessage, InstantiateMsg,
        },
        state::VOUCHERS_ADDR,
    };

    pub fn mock_deps(
        response: OwnerOfResponse,
    ) -> OwnedDeps<MockStorage, MockApi, OwnerOfMockQuerier, Empty> {
        OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: OwnerOfMockQuerier::new(MockQuerier::new(&[]), response),
            custom_query_type: PhantomData,
        }
    }

    pub struct OwnerOfMockQuerier {
        base: MockQuerier,
        response: OwnerOfResponse,
    }

    impl Querier for OwnerOfMockQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request: QueryRequest<Empty> = match from_json(bin_request) {
                Ok(v) => v,
                Err(e) => {
                    return SystemResult::Err(SystemError::InvalidRequest {
                        error: format!("Parsing query request: {}", e),
                        request: bin_request.into(),
                    })
                }
            };

            self.handle_query(&request)
        }
    }

    impl OwnerOfMockQuerier {
        pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
            match &request {
                QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: _,
                    msg,
                }) => {
                    let expected = to_json_binary(&CollectionQueryMsg::OwnerOf {
                        token_id: "3".to_owned(),
                        include_expired: None,
                    })
                    .expect("Failed to create expected query");
                    assert_eq!(expected.to_vec(), msg.to_vec(), "Query is not owner");
                    SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&self.response).expect("Failed to serialize owner response"),
                    ))
                }
                _ => self.base.handle_query(request),
            }
        }

        pub fn new(base: MockQuerier<Empty>, response: OwnerOfResponse) -> Self {
            OwnerOfMockQuerier { base, response }
        }
    }

    #[test]
    fn test_instantiate() {
        // Arrange
        let mut mocked_deps_mut = testing::mock_dependencies();
        let mocked_env = testing::mock_env();
        let deployer = Addr::unchecked("deployer");
        let vouchers = Addr::unchecked("vouchers");
        let mocked_msg_info = testing::mock_info(deployer.as_ref(), &[]);
        let instantiate_msg = InstantiateMsg {
            vouchers_addr: Some(vouchers.to_string()),
        };

        // Act
        let result = super::instantiate(
            mocked_deps_mut.as_mut(),
            mocked_env.to_owned(),
            mocked_msg_info,
            instantiate_msg.to_owned(),
        );

        // Assert
        assert!(result.is_ok(), "Failed to instantiate ics name");
        let received_response = result.unwrap();
        let expected_response = Response::default().add_event(
            Event::new("my-ics-name")
                .add_attribute("update-vouchers-address", vouchers.to_string()),
        );
        assert_eq!(received_response, expected_response);
        let saved_vouchers = VOUCHERS_ADDR
            .load(&mocked_deps_mut.storage)
            .expect("Failed to load vouchers address");
        assert_eq!(saved_vouchers, vouchers.to_string());
    }

    #[test]
    fn test_execute_transfer_name() {
        // Arrange
        let mut mocked_deps_mut = mock_deps(OwnerOfResponse {
            owner: "sender".to_owned(),
            approvals: vec![],
        });
        let mocked_env = testing::mock_env();
        let sender = Addr::unchecked("sender");
        let mocked_msg_info = testing::mock_info(sender.as_ref(), &[]);
        let transfer_msg = ExecuteMsg::IbcTransferName {
            channel_id: "2".to_owned(),
            collection: "original".to_owned(),
            token_id: "3".to_owned(),
            receiver_addr: "receiver".to_owned(),
        };

        // Act
        let result = super::execute(
            mocked_deps_mut.as_mut(),
            mocked_env.to_owned(),
            mocked_msg_info,
            transfer_msg.to_owned(),
        );

        // Assert
        assert!(result.is_ok(), "Failed to execute name transfer");
        let received_response = result.unwrap();
        let expected_escrow_exec_msg = WasmMsg::Execute {
            contract_addr: "original".to_owned(),
            msg: to_json_binary(&CollectionExecuteMsg::TransferNft {
                recipient: mocked_env.contract.address.to_string(),
                token_id: "3".to_owned(),
            })
            .expect("Failed to serialize collection message"),
            funds: vec![],
        };
        let expected_transfer_packet = IbcMsg::SendPacket {
            channel_id: "2".to_owned(),
            data: to_json_binary(&IbcPacketMessage::TransferName {
                collection: "original".to_owned(),
                token_id: "3".to_owned(),
                sender_addr: "sender".to_owned(),
                receiver_addr: "receiver".to_owned(),
            })
            .expect("Failed to serialize transfer message"),
            timeout: IbcTimeout::with_timestamp(mocked_env.block.time.plus_seconds(120)),
        };
        let expected_response = Response::default()
            .add_message(expected_escrow_exec_msg)
            .add_message(expected_transfer_packet);
        assert_eq!(received_response, expected_response);
    }
}
