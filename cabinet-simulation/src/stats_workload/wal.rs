//! This module implements a Write-Ahead Log (WAL) simulation for testing cabinet operations
//! with different event types and probabilities.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use cabinet::item::Item;
use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use rand::{Rng, RngCore};
use rand_chacha::ChaCha20Rng;
use std::collections::HashMap;
use std::fmt::Debug;
use toolbox::backend::record::Record;
use toolbox::backend::tenant::Tenant;

/// Number of different event types supported
const EVENT_TYPE_CARDINALITY: u32 = 3;

/// Maximum length for generated keys
const MAX_KEY_LENGTH: u32 = 20;
/// Maximum length for generated values
const MAX_VALUE_LENGTH: u32 = 20;

/// Minimum length for generated keys
const MIN_KEY_LENGTH: u32 = 4;
/// Minimum length for generated values
const MIN_VALUE_LENGTH: u32 = 0;

/// Represents events that can be written to the WAL
#[derive(Clone)]
pub enum WalEvent {
    /// Put a key-value pair
    Put {
        /// Key to store
        key: Vec<u8>,
        /// Value to store
        value: Vec<u8>,
    },
    /// Delete a key
    Delete {
        /// Key to delete
        key: Vec<u8>,
    },
    /// Clear all data
    Clear,
}

/// Result of applying a WAL event
#[derive(Debug)]
pub enum ApplyResult {
    /// Result of Put operation with the stored Item
    Put(Item),
    /// Result of Delete operation with optional removed Item
    Delete(Option<Item>),
    /// Result of Clear operation
    Clear,
}

impl WalEvent {
    /// Applies the event to the given cabinet
    ///
    /// # Parameters
    /// * `cabinet` - Cabinet instance to apply the event to
    ///
    /// # Returns
    /// Result containing the ApplyResult or an error
    pub async fn apply(&self, cabinet: Tenant) -> cabinet::errors::Result<ApplyResult> {
        match self {
            WalEvent::Put { key, value } => {
                let item = Item::new(&key, &value);
                cabinet.put(&item).await?;
                Ok(ApplyResult::Put(item))
            }
            WalEvent::Delete { key } => {
                if let Some(item) = cabinet.delete(&key).await? {
                    return Ok(ApplyResult::Delete(Some(item)));
                }
                Ok(ApplyResult::Delete(None))
            }
            WalEvent::Clear => {
                cabinet.clear::<Item>().await?;
                Ok(ApplyResult::Clear)
            }
        }
    }
}

impl ApplyResult {
    /// Updates statistics after applying an event
    ///
    /// # Parameters
    /// * `stats` - Statistics holder to update
    pub fn update_stats(&self, stats: &mut StatsHolder) {
        match self {
            ApplyResult::Put(item) => stats.put(item),
            ApplyResult::Delete(item) => {
                if let Some(item) = item {
                    stats.delete(item);
                }
            }
            ApplyResult::Clear => stats.clear(),
        }
    }
}

impl Debug for WalEvent {
    /// Formats the WalEvent for debug output
    ///
    /// # Parameters
    /// * `f` - Formatter to write to
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalEvent::Put { key, value } => {
                write!(
                    f,
                    "Put {{ key: {:?}, value: {:?} }}",
                    STANDARD.encode(key),
                    STANDARD.encode(value)
                )
            }
            WalEvent::Delete { key } => write!(f, "Delete {{ key: {:?} }}", STANDARD.encode(key)),
            WalEvent::Clear => write!(f, "Clear"),
        }
    }
}

/// Types of events that can occur in the WAL
#[derive(Copy, Clone, Debug)]
enum EventType {
    Put,
    Delete,
    Clear,
}

/// Array of possible event types
const EVENTS: [EventType; EVENT_TYPE_CARDINALITY as usize] =
    [EventType::Put, EventType::Delete, EventType::Clear];

/// Probability distribution for different event types
const EVENT_PROBABILITIES: [f32; EVENT_TYPE_CARDINALITY as usize] = [0.89, 0.1, 0.01];

