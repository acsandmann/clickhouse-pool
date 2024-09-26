use clickhouse_pool::ConnectionPool;
use tokio;

#[tokio::main]
async fn main() {
    // Replace with your ClickHouse server URL
    let clickhouse_url = "http://localhost:8123";

    let pool = match ConnectionPool::spawn(clickhouse_url, 5).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to create connection pool: {}", e);
            return;
        }
    };

    match pool.acquire().await {
        Ok(client_wrapper) => {
            let client = client_wrapper.client();

            let version: String = client
                .query("SELECT version()")
                .fetch_one()
                .await
                .expect("Failed to fetch version");

            println!("ClickHouse server version: {}", version);
        }
        Err(e) => {
            eprintln!("Failed to acquire a client: {}", e);
        }
    }
}
