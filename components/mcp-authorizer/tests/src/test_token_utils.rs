//! Test utilities for generating JWT tokens
//!
//! This module provides utilities for generating test JWT tokens,
//! making it easier to write tests without complex JWT setup.

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rsa::pkcs1::LineEnding as Pkcs1LineEnding;
use rsa::pkcs8::LineEnding as Pkcs8LineEnding;
use rsa::{pkcs1::EncodeRsaPrivateKey, pkcs8::EncodePublicKey, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test key pair for JWT signing
pub struct TestKeyPair {
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
}

impl TestKeyPair {
    /// Generate a new RSA key pair for testing
    pub fn generate() -> Self {
        // Use deterministic RNG for WASM compatibility
        use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};
        let mut rng = ChaCha8Rng::from_seed([42; 32]);
        let bits = 2048;
        let private_key =
            RsaPrivateKey::new(&mut rng, bits).expect("failed to generate private key");
        let public_key = RsaPublicKey::from(&private_key);

        Self {
            private_key,
            public_key,
        }
    }

    /// Get the public key in PEM format
    pub fn public_key_pem(&self) -> String {
        self.public_key
            .to_public_key_pem(Pkcs8LineEnding::LF)
            .expect("failed to encode public key")
    }

    /// Get the private key in PEM format
    pub fn private_key_pem(&self) -> String {
        self.private_key
            .to_pkcs1_pem(Pkcs1LineEnding::LF)
            .expect("failed to encode private key")
            .to_string()
    }

    /// Create a JWT token with the given claims
    pub fn create_token(&self, builder: TestTokenBuilder) -> String {
        let kid = builder.kid.clone();
        let claims = builder.build();

        let header = Header {
            alg: Algorithm::RS256,
            kid,
            ..Default::default()
        };

        let encoding_key = EncodingKey::from_rsa_pem(self.private_key_pem().as_bytes())
            .expect("failed to create encoding key");

        jsonwebtoken::encode(&header, &claims, &encoding_key).expect("failed to encode token")
    }
}

/// Builder for test JWT tokens
#[derive(Default)]
pub struct TestTokenBuilder {
    subject: Option<String>,
    issuer: Option<String>,
    audience: Option<serde_json::Value>,
    scopes: Option<Vec<String>>,
    client_id: Option<String>,
    expires_in: Option<Duration>,
    kid: Option<String>,
    additional_claims: HashMap<String, serde_json::Value>,
}

impl TestTokenBuilder {
    /// Create a new token builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the subject (sub claim)
    pub fn subject(mut self, sub: impl Into<String>) -> Self {
        self.subject = Some(sub.into());
        self
    }

    /// Set the issuer (iss claim)
    pub fn issuer(mut self, iss: impl Into<String>) -> Self {
        self.issuer = Some(iss.into());
        self
    }

    /// Set a single audience (aud claim)
    pub fn audience(mut self, aud: impl Into<String>) -> Self {
        self.audience = Some(serde_json::Value::String(aud.into()));
        self
    }

    /// Set multiple audiences (aud claim as array)
    pub fn audiences(mut self, audiences: Vec<String>) -> Self {
        self.audience = Some(serde_json::Value::Array(
            audiences
                .into_iter()
                .map(serde_json::Value::String)
                .collect(),
        ));
        self
    }

    /// Set scopes (scope claim as space-separated string)
    pub fn scopes(mut self, scopes: Vec<&str>) -> Self {
        self.scopes = Some(scopes.into_iter().map(String::from).collect());
        self
    }

    /// Set the client ID
    pub fn client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// Set token expiration time (from now)
    pub fn expires_in(mut self, duration: Duration) -> Self {
        self.expires_in = Some(duration);
        self
    }

    /// Set the key ID (kid header)
    pub fn kid(mut self, kid: impl Into<String>) -> Self {
        self.kid = Some(kid.into());
        self
    }

    /// Add a custom claim
    pub fn claim(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.additional_claims.insert(key.into(), value);
        self
    }

    /// Add Microsoft-style scp claim (as array)
    pub fn scp_array(mut self, scopes: Vec<&str>) -> Self {
        self.additional_claims.insert(
            "scp".to_string(),
            serde_json::Value::Array(
                scopes
                    .into_iter()
                    .map(|s| serde_json::Value::String(s.to_string()))
                    .collect(),
            ),
        );
        self
    }

    /// Add Microsoft-style scp claim (as string)
    pub fn scp_string(mut self, scopes: &str) -> Self {
        self.additional_claims.insert(
            "scp".to_string(),
            serde_json::Value::String(scopes.to_string()),
        );
        self
    }

    /// Build the claims
    fn build(mut self) -> Claims {
        let now = Utc::now();
        let exp = match self.expires_in {
            Some(duration) => (now + duration).timestamp(),
            None => (now + Duration::hours(1)).timestamp(), // Default 1 hour
        };

        // Extract org_id from additional claims
        let org_id = self
            .additional_claims
            .remove("org_id")
            .and_then(|v| v.as_str().map(String::from));

        let claims = Claims {
            sub: self.subject.unwrap_or_else(|| "test-user".to_string()),
            iss: self
                .issuer
                .unwrap_or_else(|| "https://test.example.com".to_string()),
            aud: self.audience,
            exp,
            iat: now.timestamp(),
            nbf: None,
            client_id: self.client_id,
            scope: self.scopes.as_ref().map(|s| s.join(" ")),
            org_id,
        };

        // Merge remaining additional claims
        let mut claims_value = serde_json::to_value(&claims).unwrap();
        if let serde_json::Value::Object(ref mut map) = claims_value {
            for (key, value) in self.additional_claims {
                map.insert(key, value);
            }
        }

        serde_json::from_value(claims_value).unwrap()
    }
}

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iss: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    aud: Option<serde_json::Value>,
    exp: i64,
    iat: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    nbf: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    org_id: Option<String>,
}

/// Create a simple test token with minimal configuration
pub fn create_test_token(key_pair: &TestKeyPair, scopes: Vec<&str>) -> String {
    key_pair.create_token(TestTokenBuilder::new().scopes(scopes))
}

/// Create an expired test token
pub fn create_expired_token(key_pair: &TestKeyPair) -> String {
    key_pair.create_token(
        TestTokenBuilder::new().expires_in(Duration::seconds(-3600)), // Expired 1 hour ago
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_pair_generation() {
        let key_pair = TestKeyPair::generate();
        assert!(!key_pair.public_key_pem().is_empty());
        assert!(!key_pair.private_key_pem().is_empty());
    }

    #[test]
    fn test_token_creation() {
        let key_pair = TestKeyPair::generate();
        let token = create_test_token(&key_pair, vec!["read", "write"]);

        // JWT has three parts separated by dots
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_token_builder() {
        let key_pair = TestKeyPair::generate();

        let token = key_pair.create_token(
            TestTokenBuilder::new()
                .subject("custom-user")
                .issuer("https://custom.issuer.com")
                .audience("https://api.example.com")
                .scopes(vec!["admin", "write"])
                .client_id("test-app")
                .expires_in(Duration::hours(2))
                .claim("custom_field", serde_json::json!("custom_value")),
        );

        assert!(!token.is_empty());
    }
}
