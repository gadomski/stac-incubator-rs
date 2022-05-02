use async_trait::async_trait;
use path_slash::PathBufExt;
use reqwest::Client;
use serde_json::Value;
use stac::{Error, Href, HrefObject, Object, Result};
use std::path::{Path, PathBuf};
use url::Url;

#[async_trait]
pub trait AsyncRead {
    async fn read<H>(&self, href: H) -> Result<HrefObject>
    where
        H: Into<Href> + Send + Sync,
    {
        let href = href.into();
        let value = self.read_json(href.clone()).await?;
        let object = Object::from_value(value)?;
        Ok(HrefObject {
            href: href.into(),
            object,
        })
    }

    async fn read_json<H>(&self, href: H) -> Result<Value>
    where
        H: Into<Href> + Send + Sync,
    {
        match href.into() {
            Href::Path(path) => self.read_json_from_path(PathBuf::from_slash(path)).await,
            Href::Url(url) => self.read_json_from_url(url).await,
        }
    }

    async fn read_json_from_path<P>(&self, path: P) -> Result<Value>
    where
        P: AsRef<Path> + Send + Sync;
    async fn read_json_from_url(&self, url: Url) -> Result<Value>;
}

#[derive(Debug, Default)]
pub struct AsyncReader {
    client: Client,
}

impl AsyncReader {
    pub fn new() -> AsyncReader {
        AsyncReader::default()
    }

    pub fn client(&self) -> Client {
        self.client.clone()
    }
}

#[async_trait]
impl AsyncRead for AsyncReader {
    async fn read_json_from_path<P>(&self, path: P) -> Result<Value>
    where
        P: AsRef<Path> + Send + Sync,
    {
        let bytes = tokio::fs::read(path).await?;
        serde_json::from_slice(&bytes).map_err(Error::from)
    }

    async fn read_json_from_url(&self, url: Url) -> Result<Value> {
        let response = self.client.get(url).send().await?;
        Ok(response.json().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::{AsyncRead, AsyncReader};

    #[tokio::test]
    async fn async_read() {
        let async_reader = AsyncReader::new();
        async_reader.read("data/simple-item.json").await.unwrap();
    }
}
