// Debug test to isolate RSA generation issue

use spin_test_sdk::{spin_test};

#[spin_test]
fn test_rsa_generation_debug() {
    println!("Starting RSA generation test");
    
    use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};
    use rsa::{RsaPrivateKey, RsaPublicKey};
    
    println!("Creating RNG with seed");
    let mut rng = ChaCha8Rng::from_seed([42; 32]);
    
    println!("Attempting to generate 2048-bit RSA key");
    let private_key = RsaPrivateKey::new(&mut rng, 2048)
        .expect("failed to generate private key");
    
    println!("Generated private key successfully");
    let public_key = RsaPublicKey::from(&private_key);
    
    println!("Generated public key successfully");
    
    // If we get here, generation works
    assert!(true);
}

#[spin_test]
fn test_small_rsa_generation() {
    use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};
    use rsa::{RsaPrivateKey, RsaPublicKey};
    
    let mut rng = ChaCha8Rng::from_seed([42; 32]);
    
    // Try progressively larger keys
    for bits in [64, 128, 256, 512, 1024, 2048] {
        println!("Generating {}-bit key", bits);
        let result = RsaPrivateKey::new(&mut rng, bits);
        match result {
            Ok(_) => println!("  Success!"),
            Err(e) => println!("  Failed: {:?}", e),
        }
    }
}