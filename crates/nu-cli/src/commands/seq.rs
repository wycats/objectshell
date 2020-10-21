use crate::commands::WholeStreamCommand;
use crate::prelude::*;
use nu_errors::ShellError;
use nu_protocol::{Primitive, ReturnSuccess, Signature, SyntaxShape, UntaggedValue, Value};
use std::cmp;
// use std::io::{stdout, Write};

// static NAME: &str = "seq";
// static VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Seq;

#[derive(Deserialize)]
pub struct SeqArgs {
    separator: Option<Value>,
    terminator: Option<Value>,
    widths: bool,
    rest: Vec<Value>,
}

#[async_trait]
impl WholeStreamCommand for Seq {
    fn name(&self) -> &str {
        "seq"
    }

    fn signature(&self) -> Signature {
        Signature::build("seq")
            .named(
                "separator",
                SyntaxShape::String,
                "separator character (defaults to \\n)",
                Some('s'),
            )
            .named(
                "terminator",
                SyntaxShape::String,
                "terminator character (defaults to separator)",
                Some('t'),
            )
            .switch(
                "widths",
                "equalize widths of all numbers by padding with zeros",
                Some('w'),
            )
            .rest(SyntaxShape::Any, "sequence values")
    }

    fn usage(&self) -> &str {
        "print sequences of numbers"
    }

    async fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        seq(args, registry).await
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Select just the name column",
                example: "ls | select name",
                result: None,
            },
            Example {
                description: "Select the name and size columns",
                example: "ls | select name size",
                result: None,
            },
        ]
    }
}

async fn seq(args: CommandArgs, registry: &CommandRegistry) -> Result<OutputStream, ShellError> {
    let registry = registry.clone();
    let (
        SeqArgs {
            separator,
            terminator,
            widths,
            rest,
        },
        _input,
    ) = args.process(&registry).await?;

    let sep = match separator {
        Some(Value {
            value: UntaggedValue::Primitive(Primitive::String(s)),
            tag,
            ..
        }) => {
            if s == r"\t" {
                '\t'
            } else {
                let vec_s: Vec<char> = s.chars().collect();
                if vec_s.len() != 1 {
                    return Err(ShellError::labeled_error(
                        "Expected a single separator char from --separator",
                        "requires a single character string input",
                        tag,
                    ));
                };
                vec_s[0]
            }
        }
        _ => '\n',
        // None => Some(Value {
        //     value: UntaggedValue::Primitive(Primitive::String("\n".to_string())),
        //     tag: Tag::unknown(),
        // }),
    };

    let term = match terminator {
        Some(Value {
            value: UntaggedValue::Primitive(Primitive::String(s)),
            tag,
            ..
        }) => {
            if s == r"\t" {
                '\t'
            } else {
                let vec_s: Vec<char> = s.chars().collect();
                if vec_s.len() != 1 {
                    return Err(ShellError::labeled_error(
                        "expected a single terminator char from --terminator",
                        "requires a single character string input",
                        tag,
                    ));
                };
                vec_s[0]
            }
        }
        _ => sep,
    };

    run_seq(sep.to_string(), Some(term.to_string()), widths, rest)
}

#[cfg(test)]
mod tests {
    use super::Seq;
    use super::ShellError;

    #[test]
    fn examples_work_as_expected() -> Result<(), ShellError> {
        use crate::examples::test as test_examples;

        Ok(test_examples(Seq {})?)
    }
}

// #[derive(Clone)]
// struct SeqOptions {
//     separator: String,
//     terminator: Option<String>,
//     widths: bool,
// }

fn parse_float(mut s: &str) -> Result<f64, String> {
    if s.starts_with('+') {
        s = &s[1..];
    }
    match s.parse() {
        Ok(n) => Ok(n),
        Err(e) => Err(format!(
            "seq: invalid floating point argument `{}`: {}",
            s, e
        )),
    }
}

fn escape_sequences(s: &str) -> String {
    s.replace("\\n", "\n").replace("\\t", "\t")
}

pub fn run_seq(
    sep: String,
    termy: Option<String>,
    widths: bool,
    rest: Vec<Value>,
) -> Result<OutputStream, ShellError> {
    let free: Vec<String> = rest
        .iter()
        .map(|v| v.as_string().expect("error mapping rest"))
        .collect();
    let mut largest_dec = 0;
    let mut padding = 0;
    let first = if free.len() > 1 {
        let slice = &free[0][..];
        let len = slice.len();
        let dec = slice.find('.').unwrap_or(len);
        largest_dec = len - dec;
        padding = dec;
        match parse_float(slice) {
            Ok(n) => n,
            Err(s) => {
                // show_error!("{}", s);
                // return 1;
                return Err(ShellError::labeled_error(
                    format!("{}", s),
                    "error parsing float",
                    Tag::unknown(),
                ));
            }
        }
    } else {
        1.0
    };
    let step = if free.len() > 2 {
        let slice = &free[1][..];
        let len = slice.len();
        let dec = slice.find('.').unwrap_or(len);
        largest_dec = cmp::max(largest_dec, len - dec);
        padding = cmp::max(padding, dec);
        match parse_float(slice) {
            Ok(n) => n,
            Err(s) => {
                // show_error!("{}", s);
                // return 1;
                return Err(ShellError::labeled_error(
                    format!("{}", s),
                    "error parsing float",
                    Tag::unknown(),
                ));
            }
        }
    } else {
        1.0
    };
    let last = {
        let slice = &free[free.len() - 1][..];
        padding = cmp::max(padding, slice.find('.').unwrap_or_else(|| slice.len()));
        match parse_float(slice) {
            Ok(n) => n,
            Err(s) => {
                // show_error!("{}", s);
                // return 1;
                return Err(ShellError::labeled_error(
                    format!("{}", s),
                    "error parsing float",
                    Tag::unknown(),
                ));
            }
        }
    };
    if largest_dec > 0 {
        largest_dec -= 1;
    }
    let separator = escape_sequences(&sep[..]);
    let terminator = match termy {
        Some(term) => escape_sequences(&term[..]),
        None => separator.clone(),
    };
    print_seq(
        first,
        step,
        last,
        largest_dec,
        separator,
        terminator,
        widths,
        padding,
    )

    // Ok(0)
}

fn done_printing(next: f64, step: f64, last: f64) -> bool {
    if step >= 0f64 {
        next > last
    } else {
        next < last
    }
}

#[allow(clippy::too_many_arguments)]
fn print_seq(
    first: f64,
    step: f64,
    last: f64,
    largest_dec: usize,
    separator: String,
    terminator: String,
    pad: bool,
    padding: usize,
) -> Result<OutputStream, ShellError> {
    let mut i = 0isize;
    let mut value = first + i as f64 * step;
    let mut ret_str = "".to_owned();
    while !done_printing(value, step, last) {
        let istr = format!("{:.*}", largest_dec, value);
        let ilen = istr.len();
        let before_dec = istr.find('.').unwrap_or(ilen);
        if pad && before_dec < padding {
            for _ in 0..(padding - before_dec) {
                // print!("0");
                ret_str.push_str("0");
            }
        }
        // print!("{}", istr);
        ret_str.push_str(&istr);
        i += 1;
        value = first + i as f64 * step;
        if !done_printing(value, step, last) {
            // print!("{}", separator);
            ret_str.push_str(&separator);
        }
    }
    if (first >= last && step < 0f64) || (first <= last && step > 0f64) {
        // print!("{}", terminator);
        ret_str.push_str(&terminator);
    }

    Ok(OutputStream::one(ReturnSuccess::value(ret_str)))
}
