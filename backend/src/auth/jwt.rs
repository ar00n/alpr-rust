use ed25519_dalek::{
    pkcs8::{spki::der::pem::LineEnding, EncodePrivateKey, EncodePublicKey},
    SigningKey,
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header};
use rand::rand_core::UnwrapErr;
use rand::rngs::SysRng;
use std::{fs, path::Path};

use crate::state::JWT;

pub fn generate_ed25519_keys(priv_path: &Path, pub_path: &Path) {
    let mut csprng = UnwrapErr(SysRng);
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let priv_pem = signing_key
        .to_pkcs8_pem(LineEnding::LF)
        .expect("Failed to make key.");
    let pub_pem = verifying_key
        .to_public_key_pem(LineEnding::LF)
        .expect("Failed to make key.");

    fs::write(priv_path, priv_pem.as_bytes()).expect("Failed to write private key file");
    fs::write(pub_path, pub_pem.as_bytes()).expect("Failed to write public key file");
}

pub fn setup_jwt() -> JWT {
    let priv_path: String =
        std::env::var("PRIV_KEY_PATH").unwrap_or_else(|_| "ed_private.pem".to_string());
    let pub_path: String =
        std::env::var("PUB_KEY_PATH").unwrap_or_else(|_| "ed_public.pem".to_string());

    let priv_path = Path::new(&priv_path);
    let pub_path = Path::new(&pub_path);

    if let Some(parent) = priv_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .expect("Failed to create parent directory for private key");
        }
    }

    if let Some(parent) = pub_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .expect("Failed to create parent directory for public key");
        }
    }

    if !priv_path.exists() || !pub_path.exists() {
        tracing::info!("JWT key pair not found. Generating new Ed25519 key pair...");
        generate_ed25519_keys(priv_path, pub_path);
    }

    let priv_pem = fs::read(priv_path).expect("Failed to read private key");
    let pub_pem = fs::read(pub_path).expect("Failed to read public key");

    JWT {
        encoding_key: EncodingKey::from_ed_pem(&priv_pem).expect("Failed to parse private key"),
        decoding_key: DecodingKey::from_ed_pem(&pub_pem).expect("Failed to parse public key"),
        header: Header::new(Algorithm::EdDSA),
    }
}
