use crate::{
    config::Config,
    db::Db,
    projector::RestaurantProjector,
    restaurant::{Item, Table},
    sql_source::SqliteEventSource,
};
use anyhow::{anyhow, bail, Result};
use hyper::{Body, Request, Response};
use std::{
    fmt::Display,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;
use toro::{MenuName, Toro};
use tracing::{debug, info, instrument};

type Projector = Arc<Mutex<RestaurantProjector<SqliteEventSource>>>;

const SECRET: &str = "pl3a53-h1r3-m3";

fn my_response<T>(status: u16, body: T) -> Response<Body>
where
    Body: From<T>,
{
    Response::builder()
        .status(status)
        .body(body.into())
        .expect("This shouldn't fail.")
}

pub async fn restaurant_service(
    config: Arc<Config>,
    db: Arc<Db>,
    projector: Projector,
    req: Request<Body>,
) -> Result<Response<Body>> {
    if let Some(api_token) = req.headers().get("Authorization") {
        // A very safe and secure non-constant time comparison
        if api_token != SECRET {
            return Ok(my_response(401, "You shall not pass."));
        }
    } else {
        return Ok(my_response(401, "Did you forget our secret word?"));
    }
    if req.uri().path() != "/order" {
        return Ok(my_response(200, "Nothing to see here."));
    }
    let full_body = hyper::body::to_bytes(req.into_body()).await?;
    let payload_str = String::from_utf8(full_body.into_iter().collect())?;

    parse_order_string(config, db, projector, &payload_str).await
}

#[instrument(name = "Got an order string", skip(config, db, projector))]
async fn parse_order_string(
    config: Arc<Config>,
    db: Arc<Db>,
    projector: Projector,
    payload_str: &str,
) -> Result<Response<Body>> {
    match Toro::from_toro_string(payload_str) {
        Ok(toro) => process_order(config, db, projector, toro).await,
        Err(e) => {
            info!("It was a weird order string. Error: {}", e);
            return Ok(my_response(400, "Invalid order string"));
        }
    }
}

async fn process_order(
    config: Arc<Config>,
    db: Arc<Db>,
    projector: Projector,
    toro: Toro,
) -> Result<Response<Body>> {
    debug!("Successfully parsed the order string.");

    use toro::Command::*;
    let result = match toro.command {
        Check => check_table(config, db, projector, toro).await,
        _ => store_event(config, db, projector, toro).await,
    };
    match result {
        Ok(_) => result,
        Err(e) => {
            info!("Something went wrong with error. {}", e);
            Ok(my_response(400, "Probably bad request"))
        }
    }
}

struct TableQuery<'a> {
    table: &'a Table,
    query: Option<Vec<MenuName>>,
}
impl<'a> TableQuery<'a> {
    fn from_table(table: &'a Table, query: Option<Vec<MenuName>>) -> Self {
        Self { table, query }
    }
}
impl<'a> Display for TableQuery<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (table, query) = (self.table, &self.query);
        writeln!(f, "Table {}:", table.id)?;
        if table.items.is_empty() {
            write!(f, "No order yet.")?;
            return Ok(());
        }
        let list: Vec<Option<&Item>> = match query {
            // Sticking with Option wrapped. Just in case some menu in the query
            // doesn't exist, so we still can handle it.
            Some(menus) => menus.iter().map(|m| table.items.get(m)).collect(),
            None => table.items.values().map(|i| Some(i)).collect(),
        };
        for i in list.iter() {
            if let Some(item) = i {
                write!(f, "{} * {}", item.id, item.quantity)?;
                // Actually this should be fetched from the event `check` itself
                // So it will show the same result every time.
                // But with the current design, we don't store the event `check`,
                // so we will just go with this.
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("time went backward.")
                    .as_secs();
                let elapsed = now - item.timestamp;
                let cooking_time = item.cooking_time.ok_or(std::fmt::Error)?;
                let eta = cooking_time.saturating_sub(elapsed);
                if eta == 0 {
                    writeln!(f, " finished")?;
                } else {
                    let (eta_min, eta_sec) = (eta / 60, eta % 60);
                    writeln!(f, " in {} minutes {} seconds", eta_min, eta_sec)?;
                }
            }
        }
        Ok(())
    }
}

#[instrument(name = "Checking table", skip_all)]
async fn check_table(
    _config: Arc<Config>,
    _db: Arc<Db>,
    projector: Projector,
    toro: Toro,
) -> Result<Response<Body>> {
    let mut proj = projector.lock().await;
    // Just In Time™ update
    proj.update()?;
    let table = proj
        .get_table(toro.table_id.ok_or(anyhow!("Expecting table id"))?)
        .ok_or(anyhow!("Table not found."))?;
    let query = match toro.param {
        Some(p) => match p {
            toro::Param::Menu(v) => Some(v),
            _ => bail!("Only menu parameter is supported"),
        },
        None => None,
    };
    Ok(my_response(
        200,
        format!("{}", TableQuery::from_table(table, query)),
    ))
}

#[instrument(name = "Storing event", skip_all)]
async fn store_event(
    config: Arc<Config>,
    db: Arc<Db>,
    _projector: Projector,
    toro: Toro,
) -> Result<Response<Body>> {
    let max_table = config.restaurant.n_table;
    if let Some(table_id) = toro.table_id {
        if table_id >= max_table as usize {
            debug!(
                "Bad event: Too large table id {}. Maximum is {}",
                table_id,
                max_table - 1
            );
            let err_str = format!(
                "We don't have table {}. We only have upto table number {}.",
                table_id,
                max_table - 1
            );
            return Ok(my_response(400, err_str));
        }
    }
    if let Some(param) = &toro.param {
        let wrong_menus: Vec<&MenuName> = match param {
            toro::Param::MenuQuantities(v) => v
                .iter()
                .filter(|(m, _)| !config.restaurant.menus.contains(m))
                .map(|(m, _)| m)
                .collect(),
            toro::Param::Menu(v) => v
                .iter()
                .filter(|&m| !config.restaurant.menus.contains(m))
                .collect(),
        };
        if !wrong_menus.is_empty() {
            debug!(
                "Got menu name that is not supported in the config: {:?}",
                wrong_menus
            );
            return Ok(my_response(
                400,
                format!("We don't serve {:?}", wrong_menus),
            ));
        }
    }
    match db.insert_event(toro) {
        Ok(_) => {
            info!("The event looks nice. Putting it in the DB.");
            Ok(my_response(200, "Order received"))
        }
        Err(e) => {
            debug!("Something went wrong with error {}", e);
            Ok(my_response(500, "Something went wrong inside."))
        }
    }
}
