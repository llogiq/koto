use {
    crate::help::Help,
    koto::{bytecode::Chunk, Koto, KotoSettings},
    std::{
        fmt,
        io::{self, Stdout, Write},
    },
    termion::{
        clear, color, cursor, cursor::DetectCursorPos, event::Key, input::TermRead,
        raw::IntoRawMode, raw::RawTerminal, style,
    },
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const PROMPT: &str = "» ";
const CONTINUED: &str = "… ";

const INDENT_SIZE: usize = 2;

#[derive(Default)]
pub struct ReplSettings {
    pub show_bytecode: bool,
    pub show_instructions: bool,
}

#[derive(Default)]
pub struct Repl {
    koto: Koto,
    settings: ReplSettings,
    help: Option<Help>,
    input: String,
    continued_lines: Vec<String>,
    input_history: Vec<String>,
    history_position: Option<usize>,
    cursor: Option<usize>,
}

impl Repl {
    pub fn with_settings(repl_settings: ReplSettings, mut koto_settings: KotoSettings) -> Self {
        koto_settings.repl_mode = true;

        let koto = Koto::with_settings(koto_settings);

        let mut prelude = koto.prelude();
        prelude.add_map("json", koto_json::make_module());
        prelude.add_map("random", koto_random::make_module());
        prelude.add_map("tempfile", koto_tempfile::make_module());
        prelude.add_map("toml", koto_toml::make_module());

        Self {
            koto,
            settings: repl_settings,
            ..Self::default()
        }
    }

    pub fn run(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut tty = if termion::is_tty(&stdout) {
            match termion::get_tty() {
                Ok(tty) => Some(tty.into_raw_mode().expect("Failed to activate raw mode")),
                Err(_) => None,
            }
        } else {
            None
        };

        write!(stdout, "Welcome to Koto v{}\r\n{}", VERSION, PROMPT).unwrap();
        stdout.flush().unwrap();

        for c in stdin.keys() {
            self.on_keypress(c.unwrap(), &mut stdout, &mut tty);

            if let Some(ref mut tty) = tty {
                let (_, cursor_y) = stdout.cursor_pos().unwrap();

                let prompt = if self.continued_lines.is_empty() {
                    PROMPT
                } else {
                    CONTINUED
                };

                write!(
                    tty,
                    "{move_cursor}{clear}{prompt}{input}",
                    move_cursor = cursor::Goto(1, cursor_y),
                    clear = clear::CurrentLine,
                    prompt = prompt,
                    input = self.input
                )
                .unwrap();

                if let Some(position) = self.cursor {
                    if position < self.input.len() {
                        let x_offset = (self.input.len() - position) as u16;
                        let (cursor_x, cursor_y) = stdout.cursor_pos().unwrap();
                        write!(tty, "{}", cursor::Goto(cursor_x - x_offset, cursor_y),).unwrap();
                    }
                }
            }

            stdout.flush().unwrap();
        }
    }

    fn on_keypress<T>(&mut self, key: Key, stdout: &mut Stdout, tty: &mut Option<RawTerminal<T>>)
    where
        T: Write,
    {
        match key {
            Key::Up => {
                if !self.input_history.is_empty() {
                    let new_position = match self.history_position {
                        Some(position) => {
                            if position > 0 {
                                position - 1
                            } else {
                                0
                            }
                        }
                        None => self.input_history.len() - 1,
                    };
                    self.input = self.input_history[new_position].clone();
                    self.cursor = None;
                    self.history_position = Some(new_position);
                }
            }
            Key::Down => {
                self.history_position = match self.history_position {
                    Some(position) => {
                        if position < self.input_history.len() - 1 {
                            Some(position + 1)
                        } else {
                            None
                        }
                    }
                    None => None,
                };
                if let Some(position) = self.history_position {
                    self.input = self.input_history[position].clone();
                } else {
                    self.input.clear();
                }
                self.cursor = None;
            }
            Key::Left => match self.cursor {
                Some(position) => {
                    if position > 0 {
                        self.cursor = Some(position - 1);
                    }
                }
                None => {
                    if !self.input.is_empty() {
                        self.cursor = Some(self.input.len() - 1);
                    }
                }
            },
            Key::Right => {
                if let Some(position) = self.cursor {
                    if position < self.input.len() - 1 {
                        self.cursor = Some(position + 1);
                    } else {
                        self.cursor = None;
                    }
                }
            }
            Key::Backspace => {
                let cursor = self.cursor;
                match cursor {
                    Some(position) => {
                        let new_position = position - 1;
                        self.input.remove(new_position);
                        if self.input.is_empty() {
                            self.cursor = None;
                        } else {
                            self.cursor = Some(new_position);
                        }
                    }
                    None => {
                        self.input.pop();
                    }
                }
            }
            Key::Char(c) => match c {
                '\n' => self.on_enter(stdout, tty),
                _ => {
                    let cursor = self.cursor;
                    match cursor {
                        Some(position) => {
                            self.input.insert(position, c);
                            self.cursor = Some(position + 1);
                        }
                        None => self.input.push(c),
                    }
                }
            },
            Key::Ctrl(c) => match c {
                'c' => {
                    if self.input.is_empty() {
                        write!(stdout, "^C\r\n").unwrap();
                        stdout.flush().unwrap();
                        if let Some(tty) = tty {
                            tty.suspend_raw_mode().unwrap();
                        }
                        std::process::exit(0)
                    } else {
                        self.input.clear();
                        self.cursor = None;
                    }
                }
                'd' if self.input.is_empty() => {
                    write!(stdout, "^D\r\n").unwrap();
                    stdout.flush().unwrap();
                    if let Some(tty) = tty {
                        tty.suspend_raw_mode().unwrap();
                    }
                    std::process::exit(0)
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn on_enter<T>(&mut self, stdout: &mut Stdout, tty: &mut Option<RawTerminal<T>>)
    where
        T: Write,
    {
        write!(stdout, "\r\n").unwrap();

        if let Some(tty) = tty {
            tty.suspend_raw_mode().unwrap();
        }

        let mut indent_next_line = false;

        let input_is_whitespace = self.input.chars().all(char::is_whitespace);

        if self.continued_lines.is_empty() || input_is_whitespace {
            let mut input = self.continued_lines.join("\n");

            if !input_is_whitespace {
                input += &self.input;
            }

            match self.koto.compile(&input) {
                Ok(chunk) => {
                    if self.settings.show_bytecode {
                        println!("{}\n", &Chunk::bytes_as_string(chunk.clone()));
                    }
                    if self.settings.show_instructions {
                        println!("Constants\n---------\n{}\n", chunk.constants.to_string());

                        let script_lines = input.lines().collect::<Vec<_>>();
                        println!(
                            "Instructions\n------------\n{}",
                            Chunk::instructions_as_string(chunk, &script_lines)
                        );
                    }
                    match self.koto.run() {
                        Ok(result) => writeln!(stdout, "{}\n", result).unwrap(),
                        Err(error) => {
                            if input.trim() == "help" {
                                let help = self.get_help(None);
                                writeln!(stdout, "{}\n", help).unwrap();
                            } else if input.starts_with("help") {
                                match input.trim().splitn(2, char::is_whitespace).skip(1).next() {
                                    Some(search) => {
                                        let help = self.get_help(Some(search));
                                        writeln!(stdout, "\n{}\n", help).unwrap();
                                    }
                                    _ => self.print_error(stdout, tty, &error),
                                }
                            } else {
                                self.print_error(stdout, tty, &error)
                            }
                        }
                    }
                    self.continued_lines.clear();
                }
                Err(e) => {
                    if e.is_indentation_error() && self.continued_lines.is_empty() {
                        self.continued_lines.push(self.input.clone());
                        indent_next_line = true;
                    } else {
                        self.print_error(stdout, tty, &e.to_string());
                        self.continued_lines.clear();
                    }
                }
            }
        } else {
            // We're in a continued expression, so cache the input for execution later
            self.continued_lines.push(self.input.clone());

            // Check if we should add indentation on the next line
            let input = self.continued_lines.join("\n");
            if let Err(e) = self.koto.compile(&input) {
                if e.is_indentation_error() {
                    indent_next_line = true;
                }
            }
        }

        if let Some(tty) = tty {
            tty.activate_raw_mode().unwrap();
        }

        if !input_is_whitespace
            && (self.input_history.is_empty() || self.input_history.last().unwrap() != &self.input)
        {
            self.input_history.push(self.input.clone());
        }

        self.history_position = None;
        self.cursor = None;

        let current_indent = if self.continued_lines.is_empty() {
            0
        } else {
            self.continued_lines
                .last()
                .unwrap()
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(0)
        };

        let indent = if indent_next_line {
            current_indent + INDENT_SIZE
        } else {
            current_indent
        };

        self.input = " ".repeat(indent);
    }

    fn get_help(&mut self, search: Option<&str>) -> String {
        let help = self.help.get_or_insert_with(|| Help::new());
        help.get_help(search)
    }

    fn print_error<T, E>(&self, stdout: &mut Stdout, tty: &mut Option<RawTerminal<T>>, error: &E)
    where
        T: Write,
        E: fmt::Display,
    {
        if let Some(tty) = tty {
            write!(
                tty,
                "{red}error{reset}: {bold}",
                red = color::Fg(color::Red),
                bold = style::Bold,
                reset = style::Reset,
            )
            .unwrap();
            tty.suspend_raw_mode().unwrap();
            println!("{:#}\n", error);
            tty.activate_raw_mode().unwrap();
            write!(tty, "{}", style::Reset).unwrap();
        } else {
            write!(stdout, "{:#}\n", error).unwrap();
        }
    }
}
