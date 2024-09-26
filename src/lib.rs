use clickhouse::Client;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::{Arc, Mutex};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
mod error;
use crate::error::Error;

pub struct ConnectionPool {
    clients: Arc<Mutex<Vec<Client>>>,
    semaphore: Arc<Semaphore>,
}

impl ConnectionPool {
    /// Spawns a new connection pool, given the address to the ClickHouse server,
    /// and the maximum number of connections that the pool can spawn.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the connections failed to instantiate.
    pub async fn spawn(params: impl Into<String>, count: usize) -> Result<Self, Error> {
        let params = params.into();
        let mut clients = Vec::with_capacity(count);

        for _ in 0..count {
            let client = connect(params.clone()).await?;
            clients.push(client);
        }

        Ok(ConnectionPool {
            clients: Arc::new(Mutex::new(clients)),
            semaphore: Arc::new(Semaphore::new(count)),
        })
    }

    /// Acquires a `Client` from the pool.
    ///
    /// Returns a `ClientWrapper` which will automatically return the client
    /// to the pool when dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if the semaphore is closed or no clients are available.
    pub async fn acquire(&self) -> Result<ClientWrapper, Error> {
        let permit = self.semaphore.clone().acquire_owned().await?;

        let client = {
            let mut clients = self.clients.lock().unwrap();
            clients.pop()
        };

        if let Some(client) = client {
            Ok(ClientWrapper {
                client: Some(client),
                pool: self.clone(),
                _permit: permit,
            })
        } else {
            // This should not happen because the semaphore ensures that clients are available
            drop(permit);
            Err(Error::Unknown)
        }
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        ConnectionPool {
            clients: Arc::clone(&self.clients),
            semaphore: Arc::clone(&self.semaphore),
        }
    }
}

/// A wrapper around `Client` that returns it to the pool when dropped.
pub struct ClientWrapper {
    client: Option<Client>,
    pool: ConnectionPool,
    _permit: OwnedSemaphorePermit,
}

impl ClientWrapper {
    /// Accesses the `Client`.
    pub fn client(&self) -> &Client {
        self.client.as_ref().unwrap()
    }

    /// Mutably accesses the `Client`.
    pub fn client_mut(&mut self) -> &mut Client {
        self.client.as_mut().unwrap()
    }
}

impl Drop for ClientWrapper {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            let mut clients = self.pool.clients.lock().unwrap();
            clients.push(client);
        }
    }
}

impl Debug for ConnectionPool {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("ConnectionPool { ... }")
    }
}

async fn connect(params: impl Into<String>) -> Result<Client, Error> {
    let client = Client::default().with_url(params);

    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clickhouse::test;
    use futures::future::join_all;
    use tokio;

    use once_cell::sync::Lazy;

    static MOCK: Lazy<test::Mock> = Lazy::new(|| test::Mock::new());

    #[tokio::test]
    async fn test_pool_limits() {
        let pool_size = 2;

        let pool = ConnectionPool::spawn(MOCK.url(), pool_size)
            .await
            .expect("Failed to spawn pool");

        let client1 = pool.acquire().await.expect("Failed to acquire client 1");
        let client2 = pool.acquire().await.expect("Failed to acquire client 2");

        let pool_clone = pool.clone();
        let acquire_future = tokio::spawn(async move {
            pool_clone
                .acquire()
                .await
                .expect("Failed to acquire client 3")
        });

        drop(client1);

        let client3 = acquire_future.await.expect("Failed to await client 3");

        drop(client2);
        drop(client3);
    }
    
    #[tokio::test]
    async fn test_concurrent_acquisitions() {
        let pool_size = 5;
        let task_count = 10;

        let pool = ConnectionPool::spawn(MOCK.url(), pool_size)
            .await
            .expect("Failed to spawn pool");

        let mut tasks = Vec::new();

        for i in 0..task_count {
            let pool = pool.clone();
            tasks.push(tokio::spawn(async move {
                let client_wrapper = pool.acquire().await.expect("Failed to acquire client");
                let client = client_wrapper.client();

                let result: u64 = client
                    .query("SELECT number FROM system.numbers LIMIT 1 OFFSET ?")
                    .bind(i)
                    .fetch_one()
                    .await
                    .expect("Failed to fetch number");

                assert_eq!(result, i as u64);
            }));
        }

        join_all(tasks).await;
    }
}
