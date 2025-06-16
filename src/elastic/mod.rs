use reqwest::Client;
use serde::Serialize;

async fn post_to_elastic<T: Serialize>(index: &str, doc: &T) {
    let elastic_url = std::env::var("ELASTIC_URL").expect("missing ELASTIC_URL");

    let url = format!("http://localhost:9200/{}/_doc", index);
    let client = Client::new();
    let res = client.post(&url).json(doc).send().await.unwrap();

    println!("{:?}", res.text().await.unwrap());
}
