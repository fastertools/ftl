// Comprehensive debug test that mimics what policy tests do

use spin_test_sdk::{spin_test};

#[spin_test]
fn test_full_policy_flow_debug() {
    println!("Starting comprehensive policy flow test");
    
    // This mimics exactly what the policy tests do
    use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};
    use rsa::{RsaPrivateKey, RsaPublicKey};
    use rsa::pkcs1::{EncodeRsaPrivateKey, LineEnding as Pkcs1LineEnding};
    use rsa::pkcs8::{EncodePublicKey, LineEnding as Pkcs8LineEnding};
    
    println!("1. Generating test keypair");
    let mut rng = ChaCha8Rng::from_seed([42; 32]);
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits)
        .expect("failed to generate private key");
    let public_key = RsaPublicKey::from(&private_key);
    
    println!("2. Encoding public key to PEM");
    let public_key_pem = public_key
        .to_public_key_pem(Pkcs8LineEnding::LF)
        .expect("failed to encode public key");
    
    println!("3. Setting up JWT validation");
    use spin_test_sdk::bindings::fermyon::spin_test_virt::variables;
    variables::set("mcp_provider_type", "jwt");
    variables::set("mcp_jwt_public_key", &public_key_pem);
    variables::set("mcp_jwt_issuer", "https://test.example.com");
    variables::set("mcp_jwt_audience", "test-audience");
    
    println!("4. Creating JWT token");
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
    }
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    let claims = Claims {
        sub: "test-user".to_string(),
        iss: "https://test.example.com".to_string(),
        aud: "test-audience".to_string(),
        exp: now + 3600,
        iat: now,
    };
    
    println!("5. Encoding private key to PEM");
    let private_key_pem = private_key
        .to_pkcs1_pem(Pkcs1LineEnding::LF)
        .expect("failed to encode private key")
        .to_string();
    
    println!("6. Creating encoding key");
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .expect("failed to create encoding key");
    
    println!("7. Encoding JWT");
    let header = Header::new(Algorithm::RS256);
    let token = encode(&header, &claims, &encoding_key)
        .expect("failed to encode token");
    
    println!("8. Token created successfully: {} bytes", token.len());
    
    // If we get here, everything works
    assert!(token.len() > 0);
}