use mongodb::{Client, Database};

use crate::config::Config;

pub async fn connect(config: &Config) -> Database {
    let client = Client::with_uri_str(&config.mongodb_uri)
        .await
        .expect("failed to connect to MongoDB");
    client.database(&config.database_name)
}
