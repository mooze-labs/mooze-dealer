use crate::models::pix;
use anyhow::bail;
use reqwest;
use serde_json::json;
use uuid::Uuid;

pub struct EulenApi {
    auth_token: String,
    url: String,
    client: reqwest::Client,
}

impl EulenApi {
    pub fn new(auth_token: String, url: String) -> Self {
        Self {
            auth_token,
            url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn deposit(
        &self,
        amount_in_cents: i32,
        address: &String,
    ) -> Result<pix::EulenDeposit, anyhow::Error> {
        let uuid = Uuid::new_v4().hyphenated().to_string();
        let payload = json!({
            "amountInCents": amount_in_cents,
            "depixAddress": address
        });

        let payload_json = self
            .client
            .post(format!("{}/api/deposit", self.url))
            .bearer_auth(&self.auth_token)
            .header("X-Nonce", uuid)
            .json(&payload);

        let response = payload_json.send().await?.text().await?;

        let response_json: serde_json::Value = serde_json::from_str(&response)?;
        match response_json.get("response") {
            Some(r) => {
                let deposit: pix::EulenDeposit = serde_json::from_value(r.clone())?;
                Ok(deposit)
            }
            None => {
                bail!("Eulen: Bad response format.")
            }
        }
    }
}
