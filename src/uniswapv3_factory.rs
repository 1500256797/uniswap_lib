use std::sync::Arc;

use anyhow::Result;

use alloy::{
    primitives::{aliases::U24, Address, U160, U256},
    providers::ProviderBuilder,
};
use alloy::{rpc::types::error, sol};
use std::str::FromStr;

use crate::unswapv3_pool::UniswapPoolFee;
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    UNIV3_FACTORY,
    "src/abi/uniswapv3_factory.json"
);
const UNIV3_FACTORY_CONTRACT_ADDR: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";

pub struct GetPoolParam {
    pub token_a: Address,
    pub token_b: Address,
    pub fee: u32,
}

pub enum UniswapV3FactoryCommand {
    GetPool(GetPoolParam),
}
pub enum UniswapV3FactoryResult {
    GetPool(Address),
}

#[derive(Debug, thiserror::Error)]
pub enum UniswapV3FactoryError {
    #[error("请替换rpc url{0}")]
    InvalidRpcUrl(String),

    #[error("无法查询pool address，请确认是否开盘或poolfee设置是否正确{0}")]
    GetPoolError(String),

    #[error("无效的命令")]
    InvalidCommand,
}

pub async fn execute(
    command: UniswapV3FactoryCommand,
    rpc_url: String,
) -> Result<UniswapV3FactoryResult, UniswapV3FactoryError> {
    let provider = ProviderBuilder::new()
        .on_builtin(&rpc_url)
        .await
        .map_err(|e| UniswapV3FactoryError::InvalidRpcUrl(e.to_string()))?;

    let client = Arc::new(provider);
    let factory_address = Address::from_str(UNIV3_FACTORY_CONTRACT_ADDR).unwrap();
    match command {
        UniswapV3FactoryCommand::GetPool(params) => {
            let contract = UNIV3_FACTORY::new(factory_address, client);
            let pool_address = contract
                .getPool(params.token_a, params.token_b, U24::from(params.fee))
                .call()
                .await
                .map_err(|e| UniswapV3FactoryError::GetPoolError(e.to_string()))?;
            Ok(UniswapV3FactoryResult::GetPool(pool_address._0))
        }
        _ => Err(UniswapV3FactoryError::InvalidCommand),
    }
}

#[cfg(test)]
mod tests {

    use crate::unswapv3_pool::UniswapPoolFee;

    use super::*;

    #[tokio::test]
    pub async fn test_get_pool_address_online() {
        let token_a = Address::from_str("0x535887989b9EdffB63b1Fd5C6b99a4d45443b49a").unwrap();
        let weth = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();

        let get_pool = UniswapV3FactoryCommand::GetPool(GetPoolParam {
            token_a,
            token_b: weth,
            fee: UniswapPoolFee::Fee10000.as_u32(),
        });

        let res = execute(get_pool, "https://eth.llamarpc.com".to_string())
            .await
            .unwrap();
        if let UniswapV3FactoryResult::GetPool(pool_address) = res {
            assert_eq!(
                Address::from_str("0xFbDbaC2d456A3CC2754A626C2fB83C1af25A3a6F").unwrap(),
                pool_address
            );
        }
    }
}
