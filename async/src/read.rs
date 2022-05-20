use async_trait::async_trait;
use path_slash::PathBufExt;
use reqwest::Client;
use serde_json::Value;
use stac::{Error, Href, HrefObject, Object, Result};
use std::path::{Path, PathBuf};
use url::Url;

/// Reads STAC objects asynchronously.
#[async_trait]
pub trait AsyncRead {
    /// Reads a [HrefObject] asynchronously.
    ///
    /// # Examples
    ///
    /// [AsyncReader] implements [AsyncRead]:
    ///
    /// ```
    /// use stac_async::{AsyncRead, AsyncReader};
    /// let reader = AsyncReader::new();
    /// # tokio_test::block_on(async {
    /// let catalog = reader.read("data/catalog.json").await.unwrap();
    /// # })
    /// ```
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

    /// Reads json data asynchonously.
    async fn read_json<H>(&self, href: H) -> Result<Value>
    where
        H: Into<Href> + Send + Sync,
    {
        match href.into() {
            Href::Path(path) => self.read_json_from_path(PathBuf::from_slash(path)).await,
            Href::Url(url) => self.read_json_from_url(url).await,
        }
    }

    /// Reads json data from a local filesystem path.
    async fn read_json_from_path<P>(&self, path: P) -> Result<Value>
    where
        P: AsRef<Path> + Send + Sync;

    /// Reads json data from a remote url.
    async fn read_json_from_url(&self, url: Url) -> Result<Value>;
}

/// A simple structure for reading STAC data asynchronously.
///
/// Includes a [Client] which is used to pool requests.
#[derive(Debug, Default)]
pub struct AsyncReader {
    client: Client,
}

impl AsyncReader {
    /// Cretaes a new AsyncReader.
    pub fn new() -> AsyncReader {
        AsyncReader::default()
    }

    /// Returns a (cheap) clone of this reader's [Client].
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
