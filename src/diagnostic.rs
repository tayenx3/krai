use tinycolor::Colorize;
use crate::span::Span;

fn line_of(pos: usize, line_starts: &[usize]) -> usize {
    match line_starts.binary_search(&pos) {
        Ok(l) => l,
        Err(l) => l - 1
    }
}

fn create_snippet(
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
    lines: &[&str],
    digit_len: usize,
    no_color: bool
) -> String {
    let mut output = String::new();

    output.push_str(&" ".repeat(digit_len + 1));
    output.push_str(&if !no_color { "│\n".cyan().to_string() } else { "│\n".to_string() });

    for line_idx in start_line..=end_line {
        let line_text = lines[line_idx];
        output.push_str(&format!(
            "{:>digit_len$} {} {line_text}\n",
            if !no_color { (line_idx + 1).to_string().cyan().to_string() } else { (line_idx + 1).to_string() },
            if !no_color { "│".cyan().to_string() } else { "│".to_string() }
        ));
        output.push_str(&format!(
            "{:>digit_len$} {} ",
            "",
            if !no_color { "│".cyan().to_string() } else { "│".to_string() }
        ));

        if line_idx == start_line && line_idx == end_line {
            output.push_str(&format!(
                "{}{}\n",
                " ".repeat(start_col),
                if !no_color { "^".repeat(end_col - start_col).red().to_string() } else { "^".repeat(end_col - start_col) }
            ));
        } else if line_idx == start_line {
            output.push_str(&format!(
                "{}{}\n",
                " ".repeat(start_col),
                if !no_color { "^".repeat(line_text.len() - start_col).red().to_string() } else { "^".repeat(line_text.len() - start_col) }
            ));
        } else if line_idx == end_line {
            output.push_str(&format!(
                "{}\n",
                if !no_color { "^".repeat(end_col).red().to_string() } else { "^".repeat(end_col) }
            ));
        } else {
            output.push_str(&format!(
                "{}\n",
                if !no_color { "^".repeat(line_text.len()).red().to_string() } else { "^".repeat(line_text.len()) }
            ));
        }
    }
    
    output.push_str(&" ".repeat(digit_len + 1));
    output.push_str(&if !no_color { "│\n".cyan().to_string() } else { "│\n".to_string() });

    output
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub path: String,
    pub msg: String,
    pub span: Span,
    pub no_color: bool,
    pub secondaries: Vec<(Option<String>, Option<Span>)>,
}

impl Diagnostic {
    pub fn format(&self, line_starts: &[usize], lines: &[&str]) -> String {
        let mut output = String::new();

        let error_prefix = if !self.no_color {
            "error".red().bold().to_string()
        } else {
            "error".to_string()
        };

        let start_line = line_of(self.span.start, line_starts);
        let start_col = self.span.start - line_starts[start_line];
        let end_line = line_of(self.span.end, line_starts);
        let end_col = self.span.end - line_starts[end_line];

        output.push_str(&format!("{error_prefix}: {}\n", self.msg));
        let digit_len = (end_line + 1).ilog10() as usize + 1;
        output.push_str(&format!(
            "{:>digit_len$} {} {}:{}:{} {}\n",
            "",
            if !self.no_color { "╭─".cyan().to_string() } else { "╭─".to_string() },
            self.path,
            start_line + 1,
            start_col + 1,
            if !self.no_color { "─".cyan().to_string() } else { "─".to_string() },
        ));
        output.push_str(&create_snippet(start_line, start_col, end_line, end_col, lines, digit_len, self.no_color));

        for (msg, span) in &self.secondaries {
            if let Some(msg) = msg { output.push_str(msg); output.push('\n') }
            if let Some(span) = span {
                let start_line = line_of(span.start, line_starts);
                let start_col = span.start - line_starts[start_line];
                let end_line = line_of(span.end, line_starts);
                let end_col = span.end - line_starts[end_line];
                let digit_len = (end_line + 1).ilog10() as usize + 1;
                output.push_str(&format!(
                    "{:>digit_len$} {} {}:{}:{} {}\n",
                    "",
                    if !self.no_color { "╭─".cyan().to_string() } else { "╭─".to_string() },
                    self.path,
                    start_line + 1,
                    start_col + 1,
                    if !self.no_color { "─".cyan().to_string() } else { "─".to_string() },
                ));
                output.push_str(&create_snippet(start_line, start_col, end_line, end_col, lines, digit_len, self.no_color));
            }
        }

        output
    }
}
