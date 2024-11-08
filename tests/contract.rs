use cosmwasm_std::{Addr, Empty};
use cw721::msg::{Cw721ExecuteMsg, Cw721QueryMsg};
use cw_multi_test::{App, ContractWrapper, Executor, WasmSudo};
use cw_my_ics_name::{
    contract::{execute, instantiate, query},
    msg::{InstantiateMsg, SudoMsg},
};
use cw_my_nameservice::{
    contract::{
        execute as execute_my_nameservice, instantiate as instantiate_my_nameservice,
        query as query_my_nameservice,
    },
    msg::InstantiateMsg as MyNameserviceInstantiateMsg,
};

pub type CollectionExecuteMsg = Cw721ExecuteMsg<Option<Empty>, Option<Empty>, Empty>;
pub type CollectionQueryMsg = Cw721QueryMsg<Option<Empty>, Option<Empty>, Empty>;

fn instantiate_nameservice(mock_app: &mut App, minter: String) -> (u64, Addr) {
    let name_vouchers_code = Box::new(ContractWrapper::new(
        execute_my_nameservice,
        instantiate_my_nameservice,
        query_my_nameservice,
    ));
    let name_vouchers_code_id = mock_app.store_code(name_vouchers_code);
    return (
        name_vouchers_code_id,
        mock_app
            .instantiate_contract(
                name_vouchers_code_id,
                Addr::unchecked("deployer-my-name-vouchers"),
                &MyNameserviceInstantiateMsg {
                    name: "my name vouchers".to_owned(),
                    symbol: "MYV".to_owned(),
                    creator: None,
                    minter: Some(minter),
                    collection_info_extension: None,
                    withdraw_address: None,
                },
                &[],
                "name vouchers",
                None,
            )
            .expect("Failed to instantiate my name vouchers"),
    );
}

fn instantiate_ics_name(mock_app: &mut App) -> (Addr, u64, Addr) {
    let ics_name_code = Box::new(ContractWrapper::new(execute, instantiate, query));
    let ics_name_code_id = mock_app.store_code(ics_name_code);
    let ics_name_addr = mock_app
        .instantiate_contract(
            ics_name_code_id,
            Addr::unchecked("deployer"),
            &InstantiateMsg {
                vouchers_addr: None,
            },
            &[],
            "ics-name",
            None,
        )
        .expect("Failed to instantiate ics-name");
    let (_, vouchers_addr) = instantiate_nameservice(mock_app, ics_name_addr.to_string());
    mock_app
        .sudo(cw_multi_test::SudoMsg::Wasm(
            WasmSudo::new(
                &ics_name_addr,
                &SudoMsg::UpdateVouchersAddr(Some(vouchers_addr.to_string())),
            )
            .expect("Failed to create new sudo message"),
        ))
        .expect("Failed to set vouchers on ics-name");
    return (vouchers_addr, ics_name_code_id, ics_name_addr);
}
