use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg},
    state::VOUCHERS_ADDR,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Deps, DepsMut, Env, Event, MessageInfo, QueryResponse, Response};

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
pub fn execute(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: ExecuteMsg) -> ContractResult {
    Ok(Response::default())
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
    use cosmwasm_std::{testing, Addr, Event, Response};

    use crate::{msg::InstantiateMsg, state::VOUCHERS_ADDR};

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
}