/// Probability of deleting an existing key vs generating a random key
const DELETION_PROBABILITY: f64 = 0.55;

/// Holds statistics about the current state
#[derive(Debug, Default)]
pub struct StatsHolder {
    count: u64,
    size: u64,
}

impl StatsHolder {
    /// Updates stats after putting an item
    ///
    /// # Parameters
    /// * `item` - Item that was put
    pub fn put(&mut self, item: &Item) {
        self.count += 1;
        self.size += item.as_bytes().expect("Unable to get item bytes").len() as u64;
    }

    /// Updates stats after deleting an item
    ///
    /// # Parameters
    /// * `item` - Item that was deleted
    pub fn delete(&mut self, item: &Item) {
        self.count -= 1;
        self.size -= item.as_bytes().expect("Unable to get item bytes").len() as u64;
    }

    /// Clears all stats
    pub fn clear(&mut self) {
        self.count = 0;
        self.size = 0;
    }

    /// Gets the current count
    pub fn get_count(&self) -> u64 {
        self.count
    }

    /// Gets the current total size
    pub fn get_size(&self) -> u64 {
        self.size
    }
}

/// Write-Ahead Log implementation for simulating workloads
pub struct Wal {
    keys: HashMap<String, Vec<Vec<u8>>>,
    rng: ChaCha20Rng,
    weighted_events: WeightedIndex<f32>,
}

/// Generates a random key using the given RNG
///
/// # Parameters
/// * `rng` - RNG to use for key generation
fn random_key(rng: &mut ChaCha20Rng) -> Vec<u8> {
    let key_length = rng.random_range(MIN_KEY_LENGTH..MAX_KEY_LENGTH);
    let mut key = vec![0; key_length as usize];
    rng.fill_bytes(&mut key);
    key
}

impl Wal {
    /// Creates a new WAL with the given RNG
    ///
    /// # Parameters
    /// * `rng` - RNG to use for generating events
    pub fn new(rng: ChaCha20Rng) -> Self {
        let weighted_events =
            WeightedIndex::new(&EVENT_PROBABILITIES).expect("Failed to create weighted index");

        Self {
            keys: Default::default(),
            rng,
            weighted_events,
        }
    }

    /// Generates the next event for a tenant
    ///
    /// # Parameters
    /// * `tenant` - Tenant to generate event for
    pub fn next_event(&mut self, tenant: &str) -> WalEvent {
        let index = self.weighted_events.sample(&mut self.rng);
        let event_type = EVENTS[index];

        match event_type {
            EventType::Put => {
                let key_length = self.rng.random_range(MIN_KEY_LENGTH..MAX_KEY_LENGTH);
                let mut key = vec![0; key_length as usize];
                self.rng.fill_bytes(&mut key);

                let value_length = self.rng.random_range(MIN_VALUE_LENGTH..MAX_VALUE_LENGTH);
                let mut data = vec![0; value_length as usize];
                self.rng.fill_bytes(&mut data);

                let event = WalEvent::Put {
                    key: key.clone(),
                    value: data,
                };
                self.keys.entry(tenant.to_string()).or_default().push(key);
                event
            }
            EventType::Delete => {
                if self.rng.random_bool(DELETION_PROBABILITY) {
                    let Some(tenant_keys) = self.keys.get_mut(tenant) else {
                        return self.push_random_delete();
                    };

                    if tenant_keys.is_empty() {
                        return self.push_random_delete();
                    }

                    let index = self.rng.random_range(0..tenant_keys.len());
                    let key = tenant_keys[index].clone();

                    let event = WalEvent::Delete { key };
                    tenant_keys.remove(index);

                    return event;
                }

                self.push_random_delete()
            }
            EventType::Clear => {
                let event = WalEvent::Clear;
                self.keys.remove(tenant);
                event
            }
        }
    }

    /// Generates a random delete event
    fn push_random_delete(&mut self) -> WalEvent {
        let key = random_key(&mut self.rng);
        let event = WalEvent::Delete { key };
        event
    }
}
