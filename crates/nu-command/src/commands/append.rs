use crate::prelude::*;
use nu_engine::WholeStreamCommand;
use nu_errors::ShellError;
use nu_protocol::{ReturnSuccess, Signature, SyntaxShape, UntaggedValue, Value};

#[derive(Deserialize)]
struct Arguments {
    value: Value,
}

pub struct Command;

impl WholeStreamCommand for Command {
    fn name(&self) -> &str {
        "append"
    }

    fn signature(&self) -> Signature {
        Signature::build("append").required(
            "row value",
            SyntaxShape::Any,
            "the value of the row to append to the table",
        )
    }

    fn usage(&self) -> &str {
        "Append a row to the table."
    }

    fn run_with_actions(&self, args: CommandArgs) -> Result<ActionStream, ShellError> {
        let (Arguments { mut value }, input) = args.process()?;

        let input: Vec<Value> = input.collect();

        if let Some(first) = input.get(0) {
            value.tag = first.tag();
        }

        // Checks if we are trying to append a row literal
        if let Value {
            value: UntaggedValue::Table(values),
            tag,
        } = &value
        {
            if values.len() == 1 && values[0].is_row() {
                value = values[0].value.clone().into_value(tag);
            }
        }

        Ok(input
            .into_iter()
            .chain(vec![value])
            .map(ReturnSuccess::value)
            .to_output_stream_with_actions())
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Add values to the end of the table",
                example: "echo [1 2 3] | append 4",
                result: Some(vec![
                    UntaggedValue::int(1).into(),
                    UntaggedValue::int(2).into(),
                    UntaggedValue::int(3).into(),
                    UntaggedValue::int(4).into(),
                ]),
            },
            Example {
                description: "Add row value to the end of the table",
                example: "echo [[country]; [Ecuador] ['New Zealand']] | append [[country]; [USA]]",
                result: Some(vec![
                    row! { "country".into() => Value::from("Ecuador")},
                    row! { "country".into() => Value::from("New Zealand")},
                    row! { "country".into() => Value::from("USA")},
                ]),
            },
        ]
    }
}
