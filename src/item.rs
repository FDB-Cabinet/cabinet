//! Item module provides key-value pair data structure and serialization utilities for cabinet storage.

use bincode::{decode_from_slice, encode_to_vec};
use std::fmt::{Debug, Formatter};

/// Represents a key-value pair item that can be stored in the cabinet.
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
    /// Creates a new Item with the given key and value.
    ///
    /// # Parameters
    /// * `key` - Byte slice containing the key
    /// * `value` - Byte slice containing the value
    ///
    /// # Returns
    /// A new Item instance
    pub fn new(key: &[u8], value: &[u8]) -> Item {
        Item {
            key: key.to_vec(),
            value: value.to_vec(),
        }
    }

    /// Gets the key of this item.
    ///
    /// # Returns
    /// A reference to the key bytes
    pub fn get_key(&self) -> &[u8] {
        &self.key
    }

    /// Serializes this item into bytes.
    ///
    /// # Returns
    /// Serialized bytes of this item
    pub fn as_bytes(&self) -> Vec<u8> {
        let config = bincode::config::standard();
        encode_to_vec(self, config).expect("Failed to encode item")
    }

    /// Creates an Item from serialized bytes.
    ///
    /// # Parameters
    /// * `bytes` - Serialized bytes of an Item
    ///
    /// # Returns
    /// Deserialized Item
    pub fn from_bytes(bytes: &[u8]) -> Item {
        let config = bincode::config::standard();
        let (item, _) = decode_from_slice(bytes, config).expect("Failed to decode item");
        item
    }
}
