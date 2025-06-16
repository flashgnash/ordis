use reqwest::Client;
use serde::Serialize;

pub async fn post_to_elastic<T: Serialize>(index: &str, doc: &T) -> Result<(), reqwest::Error> {
    let elastic_url = std::env::var("ELASTIC_URL").expect("missing ELASTIC_URL");

    let url = format!("{elastic_url}/{index}/_doc");
    let client = Client::new();
    let res = client.post(&url).json(doc).send().await?;

    println!("{:?}", res.text().await?);

    Ok(())
}
