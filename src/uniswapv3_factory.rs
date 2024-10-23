use std::sync::Arc;

use anyhow::Result;
use ethers::{
    prelude::abigen,
    providers::{Http, Provider},
    types::{Address, U256},
};
use std::str::FromStr;
abigen!(UNIV3_FACTORY, "src/abi/uniswapv3_factory.json");
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
pub async fn execute(
    command: UniswapV3FactoryCommand,
    rpc_url: String,
) -> Result<UniswapV3FactoryResult> {
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);
    let factory_address = Address::from_str(UNIV3_FACTORY_CONTRACT_ADDR).unwrap();
    match command {
        UniswapV3FactoryCommand::GetPool(params) => {
            let contract = UNIV3_FACTORY::new(factory_address, client);
            let pool_address = contract
                .get_pool(params.token_a, params.token_b, params.fee)
                .call()
                .await?;
            Ok(UniswapV3FactoryResult::GetPool(pool_address))
        }
        _ => Err(anyhow::anyhow!("invalid command")),
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
