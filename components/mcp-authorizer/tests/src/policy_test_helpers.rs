// Helper functions for policy authorization tests

use spin_test_sdk::bindings::fermyon::spin_test_virt::variables;
use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1::{EncodeRsaPrivateKey, LineEnding as Pkcs1LineEnding};
use rsa::pkcs8::{EncodePublicKey, LineEnding as Pkcs8LineEnding};

// Use a fixed symmetric key for HS256 in tests to avoid RSA generation issues
const TEST_HMAC_SECRET: &[u8] = b"test-secret-key-for-policy-tests-only-not-for-production-use";

/// Generate a test keypair - for policy tests, use smaller key to avoid ChaCha8Rng issues
pub fn generate_test_keypair() -> (RsaPrivateKey, RsaPublicKey) {
    println!("DEBUG generate_test_keypair: Starting RSA key generation");
    // Try using rand::thread_rng() first, fall back to smaller key if needed
    let mut rng = rand::thread_rng();
    println!("DEBUG generate_test_keypair: Got RNG");
    let bits = 2048; // Required for RS256 in jsonwebtoken
    println!("DEBUG generate_test_keypair: About to generate {} bit key", bits);
    let private_key = RsaPrivateKey::new(&mut rng, bits)
        .expect("failed to generate private key");
    println!("DEBUG generate_test_keypair: Private key generated");
    let public_key = RsaPublicKey::from(&private_key);
    println!("DEBUG generate_test_keypair: Public key derived");
    (private_key, public_key)
}

/// Set up JWT validation with a given keypair
pub fn setup_test_jwt_validation_with_keypair(public_key: &RsaPublicKey) {
    println!("DEBUG setup_test_jwt_validation_with_keypair: Starting");
    let public_key_pem = public_key
        .to_public_key_pem(Pkcs8LineEnding::LF)
        .expect("failed to encode public key");
    println!("DEBUG setup_test_jwt_validation_with_keypair: PEM encoded");
    
    variables::set("mcp_provider_type", "jwt");
    println!("DEBUG setup_test_jwt_validation_with_keypair: Set provider type");
    variables::set("mcp_jwt_public_key", &public_key_pem);
    println!("DEBUG setup_test_jwt_validation_with_keypair: Set public key");
    // Use a non-AuthKit issuer to avoid JWKS fetching
    variables::set("mcp_jwt_issuer", "https://test.example.com");
    println!("DEBUG setup_test_jwt_validation_with_keypair: Set issuer");
    variables::set("mcp_jwt_audience", "test-audience");
    println!("DEBUG setup_test_jwt_validation_with_keypair: Set audience");
}

/// Set up JWT validation with test key (generates a new keypair)
/// Returns the keypair for use in token creation
pub fn setup_test_jwt_validation() -> (RsaPrivateKey, RsaPublicKey) {
    let (private_key, public_key) = generate_test_keypair();
    setup_test_jwt_validation_with_keypair(&public_key);
    (private_key, public_key)
}

/// Sets up a basic allow-all policy for testing (does NOT set up JWT validation)
pub fn setup_allow_all_policy() {
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := true
"#;
    variables::set("mcp_policy", policy);
}

/// Sets up a basic deny-all policy for testing
pub fn setup_deny_all_policy() {
    let policy = r#"
package mcp.authorization
import rego.v1

default allow := false
"#;
    variables::set("mcp_policy", policy);
}

