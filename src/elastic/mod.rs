use reqwest::Client;
use serde::Serialize;
use lazy_static::lazy_static;
use elasticsearch::{auth::Credentials, Elasticsearch, http::transport::Transport};
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;
use serde_json::json;

lazy_static! {
    static ref ELASTICSEARCH_CONNECTION: Mutex<Elasticsearch> = {
        let api_key_base64 = std::env::var("ELASTIC_API_KEY").expect("missing ELASTIC_API_KEY");
        let elastic_url = std::env::var("ELASTIC_URL").expect("missing ELASTIC_URL");
        let credentials = Credentials::ApiKey(api_key_base64.to_string());
        let transport = Transport::builder(elastic_url.parse()?)
            .auth(credentials)
            .build()?;

        // Build Elasticsearch client
        let elasticsearch_client = Elasticsearch::new(transport);
        Mutex::new(elasticsearch_client)
    };
}

pub async fn post_to_elastic<T: Serialize>(index: &str, doc: &T) -> Result<(), reqwest::Error> {
    let client = ELASTICSEARCH_CONNECTION.lock().await;

    let response = client
        .index(IndexParts::Index(index))
        .body(json!(doc))
        .send()
        .await?;

    let successful = response.status_code().is_success();

    // let url = format!("{elastic_url}/{index}/_doc");
    // let client = Client::new();
    // let res = client.post(&url).json(doc).send().await?;

    println!("{:?}", successful);

    Ok(())
}
