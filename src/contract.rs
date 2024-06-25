#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, ensure_eq, to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Order, Response, StdResult, Validator,
};
use cw_storage_plus::Bound;
use validator::Validate;

use crate::error::ContractError;
use crate::msg::{EntryListResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RegistrationMsg};
use crate::state::{Config, Entry, CONFIG, ENTRIES};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    CONFIG.save(
        deps.storage,
        &Config {
            registration_fee: msg.registration_fee,
            transfer_fee: msg.transfer_fee,
        },
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Register(registration_msg) => execute_register(deps, info, registration_msg),
        ExecuteMsg::Transfer { name, to } => execute_transfer(deps, info, name, to),
        ExecuteMsg::SendFundsTo { name } => execute_send_funds_to(deps, info, name),
    }
}

fn execute_register(
    deps: DepsMut,
    info: MessageInfo,
    registration_msg: RegistrationMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    ensure_funds_paid(&info, config.registration_fee)?;

    registration_msg.validate()?;

    ensure!(
        !ENTRIES.has(deps.storage, &registration_msg.name),
        ContractError::Unauthorized {}
    );

    // store the entry
    ENTRIES.save(
        deps.storage,
        &registration_msg.name,
        &Entry {
            owner: info.sender,
            metadata: registration_msg.metadata,
        },
    )?;

    Ok(Response::new())
}

fn execute_transfer(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    to: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    ensure_funds_paid(&info, config.transfer_fee)?;

    let mut entry = ENTRIES.load(deps.storage, &name)?;

    // only owner can transfer
    ensure_eq!(info.sender, entry.owner, ContractError::Unauthorized {});

    entry.owner = deps.api.addr_validate(&to)?;

    ENTRIES.save(deps.storage, &name, &entry)?;

    Ok(Response::new())
}

fn execute_send_funds_to(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
) -> Result<Response, ContractError> {
    let entry = ENTRIES.load(deps.storage, &name)?;

    Ok(Response::new().add_message(BankMsg::Send {
        to_address: entry.owner.into_string(),
        amount: info.funds,
    }))
}

fn ensure_funds_paid(info: &MessageInfo, expected: Coin) -> Result<(), ContractError> {
    ensure_eq!(
        info.funds.len(),
        1,
        ContractError::InvalidRegistrationFee {
            given: info.funds.clone(),
            expected,
        }
    );
    ensure_eq!(
        info.funds[0],
        expected,
        ContractError::InvalidRegistrationFee {
            given: info.funds.clone(),
            expected,
        }
    );
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Lookup { name } => to_json_binary(&query_lookup(deps, name)?),
        QueryMsg::QueryValidatorInfo { name } => to_json_binary(&query_validator_info(deps, name)?),
        QueryMsg::List { start_after, limit } => {
            to_json_binary(&query_list(deps, start_after, limit)?)
        }
    }
}

const DEFAULT_LIMIT: u32 = 100u32;

fn query_list(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<EntryListResponse> {
    let entries: StdResult<Vec<_>> = ENTRIES
        .range(
            deps.storage,
            start_after
                .as_ref()
                .map(|name| Bound::exclusive(name.as_str())),
            None,
            Order::Ascending,
        )
        .take(limit.unwrap_or(DEFAULT_LIMIT) as usize)
        .collect();

    Ok(EntryListResponse { entries: entries? })
}

pub fn query_lookup(deps: Deps, name: String) -> StdResult<Entry> {
    let entry = ENTRIES.load(deps.storage, &name)?;
    Ok(entry)
}

pub fn query_validator_info(deps: Deps, name: String) -> Result<Validator, ContractError> {
    let entry = ENTRIES.load(deps.storage, &name)?;

    let validator_address = entry
        .metadata
        .validator_address
        .ok_or(ContractError::NotAValidator {})?;

    deps.querier
        .query_validator(validator_address)?
        .ok_or(ContractError::NotAValidator {})
}

#[cfg(test)]
mod tests {
    use crate::state::Metadata;

    use super::*;
    use cosmwasm_std::{coin, Empty};
    use cw_multi_test::{no_init, App, Contract, ContractWrapper, Executor};

    fn contract() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(execute, instantiate, query))
    }

    #[test]
    fn cannot_steal_entry() {
        let mut app = App::new(no_init);

        let user1 = app.api().addr_make("user1");
        let user2 = app.api().addr_make("user2");
        let fee = coin(100, "ucosm");

        app.init_modules(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &user1, vec![fee.clone()])
                .unwrap();
            router
                .bank
                .init_balance(storage, &user2, vec![fee.clone()])
                .unwrap();
        });

        let id = app.store_code(contract());

        let contract_addr = app
            .instantiate_contract(
                id,
                user1.clone(),
                &InstantiateMsg {
                    registration_fee: fee.clone(),
                    transfer_fee: fee.clone(),
                },
                &[],
                "name service",
                None,
            )
            .unwrap();

        // user1 registers
        app.execute_contract(
            user1.clone(),
            contract_addr.clone(),
            &ExecuteMsg::Register(RegistrationMsg {
                name: "myname".to_string(),
                metadata: Metadata {
                    url: None,
                    validator_address: None,
                },
            }),
            &[fee.clone()],
        )
        .unwrap();

        // user2 tries to register it
        app.execute_contract(
            user2.clone(),
            contract_addr.clone(),
            &ExecuteMsg::Register(RegistrationMsg {
                name: "myname".to_string(),
                metadata: Metadata {
                    url: None,
                    validator_address: None,
                },
            }),
            &[fee.clone()],
        )
        .unwrap_err();

        // user2 tries to transfer it
        app.execute_contract(
            user2.clone(),
            contract_addr.clone(),
            &ExecuteMsg::Transfer {
                name: "myname".to_string(),
                to: user2.to_string(),
            },
            &[fee.clone()],
        )
        .unwrap_err();
    }
}
