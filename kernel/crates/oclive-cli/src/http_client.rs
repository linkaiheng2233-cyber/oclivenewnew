//! Blocking HTTP helpers for oclive-cli (reqwest-backed; replaces ureq for supply-chain dedup).

use std::io::{Cursor, Read};
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::Method;

#[derive(Debug)]
pub struct HttpError(pub String);

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for HttpError {}

pub struct Agent {
    client: Client,
}

impl Clone for Agent {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}

pub struct AgentBuilder {
    timeout: Option<Duration>,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self { timeout: None }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Agent {
        let mut builder = Client::builder();
        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }
        Agent {
            client: builder.build().unwrap_or_else(|_| Client::new()),
        }
    }
}

pub struct Request {
    client: Client,
    method: Method,
    url: String,
    headers: Vec<(String, String)>,
    timeout: Option<Duration>,
    body: Option<Vec<u8>>,
}

pub struct Response {
    status: u16,
    body: Vec<u8>,
}

impl Response {
    pub fn status(&self) -> u16 {
        self.status
    }

    pub fn into_string(self) -> Result<String, HttpError> {
        String::from_utf8(self.body).map_err(|e| HttpError(e.to_string()))
    }

    pub fn into_reader(self) -> impl Read {
        Cursor::new(self.body)
    }
}

impl Agent {
    pub fn get(&self, url: &str) -> Request {
        Request::new(self.client.clone(), Method::GET, url)
    }

    pub fn post(&self, url: &str) -> Request {
        Request::new(self.client.clone(), Method::POST, url)
    }
}

impl Request {
    fn new(client: Client, method: Method, url: &str) -> Self {
        Self {
            client,
            method,
            url: url.to_string(),
            headers: Vec::new(),
            timeout: None,
            body: None,
        }
    }

    pub fn set(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn call(self) -> Result<Response, HttpError> {
        self.send(None)
    }

    pub fn send_string(self, body: &str) -> Result<Response, HttpError> {
        self.send(Some(body.as_bytes().to_vec()))
    }

    pub fn send_bytes(self, body: &[u8]) -> Result<Response, HttpError> {
        self.send(Some(body.to_vec()))
    }

    fn send(mut self, body: Option<Vec<u8>>) -> Result<Response, HttpError> {
        if body.is_some() {
            self.body = body;
        }
        let mut builder = self.client.request(self.method, &self.url);
        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }
        for (name, value) in self.headers {
            builder = builder.header(name, value);
        }
        if let Some(body) = self.body {
            builder = builder.body(body);
        }
        let resp = builder.send().map_err(|e| HttpError(e.to_string()))?;
        let status = resp.status().as_u16();
        let body = resp.bytes().map_err(|e| HttpError(e.to_string()))?.to_vec();
        Ok(Response { status, body })
    }
}

fn default_agent() -> Agent {
    AgentBuilder::new().build()
}

pub fn get(url: &str) -> Request {
    default_agent().get(url)
}

pub fn post(url: &str) -> Request {
    default_agent().post(url)
}
