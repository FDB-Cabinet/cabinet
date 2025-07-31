//! Item module provides key-value pair data structure and serialization utilities for cabinet storage.

use bincode::{decode_from_slice, encode_to_vec};
use std::fmt::{Debug, Formatter};
use toolbox::backend::errors::BackendError;
use toolbox::backend::record::Record;

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
}

impl Record for Item {
    /// Serializes this item into bytes.
    ///
    /// # Returns
    /// Serialized bytes of this item
    fn as_bytes(&self) -> Result<Vec<u8>, BackendError> {
        let config = bincode::config::standard();
        let encoded = encode_to_vec(self, config)
            .map_err(|err| BackendError::SerialiazationError(err.to_string()))?;
        Ok(encoded)
    }

    /// Creates an Item from serialized bytes.
    ///
    /// # Parameters
    /// * `bytes` - Serialized bytes of an Item
    ///
    /// # Returns
    /// Deserialized Item
    fn from_bytes(bytes: &[u8]) -> Result<Item, BackendError> {
        let config = bincode::config::standard();
        let (item, _) = decode_from_slice(bytes, config)
            .map_err(|err| BackendError::DeserializationError(err.to_string()))?;
        Ok(item)
    }

    /// Gets the key of this item.
    ///
    /// # Returns
    /// A reference to the key bytes
    fn get_key(&self) -> &[u8] {
        &self.key
    }
}
