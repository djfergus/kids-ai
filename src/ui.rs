use crossterm::cursor::MoveToColumn;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::ExecutableCommand;
use std::io::{self, Write};

pub fn print_welcome(child_name: Option<&str>) {
    let mut stdout = io::stdout();

    let _ = stdout.execute(SetForegroundColor(Color::Yellow));
    println!("==========================================");
    match child_name {
        Some(name) => println!("  Hi {name}! Welcome to Kids AI!"),
        None => println!("  Welcome to Kids AI! Ask me anything!"),
    }
    println!("==========================================");
    let _ = stdout.execute(ResetColor);

    println!("Type \"quit\" or \"exit\" when you're done.");
    println!();
}

pub fn print_thinking() {
    let mut stdout = io::stdout();
    let _ = stdout.execute(SetForegroundColor(Color::DarkGrey));
    print!("\nThinking...");
    let _ = stdout.execute(ResetColor);
    let _ = stdout.flush();
}

pub fn clear_thinking() {
    let mut stdout = io::stdout();
    let _ = stdout.execute(MoveToColumn(0));
    let _ = stdout.execute(Clear(ClearType::CurrentLine));
    let _ = stdout.flush();
}

pub fn print_ai_prefix() {
    let mut stdout = io::stdout();
    let _ = stdout.execute(SetForegroundColor(Color::Cyan));
    print!("AI> ");
    let _ = stdout.execute(ResetColor);
    let _ = stdout.flush();
}

pub fn print_ai_done() {
    println!();
    println!();
}

pub fn print_error(msg: &str) {
    let mut stdout = io::stdout();
    let _ = stdout.execute(SetForegroundColor(Color::Red));
    println!("\nOops! {msg}");
    let _ = stdout.execute(ResetColor);
    println!();
}

pub fn print_goodbye(child_name: Option<&str>) {
    let mut stdout = io::stdout();
    let _ = stdout.execute(SetForegroundColor(Color::Yellow));
    match child_name {
        Some(name) => println!("\nBye {name}! See you next time! 👋"),
        None => println!("\nBye! See you next time! 👋"),
    }
    let _ = stdout.execute(ResetColor);
}

pub fn prompt_string() -> String {
    format!("{}You> {}", SetForegroundColor(Color::Green), ResetColor)
}

/// Handles word-wrapping of streamed tokens to fit the terminal width.
pub struct WordWrapper {
    width: usize,
    col: usize,
    word_buf: String,
}

impl WordWrapper {
    /// Create a new wrapper. `initial_col` accounts for the "AI> " prefix already printed.
    pub fn new(initial_col: usize) -> Self {
        let width = terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
        Self {
            width,
            col: initial_col,
            word_buf: String::new(),
        }
    }

    /// Feed a streaming token chunk. Flushes complete words to stdout with wrapping.
    pub fn push(&mut self, token: &str) {
        for ch in token.chars() {
            match ch {
                '\n' => {
                    self.flush_word();
                    print!("\n");
                    self.col = 0;
                }
                ' ' | '\t' => {
                    self.flush_word();
                    if self.col > 0 && self.col < self.width {
                        // Mid-line: print the separator space.
                        print!(" ");
                        self.col += 1;
                    } else if self.col >= self.width {
                        // Word just filled the line exactly; terminal wraps
                        // naturally, so just reset the column counter.
                        self.col = 0;
                    }
                    // col == 0: we're at line start, skip the space.
                }
                _ => {
                    self.word_buf.push(ch);
                }
            }
        }
        // Flush if we have a long word that fills the terminal on its own
        let buf_len = self.word_buf.chars().count();
        if self.col + buf_len > self.width && buf_len >= self.width {
            self.flush_word();
        }
        let _ = io::stdout().flush();
    }

    /// Flush any remaining buffered word at end of response.
    pub fn finish(&mut self) {
        self.flush_word();
        let _ = io::stdout().flush();
    }

    fn flush_word(&mut self) {
        if self.word_buf.is_empty() {
            return;
        }

        let word_len = self.word_buf.chars().count();

        // Wrap to next line if this word won't fit
        if self.col > 0 && self.col + word_len > self.width {
            print!("\n");
            self.col = 0;
        }

        print!("{}", self.word_buf);
        self.col += word_len;
        self.word_buf.clear();
    }
}
