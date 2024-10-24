use std::sync::Arc;

use alloy::{
    network::TransactionBuilder,
    primitives::{aliases::U24, Address, U160, U256},
    providers::ProviderBuilder,
};
use alloy::{rpc::types::TransactionRequest, sol};
use anyhow::Result;
use std::str::FromStr;

// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    UNIV3_ROUTER,
    "src/abi/uniswapv3_router.json"
);

use crate::unswapv3_pool::UniswapPoolFee;
const UNIV3_ROUTER_CONTRACT_ADDR: &str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
pub struct ExactInputSingleParams {
    pub token_in: Address,
    pub token_out: Address,
    pub fee: UniswapPoolFee,
    pub recipient: Address,
    pub amount_in: U256,
    pub amount_out_minimum: U256,
    pub sqrt_price_limit_x96: U256,
}

impl TryFrom<ExactInputSingleParams>
    for crate::uniswapv3_router::IV3SwapRouter::ExactInputSingleParams
{
    type Error = UniswapV3RouterError;
    fn try_from(value: ExactInputSingleParams) -> std::result::Result<Self, Self::Error> {
        let val = crate::uniswapv3_router::IV3SwapRouter::ExactInputSingleParams {
            tokenIn: value.token_in,
            tokenOut: value.token_out,
            fee: U24::from(value.fee.as_u32()),
            recipient: value.recipient,
            amountIn: value.amount_in,
            amountOutMinimum: value.amount_out_minimum,
            sqrtPriceLimitX96: U160::from(value.sqrt_price_limit_x96),
        };
        Ok(val)
    }
}

pub struct ExactOutputSingleParams {
    pub token_in: Address,
    pub token_out: Address,
    pub fee: UniswapPoolFee,
    pub recipient: Address,
    pub amount_out: U256,
    pub amount_in_maximum: U256,
    pub sqrt_price_limit_x96: U256,
}

impl TryFrom<ExactOutputSingleParams>
    for crate::uniswapv3_router::IV3SwapRouter::ExactOutputSingleParams
{
    type Error = UniswapV3RouterError;
    fn try_from(value: ExactOutputSingleParams) -> std::result::Result<Self, Self::Error> {
        let val = crate::uniswapv3_router::IV3SwapRouter::ExactOutputSingleParams {
            tokenIn: value.token_in,
            tokenOut: value.token_out,
            fee: U24::from(value.fee.as_u32()),
            recipient: value.recipient,
            amountOut: value.amount_out,
            amountInMaximum: value.amount_in_maximum,
            sqrtPriceLimitX96: U160::from(value.sqrt_price_limit_x96),
        };
        Ok(val)
    }
}

pub enum UniswapV3RouterCommand {
    /// The swapExactInputSingle function is for performing exact input swaps, which swap a fixed amount of one token for a maximum possible amount of another toke
    ExactInputSingle(ExactInputSingleParams),
    /// The swapExactOutputSingle function is for performing exact output swaps, which swap a minimum possible amount of one token for a fixed amount of another token
    ExactOutputSingle(ExactOutputSingleParams),
}

pub enum UniswapV3RouterResult {
    ExactInputSingle(U256),
    ExactOutputSingle(U256),
}

#[derive(Debug, thiserror::Error)]
pub enum UniswapV3RouterError {
    #[error("RPC URL 格式不正确{0}")]
    InvalidRpcUrl(String),
    #[error("地址格式不正确{0}")]
    InvalidAddress(String),
    #[error("池子手续费不正确{0}")]
    WrongPoolFee(String),
}

pub async fn execute(
    command: UniswapV3RouterCommand,
    rpc_url: String,
) -> Result<TransactionRequest, UniswapV3RouterError> {
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_builtin(&rpc_url)
        .await
        .map_err(|e| UniswapV3RouterError::InvalidRpcUrl(e.to_string()))?;

    let client = Arc::new(provider);
    let router_address = Address::from_str(UNIV3_ROUTER_CONTRACT_ADDR)
        .map_err(|e| UniswapV3RouterError::InvalidAddress(e.to_string()))?;
    let contract = UNIV3_ROUTER::new(router_address, client);

    match command {
        UniswapV3RouterCommand::ExactInputSingle(params) => Ok(contract
            .exactInputSingle(params.try_into()?)
            .into_transaction_request()),
        UniswapV3RouterCommand::ExactOutputSingle(params) => Ok(contract
            .exactOutputSingle(params.try_into()?)
            .into_transaction_request()),
    }
}

// 0x35c8941c294E9d60E0742CB9f3d58c0D1Ba2DEc4
#[cfg(test)]
mod tests {

    use crate::utils::{from_readable_amount, Token};

    use super::*;

    #[tokio::test]
    pub async fn test_exact_input_single() {
        let rpc_url = "https://eth.llamarpc.com";
        let weth = Token::new_from_online("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", rpc_url)
            .await
            .unwrap();
        let ethc = Token::new_from_online("0x35c8941c294E9d60E0742CB9f3d58c0D1Ba2DEc4", rpc_url)
            .await
            .unwrap();
        let receiver = "0xCa017e24f449Ec454E94C843bbbF2cE61b7F6B69"
            .parse()
            .unwrap();
        let amount_in = from_readable_amount(0.02, weth.decimals);
        let params = ExactInputSingleParams {
            token_in: weth.address,
            token_out: ethc.address,
            fee: UniswapPoolFee::Fee10000,
            recipient: receiver,
            amount_in,
            amount_out_minimum: U256::ZERO,
            sqrt_price_limit_x96: U256::ZERO,
        };
        let res = execute(
            UniswapV3RouterCommand::ExactInputSingle(params),
            rpc_url.to_string(),
        )
        .await
        .unwrap();
        println!("{:?}", res);
    }
}
