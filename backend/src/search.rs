use meilisearch_sdk::client::Client;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::config::Config;

pub fn init_client(config: &Config) -> Client {
    match &config.meili_key {
        Some(key) => Client::new(&config.meili_host, Some(key.as_str()))
            .expect("Failed to create Meilisearch client"),
        None => Client::new(&config.meili_host, None::<&str>)
            .expect("Failed to create Meilisearch client"),
    }
}

pub async fn init_indexes(client: &Client, db: &PgPool) {
    let index_name = "resources";

    match client.get_index(index_name).await {
        Ok(_) => {
            tracing::info!("Index '{}' already exists", index_name);
        }
        Err(_) => {
            tracing::info!("Creating index '{}'...", index_name);
            match client.create_index(index_name, Some("id")).await {
                Ok(task) => {
                    tracing::info!(
                        "Index '{}' creation task submitted (uid: {})",
                        index_name,
                        task.task_uid
                    );
                    let _ = client.wait_for_task(task, None, None).await;
                }
                Err(e) => tracing::error!("Failed to create index '{}': {}", index_name, e),
            }
        }
    }

    apply_settings(client, db).await;
}

async fn apply_settings(client: &Client, db: &PgPool) {
    let index = client.index("resources");

    let searchable_attributes = ["title", "file_path", "type", "owner", "repo"];
    let filterable_attributes = ["type", "owner", "repo", "branch", "source_id"];
    let sortable_attributes = ["like_count", "created_at"];

    let synonyms: HashMap<String, Vec<String>> =
        match sqlx::query!("SELECT value FROM settings WHERE key = 'synonyms'")
            .fetch_optional(db)
            .await
        {
            Ok(Some(row)) => serde_json::from_value(row.value).unwrap_or_default(),
            Ok(None) => HashMap::new(),
            Err(e) => {
                tracing::error!("Failed to fetch synonyms: {}", e);
                HashMap::new()
            }
        };

    let settings = meilisearch_sdk::settings::Settings::new()
        .with_searchable_attributes(&searchable_attributes)
        .with_filterable_attributes(&filterable_attributes)
        .with_sortable_attributes(&sortable_attributes)
        .with_synonyms(synonyms);

    match index.set_settings(&settings).await {
        Ok(_) => tracing::info!("Meilisearch settings applied successfully"),
        Err(e) => tracing::error!("Failed to apply Meilisearch settings: {}", e),
    }
}
