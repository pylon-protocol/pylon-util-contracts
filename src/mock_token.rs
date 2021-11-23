use crate::mock_querier::CustomMockQuerier;
use cosmwasm_std::{
    from_binary, to_binary, ContractResult, QuerierResult, StdError, SystemError, SystemResult,
    Uint128,
};
use cw20::{BalanceResponse, Cw20QueryMsg, TokenInfoResponse};
use std::collections::HashMap;

#[derive(Clone)]
pub struct MockToken {
    pub info: TokenInfoResponse,
    pub address: String,
    pub balances: HashMap<String, Uint128>,
}

impl Default for MockToken {
    fn default() -> Self {
        MockToken::new(
            "",
            TokenInfoResponse {
                name: "".to_string(),
                symbol: "".to_string(),
                decimals: 0,
                total_supply: Default::default(),
            },
            &[],
        )
    }
}

impl MockToken {
    pub fn register(&self, querier: &mut CustomMockQuerier) {
        querier.register_wasm_raw_query_handler(self.address.as_str(), |request| {
            let request: Cw20QueryMsg = from_binary(request).unwrap();
            match request {
                Cw20QueryMsg::Balance { address } => to_binary(&BalanceResponse {
                    balance: self
                        .balances
                        .get(address.as_str())
                        .map(|x| *x)
                        .unwrap_or(Uint128::zero()),
                }),
                Cw20QueryMsg::TokenInfo {} => to_binary(&self.info),
                _ => return Err(StdError::not_found(format!("{:?}", request))),
            }
        })
    }
}

impl MockToken {
    pub fn new(address: &str, info: TokenInfoResponse, balances: &[(&String, &Uint128)]) -> Self {
        MockToken {
            info,
            address: address.to_string(),
            balances: balances_to_map(balances),
        }
    }
}

pub fn balances_to_map(balances: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut balance_map: HashMap<String, Uint128> = HashMap::new();
    for (owner, balance) in balances.iter() {
        balance_map.insert(owner.to_string(), **balance);
    }
    balance_map
}

pub fn register_token(
    querier: &mut CustomMockQuerier,
    address: &str,
    info: TokenInfoResponse,
    balances: &[(&String, &Uint128)],
) {
    MockToken::new(address, info, balances).register(querier)
}
