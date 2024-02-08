// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche nodes

use crate::{avalanche::jsonrpc::info::*, errors::*};
pub use avalanche_types::key::bls::{private_key::Key as BlsPrivateKey, ProofOfPossession};
use avalanche_types::{
    ids::node::Id as NodeId,
    jsonrpc::info::{GetNodeVersionResult, UptimeResult, VmVersions},
};
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, PKCS_RSA_SHA256};
use rustls_pemfile::certs;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// Avalanche node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheNode {
    pub id: NodeId,
    pub signer: Option<ProofOfPossession>,
    pub network: String,
    pub http_host: String,
    pub http_port: u16,
    pub https_enabled: bool,
    pub public_ip: IpAddr,
    pub staking_port: u16,
    pub versions: AvalancheNodeVersions,
    pub uptime: AvalancheNodeUptime,
}

impl Default for AvalancheNode {
    fn default() -> Self {
        Self {
            id: NodeId::default(),
            signer: None,
            network: String::from("local"),
            http_host: String::from("127.0.0.1"),
            http_port: 9650,
            https_enabled: false,
            public_ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            staking_port: 9651,
            versions: AvalancheNodeVersions::default(),
            uptime: AvalancheNodeUptime::default(),
        }
    }
}

impl AvalancheNode {
    /// Get the node HTTP endpoint URL
    /// This endpoint is used to call the node's JSON RPC APIs
    pub fn get_http_endpoint(&self) -> String {
        format!(
            "{api_scheme}://{http_host}:{http_port}",
            api_scheme = if self.https_enabled { "https" } else { "http" },
            http_host = self.http_host,
            http_port = self.http_port
        )
    }

    /// Update the node's information
    pub fn update_info(&mut self) -> Result<(), AshError> {
        let http_endpoint = self.get_http_endpoint();
        let api_path = format!("{}/{}", http_endpoint, AVAX_INFO_API_ENDPOINT);

        (self.id, self.signer) = get_node_id(&api_path).map_err(|e| RpcError::GetFailure {
            data_type: "ID".to_string(),
            target_type: "node".to_string(),
            target_value: http_endpoint.clone(),
            msg: e.to_string(),
        })?;

        // The get_node_ip() return has to be splited to get public_ip and stacking_port
        let node_ip = get_node_ip(&api_path).map_err(|e| RpcError::GetFailure {
            data_type: "node IP".to_string(),
            target_type: "node".to_string(),
            target_value: http_endpoint.clone(),
            msg: e.to_string(),
        })?;
        self.public_ip = node_ip.ip();
        self.staking_port = node_ip.port();

        self.versions = get_node_version(&api_path).map_err(|e| RpcError::GetFailure {
            data_type: "version".to_string(),
            target_type: "node".to_string(),
            target_value: http_endpoint.clone(),
            msg: e.to_string(),
        })?;

        self.network = get_network_name(&api_path).map_err(|e| RpcError::GetFailure {
            data_type: "network".to_string(),
            target_type: "node".to_string(),
            target_value: http_endpoint.clone(),
            msg: e.to_string(),
        })?;

        // If the node is not a validator, the `info.uptime` method will return an error
        // This should not get in the way of the node's information update
        let uptime = get_node_uptime(&api_path);
        match uptime {
            Ok(uptime) => self.uptime = uptime,
            Err(e) => match e {
                RpcError::ResponseError {
                    code,
                    message,
                    data,
                } => {
                    if code == -32000 && message.contains("node is not a validator") {
                        self.uptime = AvalancheNodeUptime::default();
                    } else {
                        return Err(AshError::RpcError(RpcError::GetFailure {
                            data_type: "uptime".to_string(),
                            target_type: "node".to_string(),
                            target_value: http_endpoint,
                            msg: format!(
                                "{:?}",
                                RpcError::ResponseError {
                                    code,
                                    message,
                                    data,
                                }
                            ),
                        }));
                    }
                }
                _ => {
                    return Err(AshError::RpcError(RpcError::GetFailure {
                        data_type: "uptime".to_string(),
                        target_type: "node".to_string(),
                        target_value: http_endpoint,
                        msg: e.to_string(),
                    }));
                }
            },
        }

        Ok(())
    }

