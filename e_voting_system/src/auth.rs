// ============================================================
// File: auth.rs
// Purpose: Handles authentication and user verification logic.
//
// Responsibilities:
// - Hash and verify passwords (e.g., using bcrypt)
// - Manage user login and session control
// - Differentiate between admin, district, and voter roles
// - Ensure secure access to system functionality
// ============================================================

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use rand::rngs::OsRng;

pub fn hash_password(pw: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    // The new version of Argon2 doesn't have Params::recommended(),
    // so we use Params::new(...) manually.
    let params = Params::new(15000, 2, 1, None).unwrap(); // memory cost, iterations, parallelism
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    argon.hash_password(pw.as_bytes(), &salt).unwrap().to_string()
}

pub fn verify_password(hash: &str, pw: &str) -> bool {
    if let Ok(parsed) = PasswordHash::new(hash) {
        Argon2::default().verify_password(pw.as_bytes(), &parsed).is_ok()
    } else {
        false
    }
}