/// Sets up a policy that checks for specific subject
pub fn setup_subject_check_policy(allowed_subjects: Vec<&str>) {
    let subjects_list = allowed_subjects.iter()
        .map(|s| format!("\"{}\"", s))
        .collect::<Vec<_>>()
        .join(", ");
    
    let policy = format!(r#"
package mcp.authorization
import rego.v1

default allow := false

allow if {{
    input.token.sub in [{}]
}}
"#, subjects_list);
    
    variables::set("mcp_policy", &policy);
}

/// Sets up a policy that checks for specific roles in claims
pub fn setup_role_based_policy(required_role: &str) {
    let policy = format!(r#"
package mcp.authorization
import rego.v1

default allow := false

allow if {{
    "{}" in input.token.claims.roles
}}
"#, required_role);
    
    variables::set("mcp_policy", &policy);
}

/// Sets up a policy that checks component access
/// Returns the keypair for use in token creation
pub fn setup_component_policy(allowed_components: Vec<&str>) -> (RsaPrivateKey, RsaPublicKey) {
    let (private_key, public_key) = setup_test_jwt_validation();
    let components_list = allowed_components.iter()
        .map(|c| format!("\"{}\"", c))
        .collect::<Vec<_>>()
        .join(", ");
    
    let policy = format!(r#"
package mcp.authorization
import rego.v1

default allow := false

allow if {{
    input.request.component in [{}]
}}
"#, components_list);
    
    variables::set("mcp_policy", &policy);
    (private_key, public_key)
}

/// Sets up a policy for MCP tool authorization
/// Returns the keypair for use in token creation
pub fn setup_tool_authorization_policy(allowed_tools: Vec<&str>, dangerous_tools: Vec<&str>) -> (RsaPrivateKey, RsaPublicKey) {
    let (private_key, public_key) = setup_test_jwt_validation();
    let allowed_list = allowed_tools.iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(", ");
    
    let dangerous_list = dangerous_tools.iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(", ");
    
    let policy = format!(r#"
package mcp.authorization
import rego.v1

default allow := false

# Allow tool discovery
allow if {{
    input.mcp.method == "tools/list"
}}

# Allow safe tools
allow if {{
    input.mcp.method == "tools/call"
    input.mcp.tool in [{}]
}}

# Deny dangerous tools without admin role
deny if {{
    input.mcp.method == "tools/call"
    input.mcp.tool in [{}]
    not "admin" in input.token.claims.roles
}}

# Final allow if not denied
allow if {{
    input.mcp
    not deny
}}
"#, allowed_list, dangerous_list);
    
    variables::set("mcp_policy", &policy);
    (private_key, public_key)
}

/// Sets up a policy with external data
/// Returns the keypair for use in token creation
pub fn setup_policy_with_data(policy: &str, data: &str) -> (RsaPrivateKey, RsaPublicKey) {
    let (private_key, public_key) = setup_test_jwt_validation();
    variables::set("mcp_policy", policy);
    variables::set("mcp_policy_data", data);
    (private_key, public_key)
}

/// Creates a test token with specific claims for policy testing  
/// Uses the provided private key to sign the token
pub fn create_policy_test_token_with_key(
    private_key: &RsaPrivateKey,
    subject: &str,
    roles: Vec<&str>,
    additional_claims: Vec<(&str, serde_json::Value)>,
) -> String {
    let private_key_pem = private_key
        .to_pkcs1_pem(Pkcs1LineEnding::LF)
        .expect("failed to encode private key")
        .to_string();
    
    // Create token using jsonwebtoken directly
    use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
    use serde::{Serialize, Deserialize};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        iss: String,
        aud: String,
        exp: i64,
        iat: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        roles: Option<Vec<String>>,
        #[serde(flatten)]
        additional: std::collections::HashMap<String, serde_json::Value>,
    }
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    let mut additional = std::collections::HashMap::new();
    for (key, value) in additional_claims {
        additional.insert(key.to_string(), value);
    }
    
    let claims = Claims {
        sub: subject.to_string(),
        iss: "https://test.example.com".to_string(),
        aud: "test-audience".to_string(),
        exp: now + 3600,
        iat: now,
        roles: if roles.is_empty() { None } else { Some(roles.iter().map(|s| s.to_string()).collect()) },
        additional,
    };
    
    let header = Header::new(Algorithm::RS256);
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .expect("failed to create encoding key");
    
    encode(&header, &claims, &encoding_key).expect("failed to encode token")
}

/// Creates a test token with specific claims for policy testing
/// This generates a new keypair - tests must ensure the public key is configured separately  
pub fn create_policy_test_token(
    subject: &str,
    roles: Vec<&str>,
    additional_claims: Vec<(&str, serde_json::Value)>,
) -> String {
    let (private_key, _public_key) = generate_test_keypair();
    create_policy_test_token_with_key(&private_key, subject, roles, additional_claims)
}

/// Clear all policy configuration
pub fn clear_policy_config() {
    variables::set("mcp_policy", "");
    variables::set("mcp_policy_data", "");
}