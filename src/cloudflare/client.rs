use log::{debug, warn};
use reqwest::{Request, StatusCode};

use super::models::*;
use std::time::Duration;

pub struct CloudFlareClient {
    client: reqwest::Client,
    token: String,
    zone_id: String,
    base_url: String,
}

impl CloudFlareClient {
    pub fn new(token: &str, zone_id: &str) -> Self {
        let client = reqwest::Client::builder().build().unwrap();
        Self {
            client,
            token: String::from(token),
            zone_id: String::from(zone_id),
            base_url: String::from("https://api.cloudflare.com"),
        }
    }

    async fn send_request(&self, request: Request) -> Result<reqwest::Response, reqwest::Error> {
        let mut attempts = 0;

        loop {
            let request = request.try_clone().expect("Failed to clone request");

            debug!("{} {}", request.method().to_string(), request.url());
            match self.client.execute(request).await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    let delay = Duration::from_millis(match attempts {
                        0 => 100,
                        1 => 200,
                        2 => 400,
                        3 => 800,
                        4 => 1200,
                        _ => return Err(e),
                    });

                    warn!("Request failed, retrying in {:?}", delay);
                    tokio::time::sleep(delay).await;

                    attempts += 1;
                    continue;
                }
            }
        }
    }

    async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, url);

        let request = self
            .client
            .get(url)
            .bearer_auth(&self.token)
            .build()
            .unwrap();

        self.send_request(request).await
    }

    async fn patch_body(
        &self,
        url: &str,
        body: impl serde::Serialize,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, url);

        let request = self
            .client
            .patch(url)
            .json(&body)
            .bearer_auth(&self.token)
            .build()
            .unwrap();

        self.send_request(request).await
    }

    /// api doc: https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-list-dns-records
    pub async fn get_dns_records(
        &self,
    ) -> Result<SuccessResponseList<DNSRecord>, CloudFlareClientError> {
        //TODO: implement paging ?page=1

        let url = format!(
            "/client/v4/zones/{}/dns_records?per_page=5000000",
            self.zone_id
        );

        let res = match self.get(&url).await {
            Ok(res) => res,
            Err(e) => return Err(CloudFlareClientError::Request(e)),
        };

        match res.status() {
            StatusCode::OK => Ok(res.json::<SuccessResponseList<DNSRecord>>().await.unwrap()),
            _ => Err(CloudFlareClientError::Api(
                res.json::<ErrorResponse>().await.unwrap(),
            )),
        }
    }

    /// api doc: https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-list-dns-records
    pub async fn get_dns_records_with_content(
        &self,
        content: &str,
    ) -> Result<SuccessResponseList<DNSRecord>, CloudFlareClientError> {
        //TODO: implement paging ?page=1

        let url = format!(
            "/client/v4/zones/{}/dns_records?per_page=5000000&content={}",
            self.zone_id, content
        );

        let res = match self.get(&url).await {
            Ok(res) => res,
            Err(e) => return Err(CloudFlareClientError::Request(e)),
        };

        match res.status() {
            StatusCode::OK => Ok(res.json::<SuccessResponseList<DNSRecord>>().await.unwrap()),
            _ => Err(CloudFlareClientError::Api(
                res.json::<ErrorResponse>().await.unwrap(),
            )),
        }
    }

    /// api doc: https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-patch-dns-record
    pub async fn set_dns_record(
        &self,
        request: UpdateDNSRecordRequest,
    ) -> Result<(), CloudFlareClientError> {
        let url = format!(
            "/client/v4/zones/{}/dns_records/{}",
            self.zone_id, request.id
        );

        let res = match self.patch_body(&url, request).await {
            Ok(res) => res,
            Err(e) => return Err(CloudFlareClientError::Request(e)),
        };

        match res.status() {
            StatusCode::OK => Ok(()),
            _ => Err(CloudFlareClientError::Api(
                res.json::<ErrorResponse>().await.unwrap(),
            )),
        }
    }

    /// api doc: https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-patch-dns-record
    pub async fn set_dns_record_content(
        &self,
        id: &str,
        content: &str,
    ) -> Result<(), CloudFlareClientError> {
        let url = format!("/client/v4/zones/{}/dns_records/{}", self.zone_id, id);

        #[derive(serde::Serialize)]
        struct Body {
            content: String,
        }

        let res = match self
            .patch_body(
                &url,
                Body {
                    content: String::from(content),
                },
            )
            .await
        {
            Ok(res) => res,
            Err(e) => return Err(CloudFlareClientError::Request(e)),
        };

        match res.status() {
            StatusCode::OK => Ok(()),
            _ => Err(CloudFlareClientError::Api(
                res.json::<ErrorResponse>().await.unwrap(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {}
