use sha2::{Digest, Sha256};

pub fn hash_token(raw: &str) -> String {
    let digest = Sha256::digest(raw.as_bytes());
    hex_encode(&digest)
}

pub fn generate_token(prefix: &str) -> String {
    let mut bytes = [0u8; 24];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
    format!("{prefix}{}", hex_encode(&bytes))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn bearer_from_header(header: Option<&str>) -> Option<&str> {
    header
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable() {
        assert_eq!(
            hash_token("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn bearer_parsing() {
        assert_eq!(
            bearer_from_header(Some("Bearer trm_abc")),
            Some("trm_abc")
        );
        assert_eq!(bearer_from_header(Some("Basic x")), None);
    }
}