    /// Check whether a given chain is done bootstrapping
    pub fn check_chain_bootstrapping(&self, chain: &str) -> Result<bool, AshError> {
        let http_endpoint = self.get_http_endpoint();
        let api_path = format!("{}/{}", http_endpoint, AVAX_INFO_API_ENDPOINT);

        let is_bootstrapped =
            is_bootstrapped(&api_path, chain).map_err(|e| RpcError::GetFailure {
                data_type: format!("{} chain bootstrapping", chain),
                target_type: "node".to_string(),
                target_value: http_endpoint.clone(),
                msg: e.to_string(),
            })?;

        Ok(is_bootstrapped)
    }
}

/// Avalanche node version
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheNodeVersions {
    pub avalanchego_version: String,
    pub database_version: String,
    pub git_commit: String,
    pub vm_versions: VmVersions,
    pub rpc_protocol_version: String,
}

impl From<GetNodeVersionResult> for AvalancheNodeVersions {
    fn from(node_version: GetNodeVersionResult) -> Self {
        Self {
            avalanchego_version: node_version.version,
            database_version: node_version.database_version,
            git_commit: node_version.git_commit,
            vm_versions: node_version.vm_versions,
            rpc_protocol_version: node_version.rpc_protocol_version,
        }
    }
}

/// Avalanche node uptime
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheNodeUptime {
    pub rewarding_stake_percentage: f64,
    pub weighted_average_percentage: f64,
}

impl From<UptimeResult> for AvalancheNodeUptime {
    fn from(node_uptime: UptimeResult) -> Self {
        Self {
            rewarding_stake_percentage: node_uptime.rewarding_stake_percentage,
            weighted_average_percentage: node_uptime.weighted_average_percentage,
        }
    }
}

/// Compute the node ID from the DER-encoded certificate bytes
pub fn node_id_from_cert_der(cert_bytes: &[u8]) -> Result<NodeId, AshError> {
    let node_id = NodeId::from_cert_der_bytes(cert_bytes)
        .map_err(|e| AvalancheNodeError::InvalidCertificate(e.to_string()))?;

    Ok(node_id)
}

/// Compute the node ID from the PEM-encoded X509 certificate string
pub fn node_id_from_cert_pem(cert_str: &str) -> Result<NodeId, AshError> {
    let cert_der = certs(&mut cert_str.as_bytes())
        .map_err(|e| AvalancheNodeError::InvalidCertificate(e.to_string()))?
        .remove(0);

    let node_id = node_id_from_cert_der(&cert_der)?;

    Ok(node_id)
}

/// Generate a new node ID with its TLS certificate and private key
pub fn generate_node_id(san: impl Into<Vec<String>>) -> Result<(NodeId, String, String), AshError> {
    let mut cert_params = CertificateParams::new(san);

    // Use RSA for Mac M* (ARM64) and ECDSA for everything else (AMD64)
    // See https://github.com/gyuho/cert-manager/blob/1b4211e1606ebfff6d958ba8a6a726fec03db232/src/x509.rs#L465
    if cfg!(target_arch = "aarch64") && cfg!(target_os = "macos") {
        cert_params.alg = &PKCS_RSA_SHA256
    }

    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CountryName, "Avalanche");
    distinguished_name.push(DnType::OrganizationName, "E36 Knots");
    distinguished_name.push(DnType::OrganizationalUnitName, "Ash");
    distinguished_name.push(DnType::CommonName, "Ash CLI self signed cert");
    cert_params.distinguished_name = distinguished_name;

    let cert = Certificate::from_params(cert_params).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();
    let key_pem = cert.serialize_private_key_pem();

    let node_id = node_id_from_cert_pem(&cert_pem)?;

    Ok((node_id, cert_pem, key_pem))
}

