pub enum UniswapPoolFee {
    Fee10000, // 1%
    Fee3000,  // 0.3%
    Fee500,   // 0.05%
    Fee100,   // 0.01%
}
impl UniswapPoolFee {
    pub fn as_u32(&self) -> u32 {
        match self {
            UniswapPoolFee::Fee10000 => 10000,
            UniswapPoolFee::Fee3000 => 3000,
            UniswapPoolFee::Fee500 => 500,
            UniswapPoolFee::Fee100 => 100,
        }
    }
}
