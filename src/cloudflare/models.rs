#![allow(dead_code)]
use core::fmt;
use std::{net::Ipv4Addr, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum CloudFlareClientError {
    Request(reqwest::Error),
    Api(ErrorResponse),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ErrorResponse {
    pub errors: Vec<Message>,
    pub messages: Vec<Message>,
    pub success: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct SuccessResponseList<T> {
    pub errors: Vec<Message>,
    pub messages: Vec<Message>,
    pub success: bool,
    pub result_info: ResultInfo,
    pub result: Vec<T>,
}

impl<T> SuccessResponseList<T> {
    pub fn count(&self) -> usize {
        self.result.len()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct UpdateDNSRecordRequest {
    pub content: String,
    pub name: String,
    pub proxied: Option<bool>,
    pub r#type: DNSType,
    pub comment: Option<String>,
    pub id: String,
    pub tags: Option<Vec<String>>,
    pub ttl: Option<i32>,
}

impl From<DNSRecord> for UpdateDNSRecordRequest {
    fn from(value: DNSRecord) -> Self {
        Self {
            content: value.content,
            name: value.name,
            proxied: value.proxied,
            r#type: value.r#type,
            comment: value.comment,
            id: value.id,
            tags: value.tags,
            ttl: value.ttl,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Message {
    pub code: i32,
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct ResultInfo {
    pub count: i32,
    pub page: i32,
    pub per_page: i32,
    pub total_count: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct DNSRecord {
    pub content: String,
    pub name: String,
    pub proxied: Option<bool>,
    pub r#type: DNSType,
    pub comment: Option<String>,
    pub comment_modified_on: Option<DateTime<Utc>>,
    pub created_on: DateTime<Utc>,
    pub id: String,
    pub meta: Option<DNSRecordMeta>,
    pub modified_on: DateTime<Utc>,
    pub proxiable: bool,
    pub tags: Option<Vec<String>>,
    pub tags_modified_on: Option<DateTime<Utc>>,
    pub ttl: Option<i32>,
}

impl DNSRecord {
    pub fn has_tags(&self) -> bool {
        match &self.tags {
            Some(tags) => !tags.is_empty(),
            None => false,
        }
    }

    pub fn content_as_ip(&self) -> Result<Ipv4Addr, std::net::AddrParseError> {
        Ipv4Addr::from_str(&self.content)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
#[repr(i32)]
pub enum DNSType {
    #[default]
    A = 1,
    AAAA = 28,
    CAA = 257,
    CERT = 37,
    CNAME = 5,
    DNSKEY = 48,
    DS = 43,
    HTTPS = 65,
    LOC = 29,
    MX = 15,
    NAPTR = 35,
    NS = 2,
    PTR = 12,
    SMIMEA = 53,
    SRV = 33,
    SSHFP = 44,
    SVCB = 64,
    TLSA = 52,
    TXT = 16,
    URI = 256,
}

impl fmt::Display for DNSType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl DNSType {
    /// https://en.wikipedia.org/wiki/List_of_DNS_record_types
    pub fn id(&self) -> i32 {
        self.clone() as i32
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct DNSRecordMeta {
    pub auto_added: bool,
    pub source: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_response_list_count() {
        let mut o = SuccessResponseList::<i32>::default();
        o.result = vec![1, 2, 3, 4, 5];
        assert_eq!(o.count(), 5);
    }

    #[test]
    fn dns_record_has_tags() {
        let mut o = DNSRecord::default();
        o.tags = Some(vec![String::from("a")]);
        assert_eq!(o.has_tags(), true);
    }

    #[test]
    fn dns_record_have_empty_tags() {
        let mut o = DNSRecord::default();
        o.tags = Some(vec![]);
        assert_eq!(o.has_tags(), false);
    }

    #[test]
    fn dns_record_have_none_tags() {
        let mut o = DNSRecord::default();
        o.tags = None;
        assert_eq!(o.has_tags(), false);
    }

    #[test]
    fn dns_record_content_as_ip_pass() {
        let mut o = DNSRecord::default();
        o.content = String::from("127.0.0.1");
        assert_eq!(
            o.content_as_ip().unwrap(),
            std::net::Ipv4Addr::new(127, 0, 0, 1)
        );
    }
}
