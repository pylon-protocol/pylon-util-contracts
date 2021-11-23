use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::*;
use std::collections::HashMap;
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

use crate::mock_tax::MockTax;

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, CustomMockQuerier> {
    let contract_addr = MOCK_CONTRACT_ADDR;
    let api = MockApi::default();

    OwnedDeps {
        storage: MockStorage::default(),
        api,
        querier: CustomMockQuerier::new(
            MockQuerier::new(&[(contract_addr, contract_balance)]),
            api,
        ),
    }
}

pub type WasmQueryHandler = fn(&Binary) -> StdResult<Binary>;

pub struct CustomMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    pub tax: MockTax,
    pub wasm_smart_query_handlers: HashMap<String, WasmQueryHandler>,
    pub wasm_raw_query_handlers: HashMap<String, WasmQueryHandler>,
}

impl Querier for CustomMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {:?}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl CustomMockQuerier {
    pub fn register_wasm_smart_query_handler(&mut self, address: &str, handler: WasmQueryHandler) {
        self.wasm_smart_query_handlers
            .insert(address.to_string(), handler);
    }

    pub fn register_wasm_raw_query_handler(&mut self, address: &str, handler: WasmQueryHandler) {
        self.wasm_raw_query_handlers
            .insert(address.to_string(), handler);
    }

    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if &TerraRoute::Treasury == route {
                    match query_data {
                        TerraQuery::TaxRate {} => {
                            let res = TaxRateResponse {
                                rate: self.tax.rate,
                            };
                            SystemResult::Ok(ContractResult::Ok(to_binary(&res).unwrap()))
                        }
                        TerraQuery::TaxCap { denom } => {
                            let cap = self.tax.caps.get(denom).copied().unwrap_or_default();
                            let res = TaxCapResponse { cap };
                            SystemResult::Ok(ContractResult::Ok(to_binary(&res).unwrap()))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(wasm_request) => match wasm_request {
                WasmQuery::Smart { contract_addr, msg } => SystemResult::Ok(ContractResult::Ok(
                    self.wasm_smart_query_handlers
                        .get(contract_addr.as_str())
                        .expect("wasm: smart query handler not found")(msg)
                    .unwrap(),
                )),
                WasmQuery::Raw { contract_addr, key } => SystemResult::Ok(ContractResult::Ok(
                    self.wasm_raw_query_handlers
                        .get(contract_addr.as_str())
                        .expect("wasm: raw query handler not found")(key)
                    .unwrap(),
                )),
                _ => SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: stringify!(request).to_string(),
                }),
            },
            _ => self.base.handle_query(request),
        }
    }
}

impl CustomMockQuerier {
    pub fn new<A: Api>(base: MockQuerier<TerraQueryWrapper>, _api: A) -> Self {
        CustomMockQuerier {
            base,
            tax: MockTax::default(),
            wasm_smart_query_handlers: Default::default(),
            wasm_raw_query_handlers: Default::default(),
        }
    }

    pub fn with_tax(&mut self, tax: MockTax) {
        self.tax = tax;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn register_handlers() {
        let mut deps = mock_dependencies(&[]);
        deps.querier
            .register_wasm_smart_query_handler(MOCK_CONTRACT_ADDR, |request_msg| {
                Ok(Binary::from(request_msg.as_slice()))
            });
        deps.querier
            .register_wasm_raw_query_handler(MOCK_CONTRACT_ADDR, |request_key| {
                Ok(Binary::from(request_key.as_slice()))
            });

        let response = deps
            .querier
            .handle_query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                msg: Binary::from(123u128.to_be_bytes()),
            }))
            .unwrap()
            .unwrap();
        let response = u128::from_be_bytes(response.as_slice().try_into().unwrap());
        assert_eq!(response, 123u128);

        let response = deps
            .querier
            .handle_query(&QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                key: Binary::from(123u128.to_be_bytes()),
            }))
            .unwrap()
            .unwrap();
        let response = u128::from_be_bytes(response.as_slice().try_into().unwrap());
        assert_eq!(response, 123u128);
    }
}
