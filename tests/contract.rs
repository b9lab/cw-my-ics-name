use cosmwasm_std::Addr;
use cw_multi_test::{App, ContractWrapper, Executor};
use cw_my_ics_name::{
    contract::{execute, instantiate, query},
    msg::InstantiateMsg,
};

fn instantiate_ics_name(mock_app: &mut App) -> (u64, Addr) {
    let ics_name_code = Box::new(ContractWrapper::new(execute, instantiate, query));
    let ics_name_code_id = mock_app.store_code(ics_name_code);
    let ics_name_addr = mock_app
        .instantiate_contract(
            ics_name_code_id,
            Addr::unchecked("deployer"),
            &InstantiateMsg {},
            &[],
            "ics-name",
            None,
        )
        .expect("Failed to instantiate ics-name");
    return (ics_name_code_id, ics_name_addr);
}
