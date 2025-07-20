//! Cabinet storage library for FoundationDB.
//!
//! This crate provides a high-level interface for storing and retrieving data in FoundationDB
//! with support for tenant isolation and transaction management.

use crate::cabinet::Cabinet;
use foundationdb::options::TransactionOption;
use foundationdb::{Database, FdbBindingError, RetryableTransaction};

pub mod cabinet;
pub mod errors;
pub mod item;
mod prefix;
mod stats;

/// Executes a transaction with automatic idempotency handling.
///
/// # Parameters
/// * `database` - The FoundationDB database instance
/// * `f` - Function to execute within the transaction that returns a Future
///
/// # Returns
/// Result containing the output of the transaction function or a FdbBindingError
pub async fn with_transaction<'a, F, Fut, T>(
    database: &Database,
    f: F,
) -> Result<T, FdbBindingError>
where
    F: FnOnce(RetryableTransaction) -> Fut + Clone,
    Fut: Future<Output = Result<T, FdbBindingError>>,
{
    database
        .run(|trx, _| {
            let f = f.clone();

            async move {
                trx.set_option(TransactionOption::AutomaticIdempotency)?;
                f(trx).await
            }
        })
        .await
}

/// Executes a transaction with a Cabinet instance for a specific tenant.
///
/// # Parameters
/// * `database` - The FoundationDB database instance
/// * `tenant` - Tenant identifier for isolation
/// * `f` - Function to execute with the Cabinet that returns a Future
///
/// # Returns  
/// Result containing the output of the cabinet function or an FdbBindingError
pub async fn with_cabinet<'a, F, Fut, T>(
    database: &Database,
    tenant: &str,
    f: F,
) -> Result<T, FdbBindingError>
where
    F: FnOnce(Cabinet) -> Fut + Clone,
    Fut: Future<Output = Result<T, FdbBindingError>>,
{
    with_transaction(database, |trx| {
        let f = f.clone();

        async move {
            let cabinet = Cabinet::new(trx, tenant);
            f(cabinet).await
        }
    })
    .await
}
