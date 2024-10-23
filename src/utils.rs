use std::sync::Arc;

use anyhow::Result;
use ethers::{
    contract::abigen,
    providers::{Http, Provider},
    types::{Address, U256},
};
use std::str::FromStr;
abigen!(ERC20, "src/abi/erc20.json");
pub fn from_readable_amount(amount_in: f64, decimals: u8) -> U256 {
    U256::from((amount_in * 10_f64.powi(decimals as i32)) as u128)
}
pub fn to_readable_amount(amount_in: U256, decimals: u8) -> f64 {
    amount_in.as_u128() as f64 / 10_f64.powi(decimals as i32)
}

#[derive(Debug, Clone)]
pub struct Token {
    pub address: Address,
    pub decimals: u8,
    pub token_name: String,
}

impl Token {
    pub fn new(address: &str, decimals: u8, token_name: String) -> Self {
        let address = Address::from_str(address).unwrap();
        Token {
            address,
            decimals,
            token_name,
        }
    }
    pub async fn new_from_online(address: &str, rpc_url: &str) -> Result<Self> {
        let address = Address::from_str(address).unwrap();
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let client = Arc::new(provider);
        let contract = ERC20::new(address, client);
        let name: String = contract.method("name", ())?.call().await?;
        let decimals: u8 = contract.method("decimals", ())?.call().await?;

        Ok(Self {
            address,
            decimals,
            token_name: name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_readable_amount() {
        let amount_in = from_readable_amount(1.0, 18);
        assert_eq!(U256::from(1000000000000000000u128), amount_in);
    }

    #[test]
    fn test_to_readable_amount() {
        let human_readable_num = to_readable_amount(U256::from(1000000000000000000u128), 18);
        assert_eq!(1.0, human_readable_num)
    }

    #[tokio::test]
    pub async fn test_get_token_info() {
        let mainet_rpc = "https://eth.llamarpc.com";
        let token =
            Token::new_from_online("0xA35923162C49cF95e6BF26623385eb431ad920D3", mainet_rpc)
                .await
                .unwrap();
        assert_eq!(
            token.address,
            "0xa35923162c49cf95e6bf26623385eb431ad920d3"
                .parse()
                .unwrap()
        );
        assert_eq!(token.decimals, 18);
        assert_eq!(token.token_name, "Turbo");
    }
}
