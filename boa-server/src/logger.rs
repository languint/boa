use owo_colors::{OwoColorize, Style};

#[derive(Clone)]
pub struct Logger {
    pub prefix: String,
}

impl Logger {
    pub fn new(prefix: String) -> Logger {
        Logger { prefix }
    }
}

impl Logger {
    pub fn log(&self, message: impl ToString, postfix: impl ToString) {
        println!(
            "[{}{}]: {}",
            self.prefix,
            postfix.to_string(),
            message.to_string()
        )
    }

    pub fn err(&self, message: impl ToString, postfix: impl ToString) {
        println!(
            "{}: {}",
            format!(
                "[{}{}]",
                self.prefix.bright_red(),
                postfix.to_string().bright_red()
            ),
            message.to_string().bright_red()
        );
    }

    pub fn log_style(&self, message: impl ToString, style: Style, postfix: impl ToString) {
        println!(
            "[{}{}]: {}",
            self.prefix,
            postfix.to_string(),
            message.to_string().style(style)
        )
    }
}
