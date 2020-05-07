use ansi_term::Color;
use nu_protocol::{CallInfo, Value};
use std::path::Path;
use std::process::{Command, Stdio};

pub struct Start {
    pub filenames: Vec<String>,
    pub application: Option<String>,
}

impl Start {
    pub fn parse(&mut self, call_info: CallInfo, input: Vec<Value>) {
        input.iter().for_each(|val| {
            if val.is_some() {
                self.parse_value(val);
            }
        });
        self.parse_filenames(&call_info);
        self.parse_application(&call_info);
    }

    fn add_filename(&mut self, filename: String) {
        if Path::new(&filename).exists() || url::Url::parse(&filename).is_ok() {
            self.filenames.push(filename);
        } else {
            print_warning(format!(
                "The file '{}' does not exist",
                Color::White.bold().paint(filename)
            ));
        }
    }

    fn parse_filenames(&mut self, call_info: &CallInfo) {
        let candidates = match &call_info.args.positional {
            Some(values) => values
                .iter()
                .map(|val| val.as_string())
                .collect::<Result<Vec<String>, _>>()
                .unwrap_or(vec![]),
            None => vec![],
        };

        for candidate in candidates {
            self.add_filename(candidate);
        }
    }

    fn parse_application(&mut self, call_info: &CallInfo) {
        self.application = if let Some(app) = call_info.args.get("application") {
            match app.as_string() {
                Ok(name) => Some(name),
                Err(_) => None,
            }
        } else {
            None
        };
    }

    pub fn parse_value(&mut self, input: &Value) {
        if let Ok(filename) = input.as_string() {
            self.add_filename(filename);
        } else {
            print_warning(format!("Could not convert '{:?}' to string", input));
        }
    }

    #[cfg(target_os = "macos")]
    pub fn exec(&mut self) {
        let mut args = vec![];
        args.append(&mut self.filenames);

        if let Some(app_name) = &self.application {
            args.append(&mut vec![String::from("-a"), app_name.to_string()]);
        }

        Command::new("open")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args(&args)
            .spawn()
            .unwrap();
    }
    #[cfg(target_os = "windows")]
    pub fn exec(&mut self) {}

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    pub fn exec(&mut self) {
        // executing on linux
    }
}

fn print_warning(msg: String) {
    println!("{}: {}", Color::Yellow.bold().paint("warning"), msg);
}
