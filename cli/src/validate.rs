use crate::Message;
use stac::{Error, Validate};

pub async fn validate(href: String) -> i32 {
    use Error::*;

    match super::read(&href).await {
        Ok(value) => {
            let message = Message::new(format!(
                "=> validating {}",
                value.type_name().to_lowercase()
            ));
            // TODO implement async validation
            tokio::task::spawn_blocking(move || match value.validate() {
                Ok(()) => {
                    message.finish_blue();
                    println!("{}", console::style("   OK!").green().bold());
                    0
                }
                Err(errs) => {
                    message.finish_red();
                    println!("{}", console::style("   FAILED!").red().bold());
                    for err in errs {
                        match err {
                            ValidationError(e) => println!(
                                "   {} {} @ {}",
                                console::style("(jsonschema)").dim(),
                                console::style(e.to_string()).bold(),
                                console::style(e.instance_path.clone()).bold().yellow(),
                            ),
                            _ => println!(
                                "   {} {}",
                                console::style("(other)").dim(),
                                console::style(err.to_string()).bold(),
                            ),
                        }
                    }
                    1
                }
            })
            .await
            .unwrap()
        }
        Err(err) => {
            crate::print_err(&err.to_string());
            1
        }
    }
}
