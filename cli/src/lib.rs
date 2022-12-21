mod download;
mod message;
mod validate;

use message::Message;
pub use {download::download, validate::validate};

async fn read(href: &str) -> Result<stac::Value, stac_async::Error> {
    let message = Message::new(format!("=> reading {}", console::style(href).bold()));
    match stac_async::read(href).await {
        Ok(value) => {
            message.finish_blue();
            Ok(value)
        }
        Err(err) => {
            message.finish_red();
            return Err(err);
        }
    }
}

fn print_err(s: &str) {
    println!(
        "{}",
        console::style(format!("   ERROR: {}", s)).red().bold()
    );
}
