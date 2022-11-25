use std::{fmt, vec};

use crate::error::ContractError;
use crate::msg::{CommunityRole, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG, NONCE};
// use bech32::ToBase32;
// use cosmwasm_crypto::secp256k1_recover_pubkey;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;
// use secp256k1::ecdsa::{RecoverableSignature, RecoveryId};
// use secp256k1::Secp256k1;
// use ripemd::{Digest, Ripemd160};
// use secp256k1::{
//     hashes::{hash160, sha256, Hash},
//     Message,
// };

// use sha256::digest;

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
    // let mut nonce = NONCE.load(deps.storage, &info.sender).unwrap_or_default();

    // let payload = fmt::format(format_args!("{0}|{1}|{2}", recipient, amount, nonce));

    // // let payload_hash = hex::decode(digest(payload)).unwrap();

    // let message = Message::from_hashed_data::<sha256::Hash>(payload.as_bytes());

    // let engine = Secp256k1::verification_only();

    // let recoverable = RecoverableSignature::from_compact(
    //     &hex::decode(signature).unwrap(),
    //     RecoveryId::from_i32(0).unwrap(),
    // )
    // .unwrap();

    // let key = engine.recover_ecdsa(&message, &recoverable).unwrap();

    // // let key = hex::decode(
    // //     PublicKey::from_slice(
    // //         &secp256k1_recover_pubkey(&payload_hash, &hex::decode(signature).unwrap(), 0).unwrap(),
    // //     )
    // //     .unwrap()
    // //     .to_string(),
    // // )
    // // .unwrap();

    // let hash = hash160::Hash::hash(&hex::decode(key.to_string()).unwrap());

    // let encoded_address =
    //     bech32::encode("terra", (&hash[..]).to_base32(), bech32::Variant::Bech32).unwrap();

    // if !config.community_owner.eq(&encoded_address) {
    //     return Err(ContractError::Unauthorized {});
    // }

    let sub_msg_send = SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.clone(),
        amount: coins(amount.u128(), config.stable_denom),
    }));

    // nonce = nonce + 1;

    // NONCE.save(deps.storage, &info.sender, &nonce)?;

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
