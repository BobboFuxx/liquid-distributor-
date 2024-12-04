use cosmwasm_std::{coins, Addr, BlockInfo, DepsMut, Env, MessageInfo, Response, Uint128, testing::{mock_env, mock_info}};
use cw20::{Cw20ExecuteMsg};
use crate::{contract::{instantiate, execute, distribute_tokens}, state::{TOTAL_DISTRIBUTED_PRYSM, TOTAL_DISTRIBUTED_BTC, LAST_DISTRIBUTION_BLOCK}};
use crate::msg::{InstantiateMsg, ExecuteMsg};
use cw_storage_plus::Item;

// Mock staking contract
struct MockStakingContract;

impl MockStakingContract {
    fn new() -> Self {
        MockStakingContract
    }

    fn query_total_staked(&self) -> u128 {
        // Return some mock total staked amount
        100_000_000_000_000_000_000
    }

    fn query_stakers(&self) -> Vec<String> {
        // Return some mock list of stakers
        vec!["staker1".to_string(), "staker2".to_string()]
    }

    fn query_staker_balance(&self, staker: String) -> u128 {
        // Return some mock balance for each staker
        match staker.as_str() {
            "staker1" => 50_000_000_000_000_000_000, // 50% of the stake
            "staker2" => 50_000_000_000_000_000_000, // 50% of the stake
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{Addr, Uint128, BlockInfo};
    use crate::state::{TOTAL_DISTRIBUTED_PRYSM, TOTAL_DISTRIBUTED_BTC, LAST_DISTRIBUTION_BLOCK};

    // Mock for the staking contract
    fn mock_staking_contract() -> MockStakingContract {
        MockStakingContract::new()
    }

    // Helper function to set up the contract and instantiate it
    fn setup_contract() -> (DepsMut, Env, MessageInfo, MockStakingContract) {
        let staking_contract = mock_staking_contract();
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            liquid_prysm_address: "liquid_prysm".to_string(),
            liquid_btc_address: "liquid_btc".to_string(),
            staking_contract_address: "staking_contract".to_string(),
        };

        let info = mock_info("creator", &coins(2, "token"));
        let env = mock_env();
        
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        
        (deps, env, info, staking_contract)
    }

    // Test if the token distribution occurs correctly
    #[test]
    fn test_distribute_tokens() {
        let (mut deps, env, info, staking_contract) = setup_contract();

        // Simulate the environment to be past the distribution interval
        let env = mock_env();
        env.block.height = 57601; // Make sure the block height is past the distribution interval

        // Simulate a valid distribution
        let msg = ExecuteMsg::DistributeTokens {};
        let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
        
        // Check that the response contains the expected action
        assert_eq!(res.attributes[0].value, "distribute_tokens");

        // Ensure tokens were distributed to stakers (e.g., total distributed amounts should change)
        let total_distributed_prysm = TOTAL_DISTRIBUTED_PRYSM.load(deps.storage).unwrap();
        let total_distributed_btc = TOTAL_DISTRIBUTED_BTC.load(deps.storage).unwrap();

        // Validate the total distributed amount (mocked stake proportion 50% each)
        assert_eq!(total_distributed_prysm, 100_000_000_000_000_000_000); // 100% of total staked tokens
        assert_eq!(total_distributed_btc, 100_000_000_000_000_000_000); // 100% of total staked tokens
    }

    // Test if the distribution respects the token cap
    #[test]
    fn test_distribution_cap()
