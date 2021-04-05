use super::{operate, PathSubcommandArguments};
use crate::prelude::*;
use nu_engine::WholeStreamCommand;
use nu_errors::ShellError;
use nu_protocol::{ColumnPath, Signature, SyntaxShape, UntaggedValue, Value};
use std::path::Path;

pub struct PathExists;

#[derive(Deserialize)]
struct PathExistsArguments {
    rest: Vec<ColumnPath>,
}

impl PathSubcommandArguments for PathExistsArguments {
    fn get_column_paths(&self) -> &Vec<ColumnPath> {
        &self.rest
    }
}

impl WholeStreamCommand for PathExists {
    fn name(&self) -> &str {
        "path exists"
    }

    fn signature(&self) -> Signature {
        Signature::build("path exists")
            .rest(SyntaxShape::ColumnPath, "Optionally operate by column path")
    }

    fn usage(&self) -> &str {
        "Checks whether a path exists"
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        let tag = args.call_info.name_tag.clone();
        let (PathExistsArguments { rest }, input) = args.process()?;
        let args = Arc::new(PathExistsArguments { rest });
        operate(input, &action, tag.span, args)
    }

    #[cfg(windows)]
    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Check if file exists",
            example: "echo 'C:\\Users\\joe\\todo.txt' | path exists",
            result: Some(vec![Value::from(UntaggedValue::boolean(false))]),
        }]
    }

    #[cfg(not(windows))]
    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Check if file exists",
            example: "echo '/home/joe/todo.txt' | path exists",
            result: Some(vec![Value::from(UntaggedValue::boolean(false))]),
        }]
    }
}

fn action(path: &Path, _args: &PathExistsArguments) -> UntaggedValue {
    UntaggedValue::boolean(path.exists())
}

#[cfg(test)]
mod tests {
    use super::PathExists;
    use super::ShellError;

    #[test]
    fn examples_work_as_expected() -> Result<(), ShellError> {
        use crate::examples::test as test_examples;

        test_examples(PathExists {})
    }
}
