pub use koto_parser::{AstNode, KotoParser as Parser, LookupSliceOrId, LookupOrId, Position};
use koto_runtime::Runtime;
pub use koto_runtime::{Error, RuntimeResult, Value, ValueVec, ValueList, ValueMap};
use std::{path::Path, rc::Rc};

#[derive(Default)]
pub struct Koto<'a> {
    script: String,
    parser: Parser,
    ast: AstNode,
    runtime: Runtime<'a>,
}

impl<'a> Koto<'a> {
    pub fn new() -> Self {
        let mut result = Self::default();

        koto_std::register(&mut result.runtime);

        let mut env = ValueMap::new();
        env.add_value("script_dir", Value::Empty);
        env.add_value("script_path", Value::Empty);
        env.add_list("args", ValueList::new());
        result.runtime.global_mut().add_map("env", env);

        result
    }

    pub fn run_script_with_args(
        &mut self,
        script: &str,
        args: Vec<String>,
    ) -> Result<Value<'a>, String> {
        self.parse(script)?;
        self.set_args(args);
        self.run()?;
        if self.has_function("main") {
            self.call_function("main")
        } else {
            Ok(Value::Empty)
        }
    }

    pub fn parse(&mut self, script: &str) -> Result<(), String> {
        match self.parser.parse(&script) {
            Ok(ast) => {
                self.script = script.to_string();
                self.ast = ast;
                Ok(())
            }
            Err(e) => Err(format!("Error while parsing script: {}", e)),
        }
    }

    pub fn set_args(&mut self, args: Vec<String>) {
        use Value::{Map, Str};

        let koto_args = args
            .iter()
            .map(|arg| Str(Rc::new(arg.to_string())))
            .collect::<ValueVec>();

        match self
            .runtime
            .global_mut()
            .0
            .get_mut(&Rc::new("env".to_string()))
            .unwrap()
        {
            Map(map) => map.borrow_mut().add_list("args", ValueList::with_data(koto_args)),
            _ => unreachable!(),
        }
    }

    pub fn set_script_path(&mut self, path: Option<String>) {
        use Value::{Empty, Map, Str};

        let (script_dir, script_path) = match &path {
            Some(path) => (
                Path::new(&path)
                    .parent()
                    .map(|p| {
                        Str(Rc::new(
                            p.to_str().expect("invalid script path").to_string(),
                        ))
                    })
                    .or(Some(Empty))
                    .unwrap(),
                Str(Rc::new(path.to_string())),
            ),
            None => (Empty, Empty),
        };

        self.runtime.set_script_path(path);

        match self.runtime.global_mut().0.get_mut("env").unwrap() {
            Map(map) => {
                let mut map = map.borrow_mut();
                map.add_value("script_dir", script_dir);
                map.add_value("script_path", script_path);
            }
            _ => unreachable!(),
        }
    }

    pub fn run(&mut self) -> Result<Value<'a>, String> {
        match self.runtime.evaluate(&self.ast) {
            Ok(result) => Ok(result),
            Err(e) => Err(match &e {
                Error::BuiltinError { message } => format!("Builtin error: {}\n", message,),
                Error::RuntimeError {
                    message,
                    start_pos,
                    end_pos,
                } => self.format_runtime_error(message, start_pos, end_pos),
            }),
        }
    }

    pub fn has_function(&self, function_name: &str) -> bool {
        matches!(
            self.runtime.get_value(function_name),
            Some((Value::Function(_), _))
        )
    }

    pub fn call_function(&mut self, function_name: &str) -> Result<Value<'a>, String> {
        match self.runtime.lookup_and_call_function(
            &LookupSliceOrId::Id(Rc::new(function_name.to_string())),
            &vec![],
            &AstNode::default(),
        ) {
            Ok(result) => Ok(result),
            Err(e) => Err(match &e {
                Error::BuiltinError { message } => format!("Builtin error: {}\n", message,),
                Error::RuntimeError {
                    message,
                    start_pos,
                    end_pos,
                } => self.format_runtime_error(&message, start_pos, end_pos),
            }),
        }
    }

    fn format_runtime_error(
        &self,
        message: &str,
        start_pos: &Position,
        end_pos: &Position,
    ) -> String {
        let excerpt_lines = self
            .script
            .lines()
            .skip(start_pos.line - 1)
            .take(end_pos.line - start_pos.line + 1)
            .collect::<Vec<_>>();

        let line_numbers = (start_pos.line..=end_pos.line)
            .map(|n| n.to_string())
            .collect::<Vec<_>>();

        let number_width = line_numbers.iter().max_by_key(|n| n.len()).unwrap().len();

        let padding = format!("{}", " ".repeat(number_width + 2));

        let excerpt = if excerpt_lines.len() == 1 {
            let mut excerpt = format!(
                " {:>width$} | {}\n",
                line_numbers.first().unwrap(),
                excerpt_lines.first().unwrap(),
                width = number_width
            );

            excerpt += &format!(
                "{}|{}",
                padding,
                format!(
                    "{}{}",
                    " ".repeat(start_pos.column),
                    "^".repeat(end_pos.column - start_pos.column)
                ),
            );

            excerpt
        } else {
            let mut excerpt = String::new();

            for (excerpt_line, line_number) in excerpt_lines.iter().zip(line_numbers.iter()) {
                excerpt += &format!(
                    " {:>width$} | {}",
                    line_number,
                    excerpt_line,
                    width = number_width
                );
            }

            excerpt
        };

        format!(
            "Runtime error: {message}\n --> {}:{}\n{padding}|\n{excerpt}",
            start_pos.line,
            start_pos.column,
            padding = padding,
            excerpt = excerpt,
            message = message
        )
    }
}
