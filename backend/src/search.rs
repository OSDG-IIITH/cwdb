use meilisearch_sdk::client::Client;

use crate::config::Config;

pub fn init_client(config: &Config) -> Client {
    match &config.meili_key {
        Some(key) => Client::new(&config.meili_host, Some(key.as_str())).expect("Failed to create Meilisearch client"),
        None => Client::new(&config.meili_host, None::<&str>).expect("Failed to create Meilisearch client"),
    }
}
