use bincode::{decode_from_slice, encode_to_vec};
use std::fmt::{Debug, Formatter};

#[derive(bincode::Encode, bincode::Decode)]
pub struct Item {
    key: Vec<u8>,
    pub value: Vec<u8>,
}

impl Debug for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "key: {} => value : {}",
            String::from_utf8_lossy(&self.key),
            String::from_utf8_lossy(&self.value)
        )
    }
}

impl Item {
    pub fn new(key: &[u8], value: &[u8]) -> Item {
        Item {
            key: key.to_vec(),
            value: value.to_vec(),
        }
    }

    pub fn get_key(&self) -> &[u8] {
        &self.key
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let config = bincode::config::standard();
        encode_to_vec(self, config).expect("Failed to encode item")
    }

    pub fn from_bytes(bytes: &[u8]) -> Item {
        let config = bincode::config::standard();
        let (item, _) = decode_from_slice(bytes, config).expect("Failed to decode item");
        item
    }
}
