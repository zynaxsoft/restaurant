use app::{config::Config, App};
use client::RestaurantClient;

use anyhow::Result;

fn setup_service() -> App {
    let config = Config::from_toml_string(
        r###"
[restaurant]
table = 100
menus = ["a", "b", "c", "d", "e", "f"]

[network]
ip = "0.0.0.0"
port = 3001
"###,
    );
    App::new(config, "./event_test.db".into())
}

async fn gen_client(table: u32) -> Result<()> {
    // To make it more testable the app should have a tx/rx for telling the state
    // of the application. Then we can wait until the server is in the ready state
    // and then we can make the request.
    // But in this case just sleeping is enough.
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    let client = RestaurantClient::new("http://localhost:3001/order".into());
    for _ in 0..5 {
        client
            .request(format!(
                "new order for table {}: a * 10, b * 10, c * 5, d * 4, e * 3, f * 2",
                table
            ))
            .await?;
        client
            .request(format!(
                "cancel for table {}: a * 10, b * 10, c * 5, d * 4, e * 3, f * 2",
                table
            ))
            .await?;
    }
    let res = client.request(format!("check for table {}", table)).await?;
    assert_eq!(res, format!("Table {}:\nNo order yet.", table));
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_queries() -> Result<()> {
    let result = inner().await;
    std::fs::remove_file("./event_test.db").ok();
    result
}

async fn inner() -> Result<()> {
    let service = setup_service();
    let j_service = tokio::spawn(async move { service.serve().await });
    let mut handles = Vec::new();
    for i in 0..100 {
        handles.push(tokio::spawn(gen_client(i)));
    }
    let j_clients = async {
        for h in handles {
            h.await??;
        }
        Ok::<(), anyhow::Error>(())
    };
    tokio::select! {
        _ = j_service => (),
        _ = j_clients => (),
    }
    Ok(())
}
