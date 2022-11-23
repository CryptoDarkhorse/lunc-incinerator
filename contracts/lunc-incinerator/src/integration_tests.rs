#[cfg(test)]
mod tests {
    use crate::{
        contract::{execute, instantiate},
        msg::{ExecuteMsg, InstantiateMsg},
        ContractError,
    };
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Attribute, Coin, DepsMut, Response, Uint128,
    };

    const OWNER: &str = "OWNER";
    const DEVEL: &str = "DEVEL";

    const USER: &str = "USER";
    // const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "uluna";
    const TOKEN_DENOM: &str = "ustc";

    fn do_instantiate(deps: DepsMut) -> Response {
        let instantiate_msg = InstantiateMsg {
            stable_denom: NATIVE_DENOM.to_string(),
            community_owner: OWNER.to_string(),
            community_dev: DEVEL.to_string(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();

        instantiate(deps, env, info, instantiate_msg).unwrap()
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies(&[]);

        let res = do_instantiate(deps.as_mut());

        let attrs = res.attributes;
        assert_eq!(
            vec![
                Attribute {
                    key: "method".to_string(),
                    value: "instantiate".to_string()
                },
                Attribute {
                    key: "denom".to_string(),
                    value: NATIVE_DENOM.to_string()
                },
                Attribute {
                    key: "community_owner".to_string(),
                    value: OWNER.to_string()
                },
                Attribute {
                    key: "community_dev".to_string(),
                    value: DEVEL.to_string()
                }
            ],
            attrs
        );
    }

    #[test]
    fn test_deposit() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();
        let msg = ExecuteMsg::Deposit {};

        do_instantiate(deps.as_mut());

        let empty_funds = mock_info(USER, &[]);
        let valid_funds = mock_info(
            USER,
            &[Coin {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(1),
            }],
        );
        let invalid_funds = mock_info(
            USER,
            &[Coin {
                denom: TOKEN_DENOM.to_string(),
                amount: Uint128::new(1),
            }],
        );

        let complex_funds = mock_info(
            USER,
            &[
                Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(1),
                },
                Coin {
                    denom: TOKEN_DENOM.to_string(),
                    amount: Uint128::new(1),
                },
            ],
        );

        let e = execute(deps.as_mut(), env.clone(), empty_funds, msg.clone()).unwrap_err();
        assert_eq!(e, ContractError::NotReceivedFunds {});

        let e = execute(deps.as_mut(), env.clone(), invalid_funds, msg.clone()).unwrap_err();
        assert_eq!(
            e,
            ContractError::NotAllowedDenom {
                denom: TOKEN_DENOM.to_string()
            }
        );
        let e = execute(deps.as_mut(), env.clone(), complex_funds, msg.clone()).unwrap_err();
        assert_eq!(e, ContractError::NotAllowedMultipleDenoms {});

        let res = execute(deps.as_mut(), env.clone(), valid_funds, msg.clone()).unwrap();
        assert_eq!(1, res.attributes.len());
        assert_eq!(
            vec![Attribute {
                key: "amount".to_string(),
                value: 1.to_string()
            },],
            res.attributes
        )
    }
}
