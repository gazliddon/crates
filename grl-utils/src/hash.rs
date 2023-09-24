
pub fn get_hash(text : &[u8]) -> String {
    let mut dest= [0u8; 128];

    use sha1::{Sha1, Digest};

    let mut hasher = Sha1::new();
    hasher.update(text);
    let ret = hasher.finalize();

    let ret = base16ct::lower::encode_str(&ret, &mut dest).unwrap();
    ret.to_string()
}

pub fn get_hash_from_str(text : &str) -> String {
    get_hash(text.as_bytes())
}
