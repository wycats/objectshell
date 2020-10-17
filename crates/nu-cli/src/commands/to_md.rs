use crate::commands::WholeStreamCommand;
use crate::prelude::*;
use futures::StreamExt;
use nu_data::value::format_leaf;
use nu_errors::ShellError;
use nu_protocol::{ReturnSuccess, Signature, UntaggedValue, Value};

pub struct ToMarkdown;

#[derive(Deserialize)]
pub struct ToMarkdownArgs {
    pretty: bool,
}

#[async_trait]
impl WholeStreamCommand for ToMarkdown {
    fn name(&self) -> &str {
        "to md"
    }

    fn signature(&self) -> Signature {
        Signature::build("to md").switch(
            "pretty",
            "Formats the Markdown table to vertically align items",
            Some('p'),
        )
    }

    fn usage(&self) -> &str {
        "Convert table into simple Markdown"
    }

    async fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        to_md(args, registry).await
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Outputs an unformatted md string representing the contents of ls",
                example: "ls | to md",
                result: None,
            },
            Example {
                description: "Outputs a formatted md string representing the contents of ls",
                example: "ls | to md -p",
                result: None,
            },
        ]
    }
}

async fn to_md(args: CommandArgs, registry: &CommandRegistry) -> Result<OutputStream, ShellError> {
    let registry = registry.clone();
    let name_tag = args.call_info.name_tag.clone();
    let (ToMarkdownArgs { pretty }, input) = args.process(&registry).await?;
    let input: Vec<Value> = input.collect().await;
    let headers = nu_protocol::merge_descriptors(&input);

    let mut escaped_headers: Vec<String> = Vec::new();
    let mut column_width_vector: Vec<usize> = Vec::new();

    if !headers.is_empty() && (headers.len() > 1 || headers[0] != "") {
        for header in &headers {
            let escaped_header_string = htmlescape::encode_minimal(&header);
            column_width_vector.push(escaped_header_string.len());
            escaped_headers.push(escaped_header_string);
        }
    }

    let mut escaped_rows: Vec<Vec<String>> = Vec::new();

    for row in &input {
        if let UntaggedValue::Row(row) = row.value.clone() {
            let mut escaped_row_vec: Vec<String> = Vec::new();

            for i in 0..headers.len() {
                let data = row.get_data(&headers[i]);
                let value_string = format_leaf(data.borrow()).plain_string(100_000);
                let new_column_width = value_string.len();
                escaped_row_vec.push(value_string);

                if column_width_vector[i] < new_column_width {
                    column_width_vector[i] = new_column_width;
                }
            }

            escaped_rows.push(escaped_row_vec);
        }
    }

    let output_string = get_output_string(
        &escaped_headers,
        &escaped_rows,
        &column_width_vector,
        pretty,
    );

    Ok(OutputStream::one(ReturnSuccess::value(
        UntaggedValue::string(output_string).into_value(name_tag),
    )))
}

fn get_output_string(
    headers: &Vec<String>,
    rows: &Vec<Vec<String>>,
    column_width_vector: &Vec<usize>,
    pretty: bool,
) -> String {
    let mut output_string = String::new();

    if !headers.is_empty() {
        output_string.push_str("|");

        for i in 0..headers.len() {
            if pretty {
                output_string.push_str(" ");
                output_string.push_str(&get_padded_string(
                    headers[i].clone(),
                    column_width_vector[i],
                    ' ',
                ));
                output_string.push_str(" ");
            } else {
                output_string.push_str(headers[i].as_str());
            }

            output_string.push_str("|");
        }

        output_string.push_str("\n|");

        #[allow(clippy::needless_range_loop)]
        for i in 0..headers.len() {
            if pretty {
                output_string.push_str(" ");
                output_string.push_str(&get_padded_string(
                    String::from("-"),
                    column_width_vector[i],
                    '-',
                ));
                output_string.push_str(" ");
            } else {
                output_string.push_str("-");
            }

            output_string.push_str("|");
        }

        output_string.push_str("\n");
    }

    for row in rows {
        output_string.push_str("|");

        for i in 0..headers.len() {
            if pretty {
                output_string.push_str(" ");
                output_string.push_str(&get_padded_string(
                    row[i].clone(),
                    column_width_vector[i],
                    ' ',
                ));
                output_string.push_str(" ");
            } else {
                output_string.push_str(row[i].as_str());
            }

            output_string.push_str("|");
        }

        output_string.push_str("\n");
    }

    output_string
}

fn get_padded_string(text: String, desired_length: usize, padding_character: char) -> String {
    format!(
        "{}{}",
        text,
        padding_character
            .to_string()
            .repeat(desired_length - text.len())
    )
}

#[cfg(test)]
mod tests {
    use super::ShellError;
    use super::ToMarkdown;

    #[test]
    fn examples_work_as_expected() -> Result<(), ShellError> {
        use crate::examples::test as test_examples;

        Ok(test_examples(ToMarkdown {})?)
    }
}
