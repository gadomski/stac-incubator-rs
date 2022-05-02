use anyhow::{anyhow, Result};
use console::Emoji;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use stac::{Href, Object};
use stac_async::{AsyncRead, AsyncReader};
use std::path::{Path, PathBuf};
use tokio::{fs::File, io::AsyncWriteExt, task::JoinHandle};
use url::Url;

pub async fn download_item(href: Href, outdir: impl AsRef<Path>) -> Result<()> {
    let mut multi_progress = MultiProgress::new();
    multi_progress.println(format!(
        "[1/?] {}Reading {}...",
        Emoji::new("📘 ", ""),
        href.file_name()
    ))?;
    let reader = AsyncReader::new();
    let object = reader.read(href.clone()).await?;
    let mut item = if let Object::Item(item) = object.object {
        item
    } else {
        return Err(anyhow!("expected item, got {}", object.object.r#type()));
    };
    let count = item.assets.len();
    multi_progress.println(format!(
        "[2/{}] {}Creating {}...",
        count + 3,
        Emoji::new("📁 ", ""),
        outdir.as_ref().display()
    ))?;
    tokio::fs::create_dir_all(outdir.as_ref()).await?;
    let mut handles = Vec::new();
    for (i, asset) in item.assets.values_mut().enumerate() {
        let href = Href::new(&asset.href);
        asset.href = format!("./{}", href.file_name());
        let outfile = outdir.as_ref().join(href.file_name());
        match href {
            Href::Url(url) => {
                let handle = download_url(
                    reader.client(),
                    url,
                    outfile,
                    &mut multi_progress,
                    i + 3,
                    count + 3,
                );
                handles.push(handle);
            }
            Href::Path(_) => unimplemented!(),
        }
    }
    for handle in handles {
        handle.await??;
    }
    let outpath = outdir.as_ref().join(href.file_name());
    println!(
        "[{}/{}] {}Writing {}...",
        count + 3,
        count + 3,
        Emoji::new("📘 ", ""),
        outpath.display()
    );
    tokio::fs::write(outpath, serde_json::to_vec(&item)?).await?;
    Ok(())
}

fn download_url(
    client: Client,
    url: Url,
    outfile: PathBuf,
    multi_progress: &mut MultiProgress,
    index: usize,
    total: usize,
) -> JoinHandle<Result<()>> {
    let progress_bar = multi_progress.add(ProgressBar::new(0));
    let file_name = url.path_segments().unwrap().last().unwrap().to_string();
    tokio::spawn(async move {
        progress_bar.set_style(
            ProgressStyle::with_template(
                "{prefix} [{elapsed_precise}] [{bar:.cyan/blue}] {bytes}/{total_bytes} {wide_msg:>}",
            )?
            .with_key("eta", |state| format!("{:.1}s", state.eta().as_secs_f64()))
            .progress_chars("#>-"),
        );
        progress_bar.set_prefix(format!(
            "[{}/{}] {}Downloading...",
            index,
            total,
            Emoji::new("🔗 ", ""),
        ));
        progress_bar.set_message(file_name.to_string());
        let mut response = client.get(url.clone()).send().await?;
        if let Some(content_length) = response.content_length() {
            progress_bar.set_length(content_length);
        } else {
            return Err(anyhow!("empty content: {}", url));
        }
        let mut file = File::create(outfile).await?;
        while let Some(bytes) = response.chunk().await? {
            progress_bar.inc(bytes.len() as u64);
            file.write_all(&bytes).await?;
        }
        progress_bar.set_prefix(format!(
            "[{}/{}] {}Downloaded!   ",
            index,
            total,
            Emoji::new("✅ ", ""),
        ));
        progress_bar.finish();
        Ok(())
    })
}
