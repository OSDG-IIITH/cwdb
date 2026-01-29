use meilisearch_sdk::client::Client;

use crate::config::Config;

pub fn init_client(config: &Config) -> Client {
    match &config.meili_key {
        Some(key) => Client::new(&config.meili_host, Some(key.as_str())).expect("Failed to create Meilisearch client"),
        None => Client::new(&config.meili_host, None::<&str>).expect("Failed to create Meilisearch client"),
    }
}

pub async fn init_indexes(client: &Client) {
    // Create the resources index if it doesn't exist
    let index_name = "resources";
    
    match client.get_index(index_name).await {
        Ok(_) => {
            tracing::info!("Index '{}' already exists", index_name);
        }
        Err(_) => {
            tracing::info!("Creating index '{}'...", index_name);
            match client.create_index(index_name, Some("id")).await {
                Ok(_) => tracing::info!("Index '{}' created successfully", index_name),
                Err(e) => tracing::error!("Failed to create index '{}': {}", index_name, e),
            }
        }
    }
}
