use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U160, U256},
    providers::ProviderBuilder,
};
// send a swap transaction
use anyhow::{Ok, Result};

use crate::{
    uniswapv3_router::{self, ExactInputSingleParams, UniswapV3RouterCommand},
    unswapv3_pool::UniswapPoolFee,
};
pub enum UniswapSupportChain {
    Ethereum,
    Base,
}

impl UniswapSupportChain {
    pub fn get_rpc_url(&self) -> String {
        match self {
            UniswapSupportChain::Ethereum => "https://mainnet.base.org".to_string(),
            UniswapSupportChain::Base => "https://mainnet.base.org".to_string(),
        }
    }

    pub fn as_chain_id(&self) -> u64 {
        match self {
            UniswapSupportChain::Ethereum => 1,
            UniswapSupportChain::Base => 8453,
        }
    }
}

pub enum SwapDirection {
    ExactInput,
    ExactOutput,
}

pub enum UniswapVersion {
    V2,
    V3,
}

pub struct SwapParams {
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub amount_out_min: U256,
    pub pool_fee: UniswapPoolFee,
    pub recipient: Address,
    pub deadline: U256,
}

pub async fn swap(
    chain: UniswapSupportChain,
    direction: SwapDirection,
    uniswap_version: UniswapVersion,
    params: SwapParams,
    rpc_url: String,
) -> Result<()> {
    match uniswap_version {
        UniswapVersion::V2 => {
            // V2 逻辑
            Ok(())
        }
        UniswapVersion::V3 => {
            // 判断是 ExactInput 还是 ExactOutput
            match direction {
                SwapDirection::ExactInput => {
                    let params = ExactInputSingleParams {
                        token_in: params.token_in,
                        token_out: params.token_out,
                        fee: params.pool_fee,
                        recipient: params.recipient,
                        deadline: params.deadline,
                        amount_in: params.amount_in,
                        amount_out_minimum: params.amount_out_min,
                        sqrt_price_limit_x96: U256::from(0),
                    };
                    let tx = crate::uniswapv3_router::execute(
                        UniswapV3RouterCommand::ExactInputSingle(params),
                        rpc_url,
                    )
                    .await?;
                    let tx = tx.with_chain_id(chain.as_chain_id());
                    println!("tx: {:?}", tx);
                    Ok(())
                }
                SwapDirection::ExactOutput => Ok(()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::providers::ProviderBuilder;

    use crate::utils::{from_readable_amount, Token};

    use super::*;

    #[tokio::test]
    async fn test_swap() {
        let mainet_rpc = "https://eth.llamarpc.com";
        // Create a provider with the wallet.

        let weth = Token::new_from_online("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", mainet_rpc)
            .await
            .unwrap();
        let ethc = Token::new_from_online("0x35c8941c294E9d60E0742CB9f3d58c0D1Ba2DEc4", mainet_rpc)
            .await
            .unwrap();
        let receiver: Address = "0xCa017e24f449Ec454E94C843bbbF2cE61b7F6B69"
            .parse()
            .unwrap();
        let params = SwapParams {
            token_in: weth.address,
            token_out: ethc.address,
            amount_in: from_readable_amount(0.01, weth.decimals),
            amount_out_min: U256::ZERO,
            pool_fee: UniswapPoolFee::Fee10000,
            recipient: receiver,
            deadline: U256::ZERO,
        };
        let tx = swap(
            UniswapSupportChain::Ethereum,
            SwapDirection::ExactInput,
            UniswapVersion::V3,
            params,
            mainet_rpc.to_string(),
        )
        .await
        .unwrap();
    }
}
