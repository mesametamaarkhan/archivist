use chacha20poly1305::{
    AeadCore, XChaCha20Poly1305, XNonce, aead::{Aead, KeyInit, OsRng}
};

pub fn encrypt(key: &[u8; 32], plaintext: &[u8], ad: &[u8]) -> (Vec<u8>, [u8; 24]) {
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher 
        .encrypt(XNonce::from_slice(&nonce), chacha20poly1305::aead::Payload {
            msg: plaintext,
            aad: ad,
        })
        .expect("encryption failure");

    (ciphertext, nonce.into())
}

pub fn decrypt(key: &[u8; 32], ciphertext: &[u8], nonce: &[u8; 24], ad: &[u8]) -> Result<Vec<u8>, chacha20poly1305::aead::Error> {
    let cipher = XChaCha20Poly1305::new(key.into());

    cipher.decrypt(
        XNonce::from_slice(nonce), 
        chacha20poly1305::aead::Payload {
            msg: ciphertext,
            aad: ad,
        },
    )
}