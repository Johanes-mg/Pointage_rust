use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;

const ITERATIONS_PBKDF2: u32 = 600_000;
const ITERATIONS_LEGACY: u32 = 100_000;
const PREFIX_PBKDF2: &str = "pbkdf2-sha256$";

type HmacSha256 = Hmac<Sha256>;

pub fn generer_sel() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    general_purpose::STANDARD.encode(bytes)
}

fn hmac_sha256(cle: &[u8], message: &[u8]) -> [u8; 32] {
    // HMAC accepte toutes les tailles de clé ; l'échec est impossible ici.
    let mut mac = HmacSha256::new_from_slice(cle)
        .expect("HMAC-SHA256 accepte toute taille de clé");
    mac.update(message);
    let digest = mac.finalize().into_bytes();
    let mut resultat = [0u8; 32];
    resultat.copy_from_slice(&digest);
    resultat
}

/// Nouveau format standard PBKDF2-HMAC-SHA256, avec un bloc de sortie de 32 octets.
pub fn hacher_mot_de_passe(mot_de_passe: &str, sel: &str) -> String {
    let sel_bytes = general_purpose::STANDARD
        .decode(sel)
        .unwrap_or_else(|_| sel.as_bytes().to_vec());

    let mut entree = sel_bytes;
    entree.extend_from_slice(&1_u32.to_be_bytes());

    let mut u = hmac_sha256(mot_de_passe.as_bytes(), &entree);
    let mut resultat = u;

    for _ in 1..ITERATIONS_PBKDF2 {
        u = hmac_sha256(mot_de_passe.as_bytes(), &u);
        for (octet, nouveau) in resultat.iter_mut().zip(u.iter()) {
            *octet ^= *nouveau;
        }
    }

    format!(
        "{}{}",
        PREFIX_PBKDF2,
        general_purpose::STANDARD.encode(resultat)
    )
}

/// Vérifie les anciens comptes créés par la version précédente de l'application.
/// Cet ancien algorithme était itératif, mais ne correspondait pas à PBKDF2.
fn hacher_ancien(mot_de_passe: &str, sel: &str) -> String {
    let sel_bytes = general_purpose::STANDARD
        .decode(sel)
        .unwrap_or_else(|_| sel.as_bytes().to_vec());

    let mut resultat = hmac_sha256(mot_de_passe.as_bytes(), &sel_bytes);
    for _ in 1..ITERATIONS_LEGACY {
        resultat = hmac_sha256(mot_de_passe.as_bytes(), &resultat);
    }
    general_purpose::STANDARD.encode(resultat)
}

pub fn est_ancien_hash(hash: &str) -> bool {
    !hash.starts_with(PREFIX_PBKDF2)
}

pub fn verifier_mot_de_passe(mot_de_passe: &str, sel: &str, hash_attendu: &str) -> bool {
    if let Some(hash_encode) = hash_attendu.strip_prefix(PREFIX_PBKDF2) {
        let attendu = match general_purpose::STANDARD.decode(hash_encode) {
            Ok(valeur) => valeur,
            Err(_) => return false,
        };
        let calcule = hacher_mot_de_passe(mot_de_passe, sel);
        let calcule = match calcule.strip_prefix(PREFIX_PBKDF2) {
            Some(valeur) => match general_purpose::STANDARD.decode(valeur) {
                Ok(bytes) => bytes,
                Err(_) => return false,
            },
            None => return false,
        };
        constant_eq(&calcule, &attendu)
    } else {
        let calcule = hacher_ancien(mot_de_passe, sel);
        constant_eq(calcule.as_bytes(), hash_attendu.as_bytes())
    }
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
