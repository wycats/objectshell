use super::{handle_value, PathSubcommandArguments};
use crate::prelude::*;
use nu_engine::WholeStreamCommand;
use nu_errors::ShellError;
use nu_protocol::{ColumnPath, ReturnSuccess, Signature, SyntaxShape, UntaggedValue, Value};
use std::path::Path;

pub struct PathSplit;

struct PathSplitArguments {
    rest: Vec<ColumnPath>,
}

impl PathSubcommandArguments for PathSplitArguments {
    fn get_column_paths(&self) -> &Vec<ColumnPath> {
        &self.rest
    }
}

impl WholeStreamCommand for PathSplit {
    fn name(&self) -> &str {
        "path split"
    }

    fn signature(&self) -> Signature {
        Signature::build("path split")
            .rest(SyntaxShape::ColumnPath, "Optionally operate by column path")
    }

    fn usage(&self) -> &str {
        "Split a path into parts along a separator."
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        let tag = args.call_info.name_tag.clone();
        let args = args.evaluate_once()?;
        let cmd_args = Arc::new(PathSplitArguments {
            rest: args.rest_args()?,
        });

        Ok(operate_split(args.input, &action, tag.span, cmd_args))
    }

    #[cfg(windows)]
    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Split a path into parts",
            example: r"echo 'C:\Users\viking\spam.txt' | path split",
            result: Some(vec![
                Value::from(UntaggedValue::string("C:")),
                Value::from(UntaggedValue::string("Users")),
                Value::from(UntaggedValue::string("viking")),
                Value::from(UntaggedValue::string("spam.txt")),
            ]),
        }]
    }

    #[cfg(not(windows))]
    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Split a path into parts",
            example: r"echo '/home/viking/spam.txt' | path split",
            result: Some(vec![
                Value::from(UntaggedValue::string("/")),
                Value::from(UntaggedValue::string("home")),
                Value::from(UntaggedValue::string("viking")),
                Value::from(UntaggedValue::string("spam.txt")),
            ]),
        }]
    }
}

fn operate_split<F, T>(
    input: crate::InputStream,
    action: &'static F,
    span: Span,
    args: Arc<T>,
) -> OutputStream
where
    T: PathSubcommandArguments + Send + Sync + 'static,
    F: Fn(&Path, Tag, &T) -> Value + Send + Sync + 'static,
{
    if args.get_column_paths().is_empty() {
        // Do not wrap result into a table
        input
            .flat_map(move |v| {
                let split_result = handle_value(&action, &v, span, Arc::clone(&args));

                match split_result {
                    Ok(val) => {
                        if let Value {
                            value: UntaggedValue::Table(parts),
                            ..
                        } = val
                        {
                            parts
                                .into_iter()
                                .map(ReturnSuccess::value)
                                .to_output_stream()
                        } else {
                            OutputStream::one(Err(ShellError::labeled_error(
                                "Internal Error",
                                "unexpected result from the split function",
                                span,
                            )))
                        }
                    }
                    Err(e) => OutputStream::one(Err(e)),
                }
            })
            .to_output_stream()
    } else {
        input
            .map(move |v| {
                let mut ret = v;

                for path in args.get_column_paths() {
                    let cloned_args = Arc::clone(&args);
                    ret = ret.swap_data_by_column_path(
                        path,
                        Box::new(move |old| handle_value(&action, &old, span, cloned_args)),
                    )?;
                }

                ReturnSuccess::value(ret)
            })
            .to_output_stream()
    }
}

fn action(path: &Path, tag: Tag, _args: &PathSplitArguments) -> Value {
    let parts: Vec<Value> = path
        .components()
        .map(|comp| {
            let s = comp.as_os_str().to_string_lossy();
            UntaggedValue::string(s).into_value(&tag)
        })
        .collect();

    UntaggedValue::table(&parts).into_value(tag)
}

#[cfg(test)]
mod tests {
    use super::PathSplit;
    use super::ShellError;

    #[test]
    fn examples_work_as_expected() -> Result<(), ShellError> {
        use crate::examples::test as test_examples;

        test_examples(PathSplit {})
    }
}
