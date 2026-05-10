use crate::diagnostic::Diagnostic;

#[derive(Debug)]
pub enum IrGenError {
    BackendError(Box<dyn std::error::Error>, bool),
    Diagnostic(Diagnostic)
}

impl IrGenError {
    pub fn format(&self, line_starts: &[usize], lines: &[&str]) -> String {
        use tinycolor::Colorize;

        match self {
            Self::BackendError(e, no_color) => format!(
                "{}: backend error: {e}",
                if *no_color {
                    "error".to_string()
                } else {
                    "error".red().bold().to_string()
                }
            ),
            Self::Diagnostic(d) => d.format(line_starts, lines)
        }
    }
}
