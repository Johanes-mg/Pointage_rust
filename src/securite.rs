use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;

const ITERATIONS: u32 = 100_000;

pub fn generer_sel() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    general_purpose::STANDARD.encode(bytes)
}

pub fn hacher_mot_de_passe(mot_de_passe: &str, sel: &str) -> String {
    let sel_bytes = general_purpose::STANDARD
        .decode(sel)
        .unwrap_or_else(|_| sel.as_bytes().to_vec());

    let mut mac = Hmac::<Sha256>::new_from_slice(mot_de_passe.as_bytes())
        .expect("HMAC accepte toute taille de cle");
    mac.update(&sel_bytes);
    let mut resultat = mac.finalize().into_bytes().to_vec();

    for _ in 1..ITERATIONS {
        let mut mac = Hmac::<Sha256>::new_from_slice(mot_de_passe.as_bytes())
            .expect("HMAC accepte toute taille de cle");
        mac.update(&resultat);
        resultat = mac.finalize().into_bytes().to_vec();
    }

    general_purpose::STANDARD.encode(resultat)
}

pub fn verifier_mot_de_passe(mot_de_passe: &str, sel: &str, hash_attendu: &str) -> bool {
    let calcule = hacher_mot_de_passe(mot_de_passe, sel);
    constant_eq(calcule.as_bytes(), hash_attendu.as_bytes())
}

fn constant_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