/// Generate a new node BLS private key with its proof of possession (public key + pop)
pub fn generate_node_bls_key() -> Result<(BlsPrivateKey, ProofOfPossession), AshError> {
    let key = BlsPrivateKey::generate().map_err(|e| {
        AvalancheNodeError::BlsError(format!("failed to generate private key: {}", e))
    })?;

    let pop = key.to_proof_of_possession();

    Ok((key, pop))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, str::FromStr};

    // Using avalanche-network-runner to run a test network
    const ASH_TEST_HTTP_PORT: u16 = 9650;
    const ASH_TEST_HTTP_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    const ASH_TEST_STACKING_PORT: u16 = 9651;
    const ASH_TEST_NODE_ID: &str = "NodeID-7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg";
    const ASH_TEST_NETWORK_NAME: &str = "network-1337";

    #[test]
    #[ignore]
    fn test_avalanche_node_update_info() {
        let mut node = AvalancheNode {
            http_host: ASH_TEST_HTTP_HOST.to_string(),
            http_port: ASH_TEST_HTTP_PORT,
            ..Default::default()
        };

        // Test that the node has the right http_host and http_port
        assert_eq!(node.http_host, ASH_TEST_HTTP_HOST.to_string());
        assert_eq!(node.http_port, ASH_TEST_HTTP_PORT);

        node.update_info().unwrap();

        // Test the node ID, network, public_ip and stacking_port
        assert_eq!(node.id, NodeId::from_str(ASH_TEST_NODE_ID).unwrap());
        assert_eq!(node.network, ASH_TEST_NETWORK_NAME);
        assert_eq!(node.public_ip, ASH_TEST_HTTP_HOST);
        assert_eq!(node.staking_port, ASH_TEST_STACKING_PORT);

        // Only test that the node version is not empty
        assert!(!node.versions.avalanchego_version.is_empty());
        assert!(!node.versions.database_version.is_empty());
        assert!(!node.versions.git_commit.is_empty());
        assert!(node.versions.vm_versions != VmVersions::default());
        assert!(!node.versions.rpc_protocol_version.is_empty());

        // Test that the node uptime is not equal to 0
        assert_ne!(node.uptime.rewarding_stake_percentage, 0.0);
        assert_ne!(node.uptime.weighted_average_percentage, 0.0);
    }

    #[test]
    #[ignore]
    fn test_avalanche_node_chain_bootstrapping() {
        let node = AvalancheNode {
            http_host: ASH_TEST_HTTP_HOST.to_string(),
            http_port: ASH_TEST_HTTP_PORT,
            ..Default::default()
        };

        // Get the node's bootstrapping status for the P, X and C chains
        let is_bootstrapped_p = node.check_chain_bootstrapping("P").unwrap();
        let is_bootstrapped_x = node.check_chain_bootstrapping("X").unwrap();
        let is_bootstrapped_c = node.check_chain_bootstrapping("C").unwrap();

        // Test that the node is bootstrapped for the P, X and C chains
        assert!(is_bootstrapped_p);
        assert!(is_bootstrapped_x);
        assert!(is_bootstrapped_c);
    }

    #[test]
    fn test_node_id_from_cert_der() {
        let cert_pem = fs::read_to_string("tests/certs/validator01.crt").unwrap();
        let cert_der = certs(&mut cert_pem.as_bytes()).unwrap().remove(0);

        let node_id = node_id_from_cert_der(&cert_der).unwrap();

        assert_eq!(
            node_id,
            NodeId::from_str("NodeID-7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg").unwrap()
        );
    }

    #[test]
    fn test_node_id_from_cert_pem() {
        let cert_pem = fs::read_to_string("tests/certs/validator01.crt").unwrap();

        let node_id = node_id_from_cert_pem(&cert_pem).unwrap();

        assert_eq!(
            node_id,
            NodeId::from_str("NodeID-7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg").unwrap()
        );
    }
}
