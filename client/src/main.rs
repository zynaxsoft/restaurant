use std::process::exit;

use client::RestaurantClient;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!(
            "Please provide 2 arguments.
The first one for the entry point URL such as http://localhost:3000/order.
The second one is the TORO string
Example: new order for table 1: yakisoba * 1");
        exit(1);
    }
    let entry_point = &args[1];
    let payload = &args[2];
    let client = RestaurantClient::new(entry_point.into());
    let response = client.request(payload.clone()).await.unwrap();
    println!("Got a response:\n{}", response);
}
