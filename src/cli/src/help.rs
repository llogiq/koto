use {indexmap::IndexMap, std::iter::Peekable};

pub struct Help {
    map: IndexMap<String, String>,
}

impl Help {
    pub fn new() -> Self {
        let mut result = Self {
            map: IndexMap::new(),
        };

        result.add_help_from_markdown(include_str!("../../../docs/reference/iterator.md"));
        result.add_help_from_markdown(include_str!("../../../docs/reference/io.md"));
        result.add_help_from_markdown(include_str!("../../../docs/reference/list.md"));
        result.add_help_from_markdown(include_str!("../../../docs/reference/map.md"));
        result.add_help_from_markdown(include_str!("../../../docs/reference/number.md"));
        result.add_help_from_markdown(include_str!("../../../docs/reference/num2.md"));
        result.add_help_from_markdown(include_str!("../../../docs/reference/num4.md"));
        result.add_help_from_markdown(include_str!("../../../docs/reference/string.md"));
        result.add_help_from_markdown(include_str!("../../../docs/reference/tuple.md"));

        result.add_help_from_markdown(include_str!("../../../docs/reference/file.md"));

        result
    }

    pub fn get_help(&self, search: Option<&str>) -> String {
        match search {
            Some(search) => {
                let search = search.trim();
                match self.map.get(search) {
                    Some(help) => help.into(),
                    None => {
                        let matches = self
                            .map
                            .keys()
                            .filter(|key| key.contains(search))
                            .collect::<Vec<_>>();
                        match matches.as_slice() {
                            [] => format!("Help for '{}' not found.", search),
                            [only_match] => self.get_help(Some(only_match)),
                            _ => {
                                let mut help = String::new();
                                help.push_str("Possible matches: ");
                                for maybe in matches {
                                    help.push_str("\n  ");
                                    help.push_str(maybe);
                                }
                                help
                            }
                        }
                    }
                }
            }
            None => {
                let mut help = "
To get help for a module, type `help <module>` (e.g. `help map`),
or to for an item in a module, type `help <module>.<item>` (e.g. `help map.keys`).

Help is available for the following modules:
  "
                .to_string();

                for (i, module) in self
                    .map
                    .keys()
                    .filter(|item| !item.contains("."))
                    .enumerate()
                {
                    if i > 0 {
                        help.push_str(", ");
                    }
                    help.push_str(module);
                }

                help
            }
        }
    }

    fn add_help_from_markdown(&mut self, markdown: &str) {
        use pulldown_cmark::{Event, Parser, Tag};

        let mut parser = Parser::new(markdown).peekable();

        // Consume the module overview section
        let (module_name, help) = consume_help_section(&mut parser, None);
        self.map.insert(module_name.clone(), help);

        // Skip ahead until the first reference subsection is found
        while let Some(peeked) = parser.peek() {
            if matches!(peeked, Event::Start(Tag::Heading(2))) {
                break;
            }
            parser.next();
        }

        // Consume each module entry
        while parser.peek().is_some() {
            let (entry_name, help) = consume_help_section(&mut parser, Some(&module_name));
            self.map.insert(entry_name, help);
        }
    }
}

fn consume_help_section<'a>(
    parser: &mut Peekable<pulldown_cmark::Parser<'a>>,
    module_name: Option<&str>,
) -> (String, String) {
    use pulldown_cmark::{Event::*, Tag::*};

    let mut section_level = None;
    let mut section_name = String::new();
    let mut result = String::new();

    let mut list_indent = 0;
    let mut heading_start = 0;
    let mut first_heading = true;
    let mut in_code_block = false;

    while let Some(peeked) = parser.peek() {
        match peeked {
            Start(Heading(level)) => {
                match section_level {
                    Some(section_level) if section_level >= *level => {
                        // We've reached the end of the section, so break out
                        break;
                    }
                    Some(_) => {
                        // Start a new subsection
                        result.push_str("\n\n");
                    }
                    None => section_level = Some(*level),
                }
                heading_start = result.len();
            }
            End(Heading(_)) => {
                let heading_length = result.len() - heading_start;
                result.push_str("\n");
                let heading_underline = if first_heading { "=" } else { "-" };
                for _ in 0..heading_length {
                    result.push_str(heading_underline)
                }
                first_heading = false;
            }
            Start(Link(_type, _url, title)) => result.push_str(title),
            End(Link(_, _, _)) => {}
            Start(List(_)) => {
                if list_indent == 0 {
                    result.push_str("\n");
                }
                list_indent += 1;
            }
            End(List(_)) => list_indent -= 1,
            Start(Item) => {
                result.push_str("\n");
                for _ in 1..list_indent {
                    result.push_str("  ");
                }
                result.push_str("- ");
            }
            End(Item) => {}
            Start(Paragraph) => result.push_str("\n\n"),
            End(Paragraph) => {}
            Start(CodeBlock(_)) => {
                result.push_str("\n\n");
                in_code_block = true;
            }
            End(CodeBlock(_)) => in_code_block = false,
            Start(Emphasis) => result.push_str("_"),
            End(Emphasis) => result.push_str("_"),
            Start(Strong) => result.push_str("*"),
            End(Strong) => result.push_str("*"),
            Text(text) => {
                if section_name.is_empty() {
                    if let Some(module_name) = module_name {
                        section_name = format!("{}.{}", module_name, text);
                    } else {
                        section_name = text.to_string();
                    }
                    result.push_str(&section_name);
                } else {
                    if in_code_block {
                        for (i, line) in text.split('\n').enumerate() {
                            if i == 0 {
                                result.push_str("|");
                            }
                            result.push_str("\n|  ");
                            result.push_str(line);
                        }
                    } else {
                        result.push_str(text);
                    }
                }
            }
            Code(code) => {
                result.push_str("`");
                if section_name.is_empty() {
                    if let Some(module_name) = module_name {
                        section_name = format!("{}.{}", module_name, code);
                    } else {
                        section_name = code.to_string();
                    }
                    result.push_str(&section_name);
                } else {
                    result.push_str(code);
                }
                result.push_str("`");
            }
            SoftBreak => result.push_str(" "),
            HardBreak => result.push_str("\n"),
            _other => {}
        }

        parser.next();
    }

    (section_name, result)
}
