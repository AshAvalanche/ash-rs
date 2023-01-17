// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains code to interact with Avalanche networks

use avalanche_types::ids::Id;
use ethers::providers::{Http, Provider};
use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};
use std::{collections::HashMap, convert::TryFrom, str::FromStr};

// Avalanche networks constants
pub const AVAX_PRIMARY_NETWORK_ID: &str = "11111111111111111111111111111111LpoYY";
pub const AVAX_MAINNET_CCHAIN_ID: &str = "2q9e4r6Mu3U68nU1fYjgbR6JvwrRx36CohpAX5UQxse55x1Q5";
pub const AVAX_MAINNET_CCHAIN_RPC: &str = "https://api.avax.network/ext/bc/C/rpc";
pub const AVAX_FUJI_CCHAIN_ID: &str = "yH8D7ThNJkxmtkuv2jgBa4P1Rn3Qpr4pPr7QYNfcdoS6k6HWp";
pub const AVAX_FUJI_CCHAIN_RPC: &str = "https://api.avax-test.network/ext/bc/C/rpc";

// Avalanche network
#[derive(Debug, Serialize)]
pub struct AvalancheNetwork {
    pub name: String,
    // Map of <subnet ID, AvalancheSubnet>
    pub subnets: HashMap<String, AvalancheSubnet>,
}

// Avalanche Subnet
#[derive(Debug, Serialize)]
pub struct AvalancheSubnet {
    pub id: Id,
    // Map of <blockchain ID, AvalancheBlockchain>
    pub blockchains: HashMap<String, AvalancheBlockchain>,
}

// Different Avalanche blockchains types
#[derive(Debug)]
pub enum AvalancheBlockchain {
    Evm {
        name: String,
        id: Id,
        provider: Provider<Http>,
    },
}

// Implement the Serialize trait for AvalancheBlockchain because Provider is not serializable
impl Serialize for AvalancheBlockchain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AvalancheBlockchain::Evm { name, id, provider } => {
                let mut state = serializer.serialize_struct("AvalancheBlockchain", 5)?;
                state.serialize_field("name", name)?;
                state.serialize_field("id", &id.to_string())?;
                state.serialize_field("type", "EVM")?;
                state.serialize_field("provider", "HTTP")?;
                state.serialize_field("url", &provider.url().to_string())?;
                state.end()
            }
        }
    }
}

impl AvalancheNetwork {
    // Create a new AvalancheNetwork
    pub fn new(network: &str) -> Result<AvalancheNetwork, String> {
        match network {
            "mainnet" => {
                // Never fails as AVAX_*_ID are valid Avalanche IDs
                let primary_network_id = Id::from_str(AVAX_PRIMARY_NETWORK_ID).unwrap();
                let cchain_id = Id::from_str(AVAX_MAINNET_CCHAIN_ID).unwrap();

                Ok(AvalancheNetwork {
                    name: "mainnet".to_string(),
                    subnets: HashMap::from([(
                        primary_network_id.to_string(),
                        AvalancheSubnet {
                            id: primary_network_id,
                            blockchains: HashMap::from([(
                                cchain_id.to_string(),
                                AvalancheBlockchain::Evm {
                                    // TODO: Get the name from the RPC
                                    name: "C-Chain".to_string(),
                                    id: cchain_id,
                                    // Never fails as AVAX_MAINNET_CCHAIN_RPC is a valid RPC URL
                                    provider: Provider::try_from(AVAX_MAINNET_CCHAIN_RPC).unwrap(),
                                },
                            )]),
                        },
                    )]),
                })
            }
            "fuji" => {
                // Never fails as AVAX_*_ID are valid Avalanche IDs
                let primary_network_id = Id::from_str(AVAX_PRIMARY_NETWORK_ID).unwrap();
                let cchain_id = Id::from_str(AVAX_FUJI_CCHAIN_ID).unwrap();

                Ok(AvalancheNetwork {
                    name: "fuji".to_string(),
                    subnets: HashMap::from([(
                        primary_network_id.to_string(),
                        AvalancheSubnet {
                            id: primary_network_id,
                            blockchains: HashMap::from([(
                                cchain_id.to_string(),
                                AvalancheBlockchain::Evm {
                                    // TODO: Get the name from the RPC
                                    name: "C-Chain".to_string(),
                                    id: cchain_id,
                                    // Never fails as AVAX_FUJI_CCHAIN_RPC is a valid RPC URL
                                    provider: Provider::try_from(AVAX_FUJI_CCHAIN_RPC).unwrap(),
                                },
                            )]),
                        },
                    )]),
                })
            }
            _ => Err(format!("'{}' is not a valid Avalanche network", network)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_avalanche_network_new() {
        // Only test the mainnet network as the fuji network is the same structurally
        let mainnet = AvalancheNetwork::new("mainnet").unwrap();
        assert_eq!(mainnet.name, "mainnet");
        assert_eq!(mainnet.subnets.len(), 1);

        // Should never fail as AVAX_PRIMARY_NETWORK_ID should always be a valid key
        let AvalancheSubnet { id, blockchains } =
            mainnet.subnets.get(AVAX_PRIMARY_NETWORK_ID).unwrap();
        assert_eq!(id.to_string(), AVAX_PRIMARY_NETWORK_ID);
        assert_eq!(blockchains.len(), 1);

        // Should never fail as AVAX_MAINNET_CCHAIN_ID should always be a valid key
        let AvalancheBlockchain::Evm { name, id, provider } =
            &blockchains.get(AVAX_MAINNET_CCHAIN_ID).unwrap();
        assert_eq!(name, "C-Chain");
        assert_eq!(id.to_string(), AVAX_MAINNET_CCHAIN_ID);
        assert_eq!(
            provider.url().to_string(),
            "https://api.avax.network/ext/bc/C/rpc"
        );

        assert!(AvalancheNetwork::new("invalid").is_err());
    }
}
