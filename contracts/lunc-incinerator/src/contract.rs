use std::{fmt, vec};

use crate::error::ContractError;
use crate::msg::{CommunityRole, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, NONCE};
use bech32::ToBase32;
// use cosmwasm_crypto::secp256k1_recover_pubkey;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    coins, entry_point, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;
use ripemd::{Digest, Ripemd160};

/* Define contract name and version */
const CONTRACT_NAME: &str = "crates.io:lunc-inerator";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /* Define the initial configuration for this contract that way you can
    limit the type of coin you want to accept each time a lunc-inerator is
    created and also which kind of token would you like to mint based on
    the code id of the contract deployed */
    let state = Config {
        stable_denom: msg.stable_denom.to_string(),
        burn_address: "terra1sk06e3dyexuq4shw77y3dsv480xv42mq73anxu".to_string(),
        community_owner: msg.community_owner.to_string(),
        community_dev: msg.community_dev.to_string(),
        owner_recovery_param: 0xff,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("denom", msg.stable_denom)
        .add_attribute("community_owner", msg.community_owner)
        .add_attribute("community_dev", msg.community_dev))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => deposit(deps.as_ref(), env, info),
        ExecuteMsg::Withdraw {
            amount,
            recipient,
            sigature,
        } => withdraw(deps, env, info, recipient, amount, sigature),
        ExecuteMsg::ChangeCommunityInfo { role, value } => {
            change_community_info(deps, env, info, role, value)
        }
        ExecuteMsg::Burn { amount } => burn(deps, env, info, amount),
    }
}

pub fn deposit(deps: Deps, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    match info.funds.get(0) {
        None => return Err(ContractError::NotReceivedFunds {}),
        Some(received) => {
            /* Amount of tokens received cannot be zero */
            if received.amount.is_zero() {
                return Err(ContractError::NotAllowZeroAmount {});
            }

            /* Allow to receive only token denomination defined
            on contract instantiation "config.stable_denom" */
            if received.denom.ne(&config.stable_denom) {
                return Err(ContractError::NotAllowedDenom {
                    denom: received.denom.to_string(),
                });
            }

            /* Only one token can be received */
            if info.funds.len() > 1 {
                return Err(ContractError::NotAllowedMultipleDenoms {});
            }
            Ok(Response::new().add_attribute("amount", received.amount))
        }
    }
}

fn verify_signature(deps: Deps, payload: String, signature: String, owner_address: String) -> bool {
    for recovery_param in 0..2 {
        let payload_hash = hex::decode(sha256::digest(payload.clone())).unwrap();

        let key = deps
            .api
            .secp256k1_recover_pubkey(
                &payload_hash,
                &hex::decode(signature.clone()).unwrap(),
                recovery_param,
            )
            .unwrap();

        // compress key
        let mut compressed_key = vec![0u8; 33];
        if key[64] % 2 == 0 {
            compressed_key[0] = 2;
            compressed_key[1..33].clone_from_slice(&key[1..33]);
        } else {
            compressed_key[0] = 3;
            compressed_key[1..33].clone_from_slice(&key[33..65])
        }

        // get address from public key
        let mut hasher = Ripemd160::new();
        hasher.update(hex::decode(sha256::digest(&compressed_key[..])).unwrap());
        let encoded_address = bech32::encode(
            "terra",
            hasher.finalize().to_base32(),
            bech32::Variant::Bech32,
        )
        .unwrap();

        if owner_address.eq(&encoded_address) {
            return true;
        }
    }

    false
}

pub fn withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
    signature: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // check signature
    let mut nonce = NONCE.load(deps.storage, &info.sender).unwrap_or_default();

    let payload = fmt::format(format_args!("{0}|{1}|{2}", recipient, amount, nonce));

    if !verify_signature(deps.as_ref(), payload, signature, config.community_owner) {
        return Err(ContractError::Unauthorized {});
    }

    let sub_msg_send = SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.clone(),
        amount: coins(amount.u128(), config.stable_denom),
    }));

    // increase nonce
    nonce = nonce + 1;
    NONCE.save(deps.storage, &info.sender, &nonce)?;

    Ok(Response::new()
        .add_attribute("method", "withdraw")
        .add_submessages(vec![sub_msg_send]))
}

pub fn change_community_info(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    role: CommunityRole,
    value: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    match role {
        CommunityRole::Owner {} => {
            if !info.sender.to_string().eq(&config.community_owner) {
                return Err(ContractError::Unauthorized {});
            }
            config.community_owner = value;
            config.owner_recovery_param = 0xff;
        }
        CommunityRole::Developer {} => {
            if !info.sender.to_string().eq(&config.community_dev) {
                return Err(ContractError::Unauthorized {});
            }
            config.community_dev = value;
        }
    }
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("method", "change_community_info"))
}

pub fn burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if !info.sender.to_string().eq(&config.community_owner) {
        return Err(ContractError::Unauthorized {});
    }

    let burn_amount = amount / Uint128::from(2u32);

    let remaining_amount = amount - burn_amount;

    let dev_amount = remaining_amount / Uint128::from(10u32);
    let owner_amount = remaining_amount - dev_amount;

    let sub_msg_burn = SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
        to_address: config.burn_address,
        amount: coins(burn_amount.u128(), config.stable_denom.clone()),
    }));

    let sub_msg_owner = SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
        to_address: config.community_owner,
        amount: coins(owner_amount.u128(), config.stable_denom.clone()),
    }));

    let sub_msg_dev = SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
        to_address: config.community_dev,
        amount: coins(dev_amount.u128(), config.stable_denom.clone()),
    }));

    Ok(Response::new()
        .add_attribute("method", "burn")
        .add_submessages(vec![sub_msg_burn, sub_msg_owner, sub_msg_dev]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    match msg {
        QueryMsg::CommunityOwner {} => to_binary(&config.community_owner),
        QueryMsg::CommunityDeveloper {} => to_binary(&config.community_dev),
        QueryMsg::Nonce { address } => to_binary(
            &NONCE
                .load(deps.storage, &Addr::unchecked(address))
                .unwrap_or_default(),
        ),
    }
}

/* In case you want to upgrade this contract you can find information about
how to migrate the contract in the following link:
https://docs.terra.money/docs/develop/dapp/quick-start/contract-migration.html*/
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
