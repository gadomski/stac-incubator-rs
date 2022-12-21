use crate::message::Message;
use console::{style, Term};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use stac::{Asset, Item, Link, Links, Value};
use std::path::Path;
use thiserror::Error;
use tokio::{fs::File, io::AsyncWriteExt, task::JoinSet};
use url::Url;

#[derive(Debug, Error)]
enum Error {
    #[error("no file name: {0}")]
    NoFileName(Url),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}

pub async fn download(href: String, directory: impl AsRef<Path>) -> i32 {
    match super::read(&href).await {
        Ok(value) => {
            let mut item = if let Value::Item(item) = value {
                item
            } else {
                crate::print_err(&format!(
                    "STAC value is a {}, not an Item",
                    value.type_name()
                ));
                return 1;
            };
            if !directory.as_ref().exists() {
                let message = Message::new(format!(
                    "=> creating {}",
                    style(directory.as_ref().to_string_lossy()).bold()
                ));
                if let Err(err) = tokio::fs::create_dir_all(directory.as_ref()).await {
                    message.finish_red();
                    crate::print_err(&err.to_string());
                    return 1;
                }
                message.finish_blue();
            }

            let multi_progress = MultiProgress::new();
            let message = Message::new(format!(
                "=> downloading assets for {} to {}",
                style(&item.id),
                style(directory.as_ref().to_string_lossy()).bold()
            ));
            let client = Client::new();
            let mut set = JoinSet::new();
            for (key, asset) in item.assets.drain() {
                let directory = directory.as_ref().to_path_buf();
                let progress_bar = multi_progress.add(ProgressBar::new(0));
                let client = client.clone();
                set.spawn(async move {
                    match download_asset(
                        client,
                        key.clone(),
                        asset.clone(),
                        directory,
                        progress_bar,
                    )
                    .await
                    {
                        Ok((key, asset)) => Ok((key, asset)),
                        Err(err) => Err((key, asset, err)),
                    }
                });
            }
            let mut errors = Vec::new();
            let mut at_least_one_download_ok = false;
            while let Some(result) = set.join_next().await {
                match result {
                    Ok(result) => match result {
                        Ok((key, asset)) => {
                            at_least_one_download_ok = true;
                            item.assets.insert(key, asset);
                        }
                        Err((key, asset, err)) => {
                            errors.push((Some(key), Some(asset), err));
                        }
                    },
                    Err(err) => {
                        errors.push((None, None, Error::from(err)));
                    }
                }
            }
            let term = Term::stdout();
            let _ = term.clear_last_lines(1);
            if at_least_one_download_ok {
                if errors.is_empty() {
                    message.finish_blue();
                } else {
                    message.finish_yellow();
                }
            } else {
                message.finish_red();
            }
            for (key, mut asset, err) in errors {
                let message = if let Some((key, _)) =
                    key.and_then(|key| asset.take().map(|asset| (key, asset)))
                {
                    format!("{}: {}", key, err)
                } else {
                    format!("{}", err)
                };
                if at_least_one_download_ok {
                    println!("   {}", style(format!("WARN: {}", message)).yellow());
                } else {
                    println!("   {}", style(format!("ERR: {}", message)).red().bold());
                }
            }

            let message = Message::new("=> updating item links".to_string());
            let path = directory.as_ref().join(&item.id).with_extension("json");
            match update_links(&mut item, &path) {
                Ok(()) => message.finish_blue(),
                Err(err) => {
                    message.finish_red();
                    crate::print_err(&err.to_string());
                    return 1;
                }
            }

            let message = Message::new(format!(
                "=> saving item to {}",
                style(path.to_string_lossy()).bold()
            ));
            match stac_async::write_json_to_path(path, item).await {
                Ok(()) => {
                    message.finish_blue();
                    0
                }
                Err(err) => {
                    message.finish_red();
                    crate::print_err(&err.to_string());
                    1
                }
            }
        }
        Err(err) => {
            crate::print_err(&err.to_string());
            1
        }
    }
}

async fn download_asset(
    client: Client,
    key: String,
    mut asset: Asset,
    directory: impl AsRef<Path>,
    progress_bar: ProgressBar,
) -> Result<(String, Asset), Error> {
    progress_bar.set_style(
        ProgressStyle::with_template(
            "=> => [{elapsed_precise}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}",
        )
        .unwrap(),
    );
    progress_bar.set_message(format!("{}: {}", key, asset.href));
    let url = Url::parse(&asset.href)?;
    let file_name = url
        .path_segments()
        .and_then(|s| s.last().map(|s| s.to_string()))
        .ok_or_else(|| Error::NoFileName(url.clone()))?;
    let mut response = client
        .get(url)
        .send()
        .await
        .and_then(|response| response.error_for_status())?;
    if let Some(content_length) = response.content_length() {
        progress_bar.set_length(content_length);
    }
    let path = directory.as_ref().join(&file_name);
    let mut file = File::create(path).await?;
    while let Some(chunk) = response.chunk().await? {
        progress_bar.inc(chunk.len() as u64);
        file.write_all(&chunk).await?;
    }
    asset.href = format!("./{}", file_name);
    progress_bar.finish_and_clear();
    Ok((key, asset))
}

fn update_links(item: &mut Item, path: impl AsRef<Path>) -> Result<(), stac::Error> {
    item.make_relative_links_absolute()?;
    if let Some(mut link) = item.self_link().cloned() {
        link.rel = "canonical".to_string();
        item.set_link(link)
    }
    item.set_link(Link::self_(path.as_ref().to_string_lossy().into_owned()));
    Ok(())
}
