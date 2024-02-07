// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with the Ash Console OAuth2 provider

use oauth2::{
    basic::BasicClient, devicecode::StandardDeviceAuthorizationResponse, reqwest::http_client,
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    DeviceAuthorizationUrl, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, Scope,
    TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::errors::*;

/// Ash Console OAuth2 client
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AshConsoleOAuth2Client {
    /// OAuth2 client
    #[serde(skip)]
    pub client: Option<BasicClient>,
    /// OAuth2 client ID
    #[serde(rename = "clientID")]
    pub client_id: ClientId,
    /// OAuth2 client secret
    pub client_secret: Option<ClientSecret>,
    /// OAuth2 authorization URL
    pub authorization_url: AuthUrl,
    /// OAuth2 token URL
    pub token_url: TokenUrl,
    /// OAuth2 redirect URL
    pub redirect_url: Option<RedirectUrl>,
    /// OAuth2 device authorization URL
    pub device_authorization_url: Option<DeviceAuthorizationUrl>,
}

impl Default for AshConsoleOAuth2Client {
    fn default() -> Self {
        Self {
            client: None,
            client_id: ClientId::new("cf83e1357eefb8bd".to_string()),
            client_secret: None,
            authorization_url: AuthUrl::new(
                "http://localhost:8090/realms/jeeo/protocol/openid-connect/auth".to_string(),
            )
            .unwrap(),
            token_url: TokenUrl::new(
                "http://localhost:8090/realms/jeeo/protocol/openid-connect/token".to_string(),
            )
            .unwrap(),
            redirect_url: Some(RedirectUrl::new("about:blank".to_string()).unwrap()),
            device_authorization_url: Some(
                DeviceAuthorizationUrl::new(
                    "http://localhost:8090/realms/jeeo/protocol/openid-connect/auth/device"
                        .to_string(),
                )
                .unwrap(),
            ),
        }
    }
}

impl AshConsoleOAuth2Client {
    /// Initialize the OAuth2 client
    pub fn init(&mut self) {
        // Initialize the OAuth2 client
        let mut client = BasicClient::new(
            self.client_id.clone(),
            self.client_secret.clone(),
            self.authorization_url.clone(),
            Some(self.token_url.clone()),
        );

        if let Some(redirect_uri) = &self.redirect_url {
            client = client.set_redirect_uri(redirect_uri.clone());
        }

        if let Some(device_authorization_url) = &self.device_authorization_url {
            client = client.set_device_authorization_url(device_authorization_url.clone());
        }

        self.client = Some(client);
    }

    /// Check if the client is initialized
    pub fn is_initialized(&self) -> Result<(), AshError> {
        if self.client.is_none() {
            return Err(ConsoleOAuth2Error::ClientNotInitialized.into());
        }

        Ok(())
    }

    /// Generate a full authorization URL with a PKCE challenge and scopes if specified
    pub fn generate_authorization_code_grant_url(
        &self,
        scopes: Option<&str>,
    ) -> Result<(Url, CsrfToken, PkceCodeVerifier), AshError> {
        // If redirect URL is not specified, return an error
        if self.redirect_url.is_none() {
            return Err(ConsoleOAuth2Error::UrlNotSpecified {
                url: "redirectUrl".to_string(),
            }
            .into());
        }

        self.is_initialized()?;

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = self
            .client
            .as_ref()
            .unwrap()
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes.map(|s| Scope::new(s.to_string())))
            .set_pkce_challenge(pkce_challenge)
            .url();

        Ok((auth_url, csrf_token, pkce_verifier))
    }

    /// Exchange an authorization code for an access token
    pub fn exchange_authorization_code(
        &self,
        authorization_code: &str,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<AccessToken, AshError> {
        self.is_initialized()?;

        let token = self
            .client
            .as_ref()
            .unwrap()
            .exchange_code(AuthorizationCode::new(authorization_code.to_string()))
            .set_pkce_verifier(pkce_verifier)
            .request(http_client)
            .map_err(|e| ConsoleOAuth2Error::TokenRequestFailure { msg: e.to_string() })?;

        Ok(token.access_token().clone())
    }

    /// Generate a device authorization response which contains the verification URL and user code
    pub fn generate_device_authorization_response(
        &self,
        scopes: Option<&str>,
    ) -> Result<StandardDeviceAuthorizationResponse, AshError> {
        // If device authorization URL is not specified, return an error
        if self.device_authorization_url.is_none() {
            return Err(ConsoleOAuth2Error::UrlNotSpecified {
                url: "deviceAuthorizationUrl".to_string(),
            }
            .into());
        }

        self.is_initialized()?;

        let details = self
            .client
            .as_ref()
            .unwrap()
            .exchange_device_code()
            .map_err(|e| ConsoleOAuth2Error::TokenRequestFailure { msg: e.to_string() })?
            .add_scopes(scopes.map(|s| Scope::new(s.to_string())))
            .request(http_client)
            .map_err(|e| ConsoleOAuth2Error::TokenRequestFailure { msg: e.to_string() })?;

        Ok(details)
    }

    /// Exchange a device code for an access token
    pub fn exchange_device_code(
        &self,
        device_auth_response: &StandardDeviceAuthorizationResponse,
    ) -> Result<(AccessToken, RefreshToken), AshError> {
        self.is_initialized()?;

        let token = self
            .client
            .as_ref()
            .unwrap()
            .exchange_device_access_token(device_auth_response)
            .request(http_client, std::thread::sleep, None)
            .map_err(|e| ConsoleOAuth2Error::TokenRequestFailure { msg: e.to_string() })?;

        Ok((
            token.access_token().clone(),
            // Assume that token refresh is allowed
            token.refresh_token().unwrap().clone(),
        ))
    }

    /// Refresh an access token
    pub fn refresh_access_token(&self, refresh_token_str: &str) -> Result<AccessToken, AshError> {
        self.is_initialized()?;

        let refresh_token = RefreshToken::new(refresh_token_str.to_string());

        let token = self
            .client
            .as_ref()
            .unwrap()
            .exchange_refresh_token(&refresh_token)
            .request(http_client)
            .map_err(|e| ConsoleOAuth2Error::TokenRequestFailure { msg: e.to_string() })?;

        Ok(token.access_token().clone())
    }
}
