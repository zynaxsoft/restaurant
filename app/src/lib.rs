pub mod config;
pub mod db;
pub mod event;
pub mod projector;
pub mod restaurant;
pub mod service;
pub mod sql_source;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use tokio::sync::Mutex;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

use tracing::{error, info};

use crate::config::Config;
use crate::db::Db;
use crate::projector::RestaurantProjector;
use crate::service::restaurant_service;
use crate::sql_source::SqliteEventSource;

pub struct App {
    config: Arc<Config>,
    db_file: String,
}

impl App {
    pub fn new(config: Config, db_file: String) -> Self {
        Self {
            config: Arc::new(config),
            db_file,
        }
    }

    pub async fn serve(&self) {
        info!("Initializing the application...");
        info!("Setting up database connection...");
        let db = Arc::new(
            Db::init(&self.db_file).expect("Something went wrong when connecting the database"),
        );
        let event_source = SqliteEventSource::new(db.clone());
        let projector = RestaurantProjector::new(self.config.restaurant.n_table, event_source);
        let projector = Arc::new(Mutex::new(projector));
        info!("Catching up old events...");
        projector
            .lock()
            .await
            .update()
            .expect("There are some bad events in the database.");

        info!("Making a service...");
        let addr = SocketAddr::from_str(&format!(
            "{}:{}",
            self.config.network.ip, self.config.network.port
        ))
        .expect("Please use a correct ip and port");
        let make_svc = make_service_fn(move |_conn| {
            let config = self.config.clone();
            let db = db.clone();
            let projector = projector.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    restaurant_service(config.clone(), db.clone(), projector.clone(), req)
                }))
            }
        });
        let server = Server::bind(&addr).serve(make_svc);

        info!("Serving on {}", addr);
        if let Err(e) = server.await {
            error!("server error: {}", e);
        }
    }
}
