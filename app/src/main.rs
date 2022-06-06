use app::config::Config;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("app=debug")
        .init();
    info!("Loading config file...");
    let config = Config::from_file("./config/restaurant.toml");
    let my_app = app::App::new(config, "./events.db".into());
    my_app.serve().await;
}
