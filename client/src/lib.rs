use anyhow::Result;
use hyper::{client::HttpConnector, Client, Request, Method};

pub struct RestaurantClient {
    client: Client<HttpConnector>,
    entry_point: String,
}

impl RestaurantClient {
    pub fn new(entry_point: String) -> Self {
        Self {
            client: Client::new(),
            entry_point,
        }
    }

    pub async fn request<T>(&self, order: T) -> Result<String>
    where
        hyper::Body: From<T>,
    {
        let req = Request::builder()
            .method(Method::POST)
            .uri(&self.entry_point)
            .header("Authorization", "pl3a53-h1r3-m3")
            .body(order.into())?;
        let res = self.client.request(req).await?;
        let full_body = hyper::body::to_bytes(res.into_body()).await?;
        Ok(String::from_utf8(full_body.into_iter().collect())?)
    }
}
