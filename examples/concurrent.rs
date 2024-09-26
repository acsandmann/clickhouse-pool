use clickhouse_pool::ConnectionPool;
use futures::future::join_all;
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

    let task_count = 10;

    let mut tasks = Vec::new();

    for i in 0..task_count {
        let pool = pool.clone();
        tasks.push(tokio::spawn(async move {
            match pool.acquire().await {
                Ok(client_wrapper) => {
                    let client = client_wrapper.client();

                    let result: u64 = client
                        .query("SELECT number FROM system.numbers LIMIT 1 OFFSET ?")
                        .bind(i)
                        .fetch_one()
                        .await
                        .expect("Failed to fetch number");

                    println!("Task {}: Received number {}", i, result);
                }
                Err(e) => {
                    eprintln!("Task {}: Failed to acquire a client: {}", i, e);
                }
            }
        }));
    }

    join_all(tasks).await;
}
