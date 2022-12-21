use console::style;
use indicatif::{ProgressBar, ProgressStyle};

pub struct Message {
    progress_bar: ProgressBar,
    message: String,
}

impl Message {
    pub fn new(message: String) -> Message {
        let progress_bar =
            ProgressBar::new_spinner().with_style(ProgressStyle::with_template("{msg}").unwrap());
        progress_bar.set_message(message.clone());
        Message {
            progress_bar,
            message,
        }
    }

    pub fn finish_blue(self) {
        let message = style(self.message).blue();
        self.progress_bar.finish_with_message(message.to_string());
    }

    pub fn finish_yellow(self) {
        let message = style(self.message).yellow();
        self.progress_bar.finish_with_message(message.to_string());
    }

    pub fn finish_red(self) {
        self.progress_bar
            .finish_with_message(style(self.message).red().to_string())
    }
}
