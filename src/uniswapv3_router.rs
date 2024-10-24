use std::sync::Arc;

use alloy::{
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
    pub deadline: U256,
    pub amount_in: U256,
    pub amount_out_minimum: U256,
    pub sqrt_price_limit_x96: U256,
}

impl TryFrom<ExactInputSingleParams>
    for crate::uniswapv3_router::ISwapRouter::ExactInputSingleParams
{
    type Error = UniswapV3RouterError;
    fn try_from(value: ExactInputSingleParams) -> std::result::Result<Self, Self::Error> {
        let val = crate::uniswapv3_router::ISwapRouter::ExactInputSingleParams {
            tokenIn: value.token_in,
            tokenOut: value.token_out,
            fee: U24::from(value.fee.as_u32()),
            recipient: value.recipient,
            deadline: value.deadline,
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
    pub deadline: U256,
    pub amount_out: U256,
    pub amount_in_maximum: U256,
    pub sqrt_price_limit_x96: U256,
}

impl TryFrom<ExactOutputSingleParams>
    for crate::uniswapv3_router::ISwapRouter::ExactOutputSingleParams
{
    type Error = UniswapV3RouterError;
    fn try_from(value: ExactOutputSingleParams) -> std::result::Result<Self, Self::Error> {
        let val = crate::uniswapv3_router::ISwapRouter::ExactOutputSingleParams {
            tokenIn: value.token_in,
            tokenOut: value.token_out,
            fee: U24::from(value.fee.as_u32()),
            recipient: value.recipient,
            deadline: value.deadline,
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

    use alloy::hex;

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
            deadline: U256::ZERO,
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

    /// https://etherscan.io/tx/0x9f130d339b1b444c86593603ce5ade9f4edb7dc2e181b068eb925a5bec1101b3
    #[tokio::test]
    pub async fn test_exact_input_single_eg2() {
        let rpc_url = "https://eth.llamarpc.com";
        let weth = Token::new_from_online("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", rpc_url)
            .await
            .unwrap();
        let ethc = Token::new_from_online("0x35c8941c294E9d60E0742CB9f3d58c0D1Ba2DEc4", rpc_url)
            .await
            .unwrap();
        let receiver = "0x1a2f6A0AC3646992E4864159C41bC38990424CA3"
            .parse()
            .unwrap();
        let amount_in = from_readable_amount(200.0, ethc.decimals);
        let params = ExactInputSingleParams {
            token_in: ethc.address,
            token_out: weth.address,
            fee: UniswapPoolFee::Fee10000,
            recipient: receiver,
            deadline: U256::from(1729779337),
            amount_in,
            amount_out_minimum: U256::from(47639961375419603i64),
            sqrt_price_limit_x96: U256::ZERO,
        };
        let res = execute(
            UniswapV3RouterCommand::ExactInputSingle(params),
            rpc_url.to_string(),
        )
        .await
        .unwrap();
        let input = res.input;
        let data = input.input.unwrap();
        assert_eq!("414bf38900000000000000000000000035c8941c294e9d60e0742cb9f3d58c0d1ba2dec4000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc200000000000000000000000000000000000000000000000000000000000027100000000000000000000000001a2f6a0ac3646992e4864159c41bc38990424ca300000000000000000000000000000000000000000000000000000000671a568900000000000000000000000000000000000000000000000ad78ebc5ac620000000000000000000000000000000000000000000000000000000a9404adee52cd30000000000000000000000000000000000000000000000000000000000000000", hex::encode(data));
    }

    //https://etherscan.io/tx/0x8e3bfac43676b709137a230bf989bbd63c759cc6694d997b1a64e50492855aeb
    #[tokio::test]
    pub async fn test_exact_output_single_() {
        let rpc_url = "https://eth.llamarpc.com";
        let weth = Token::new_from_online("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", rpc_url)
            .await
            .unwrap();
        let usdc = Token::new_from_online("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", rpc_url)
            .await
            .unwrap();
        let receiver = "0xf5213a6a2f0890321712520b8048D9886c1A9900"
            .parse()
            .unwrap();
        let amount_out = from_readable_amount(13148.9, usdc.decimals);
        let params = ExactOutputSingleParams {
            token_in: weth.address,
            token_out: usdc.address,
            fee: UniswapPoolFee::Fee100,
            recipient: receiver,
            deadline: U256::from(1729776724),
            amount_out,
            amount_in_maximum: U256::from(5201336670800000000i64),
            sqrt_price_limit_x96: U256::ZERO,
        };
        let res = execute(
            UniswapV3RouterCommand::ExactOutputSingle(params),
            rpc_url.to_string(),
        )
        .await
        .unwrap();
        let input = res.input;
        let data = input.input.unwrap();
        assert_eq!("db3e2198000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb480000000000000000000000000000000000000000000000000000000000000064000000000000000000000000f5213a6a2f0890321712520b8048d9886c1a990000000000000000000000000000000000000000000000000000000000671a4c54000000000000000000000000000000000000000000000000000000030fbc4aa0000000000000000000000000000000000000000000000000482edc24ec3f34000000000000000000000000000000000000000000000000000000000000000000", hex::encode(data));
    }
}
