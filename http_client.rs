use reqwest::Client;

pub struct HttpClient {
    client: Client,

}
impl HttpClient {
        pub fn new() -> Self {
            HttpClient {
                client: Client::new(),
            }
        }

        pub fn client(&self) -> &Client {
            &self.client

        }

}
