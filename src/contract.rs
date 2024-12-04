use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, StdResult, to_binary, Uint128};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_storage_plus::{Item, Map};
use crate::state::{LAST_DISTRIBUTION_BLOCK, TOTAL_DISTRIBUTED_PRYSM, TOTAL_DISTRIBUTED_BTC};
use crate::msg::{ExecuteMsg, InstantiateMsg};

const LIQUID_PRYSM_CAP: u128 = 1_500_000 * 1_000_000_000_000_000_000; // 1.5M tokens, 18 decimals
const LIQUID_BTC_CAP: u128 = 750_000 * 1_000_000_000_000_000_000; // 750K tokens, 18 decimals
const DISTRIBUTION_INTERVAL: u64 = 57600; // 57600 blocks for 24 hours (1.5s block time)

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // Initialize storage with provided addresses
    deps.api.set_storage_value("liquid_prysm_address", msg.liquid_prysm_address.as_bytes());
    deps.api.set_storage_value("liquid_btc_address", msg.liquid_btc_address.as_bytes());
    deps.api.set_storage_value("staking_contract_address", msg.staking_contract_address.as_bytes());

    // Initialize distributed amounts to 0
    TOTAL_DISTRIBUTED_PRYSM.save(deps.storage, &0)?;
    TOTAL_DISTRIBUTED_BTC.save(deps.storage, &0)?;

    // Initialize last distribution block to 0
    LAST_DISTRIBUTION_BLOCK.save(deps.storage, &0)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::DistributeTokens {} => distribute_tokens(deps, env, info),
    }
}

fn distribute_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    // Fetch the last distribution block
    let last_distribution_block = LAST_DISTRIBUTION_BLOCK.load(deps.storage)?;

    // Ensure that the distribution is happening after the required interval (24 hours)
    if env.block.height < last_distribution_block + DISTRIBUTION_INTERVAL {
        return Err(StdError::generic_err("Distribution interval not reached"));
    }

    // Fetch total staked tokens and stakers from the staking contract
    let total_staked: u128 = query_total_staked(deps, env.clone())?; // Query the staking contract for total staked amount
    let stakers = query_stakers(deps, env.clone())?; // Query the staking contract for a list of stakers

    // Calculate the reward per staker
    let total_reward_prysm = total_staked;
    let total_reward_btc = total_staked;

    // Ensure that the distribution does not exceed the caps
    let total_distributed_prysm = TOTAL_DISTRIBUTED_PRYSM.load(deps.storage)?;
    let total_distributed_btc = TOTAL_DISTRIBUTED_BTC.load(deps.storage)?;

    if total_distributed_prysm + total_reward_prysm > LIQUID_PRYSM_CAP {
        return Err(StdError::generic_err("Exceeded liquidPRYSM cap"));
    }
    if total_distributed_btc + total_reward_btc > LIQUID_BTC_CAP {
        return Err(StdError::generic_err("Exceeded liquidBTC cap"));
    }

    // Mint the tokens and distribute them to stakers
    for staker in stakers {
        let staker_balance = query_staker_balance(deps, staker.clone())?; // Query staking contract for each staker's balance

        // Calculate reward for this staker based on their stake proportion
        let reward_prysm = (staker_balance * total_reward_prysm) / total_staked;
        let reward_btc = (staker_balance * total_reward_btc) / total_staked;

        // Mint and send tokens to the staker
        mint_and_send_tokens(deps, &staker, reward_prysm, reward_btc)?;
    }

    // Update the total distributed amounts
    TOTAL_DISTRIBUTED_PRYSM.save(deps.storage, &(total_distributed_prysm + total_reward_prysm))?;
    TOTAL_DISTRIBUTED_BTC.save(deps.storage, &(total_distributed_btc + total_reward_btc))?;

    // Update the last distribution block
    LAST_DISTRIBUTION_BLOCK.save(deps.storage, &env.block.height)?;

    Ok(Response::new().add_attribute("action", "distribute_tokens"))
}

// Example of minting and sending the tokens to a staker (CW20-based contract)
fn mint_and_send_tokens(
    deps: DepsMut,
    recipient: &str,
    amount_prysm: u128,
    amount_btc: u128,
) -> StdResult<()> {
    // Mint liquidPRYSM tokens to the staker
    let liquid_prysm_address = deps.api.get_storage_value("liquid_prysm_address")?;
    let msg = Cw20ExecuteMsg::Mint {
        recipient: recipient.to_string(),
        amount: Uint128::from(amount_prysm),
    };
    deps.api.execute_contract(&liquid_prysm_address, &msg)?;

    // Mint liquidBTC tokens to the staker
    let liquid_btc_address = deps.api.get_storage_value("liquid_btc_address")?;
    let msg = Cw20ExecuteMsg::Mint {
        recipient: recipient.to_string(),
        amount: Uint128::from(amount_btc),
    };
    deps.api.execute_contract(&liquid_btc_address, &msg)?;

    Ok(())
}
