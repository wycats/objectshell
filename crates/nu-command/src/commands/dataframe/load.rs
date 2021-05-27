use std::path::PathBuf;

use crate::prelude::*;
use nu_engine::{EvaluatedCommandArgs, WholeStreamCommand};
use nu_errors::ShellError;
use nu_protocol::{
    dataframe::{NuDataFrame, PolarsData},
    Primitive, Signature, SyntaxShape, UntaggedValue, Value,
};

use nu_source::Tagged;
use polars::prelude::{CsvReader, JsonReader, ParquetReader, SerReader};
use std::fs::File;

pub struct DataFrame;

impl WholeStreamCommand for DataFrame {
    fn name(&self) -> &str {
        "pls load"
    }

    fn usage(&self) -> &str {
        "Loads dataframe form csv file"
    }

    fn signature(&self) -> Signature {
        Signature::build("pls load")
            .required(
                "file",
                SyntaxShape::FilePath,
                "the file path to load values from",
            )
            .named(
                "delimiter",
                SyntaxShape::String,
                "file delimiter character. CSV file",
                Some('d'),
            )
            .switch(
                "no_header",
                "Indicates if file doesn't have header. CSV file",
                None,
            )
            .named(
                "infer_schema",
                SyntaxShape::Number,
                "Set number of row to infer the schema of the file. CSV file",
                None,
            )
            .named(
                "skip_rows",
                SyntaxShape::Number,
                "Number of rows to skip from file. CSV file",
                None,
            )
            .named(
                "columns",
                SyntaxShape::Table,
                "Columns to be selected from csv file. CSV file",
                None,
            )
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        command(args)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Takes a file name and creates a dataframe",
            example: "pls load test.csv",
            result: None,
        }]
    }
}

fn command(args: CommandArgs) -> Result<OutputStream, ShellError> {
    let tag = args.call_info.name_tag.clone();
    let args = args.evaluate_once()?;
    let file: Tagged<PathBuf> = args.req(0)?;

    let df = match file.item().extension() {
        Some(e) => match e.to_str() {
            Some("csv") => from_csv(args),
            Some("parquet") => from_parquet(args),
            Some("json") => from_json(args),
            _ => Err(ShellError::labeled_error(
                "Error with file",
                "Not a csv, parquet or json file",
                &file.tag,
            )),
        },
        None => Err(ShellError::labeled_error(
            "Error with file",
            "File without extension",
            &file.tag,
        )),
    }?;

    let file_name = match file.item.into_os_string().into_string() {
        Ok(name) => name,
        Err(e) => {
            return Err(ShellError::labeled_error(
                "Error with file name",
                format!("{:?}", e),
                &file.tag,
            ))
        }
    };

    let init = InputStream::one(
        UntaggedValue::DataFrame(PolarsData::EagerDataFrame(NuDataFrame::new_with_name(
            df, file_name,
        )))
        .into_value(&tag),
    );

    Ok(init.to_output_stream())
}

fn from_parquet(args: EvaluatedCommandArgs) -> Result<polars::prelude::DataFrame, ShellError> {
    let file: Tagged<PathBuf> = args.req(0)?;

    let r = File::open(&file.item)
        .map_err(|e| ShellError::labeled_error("Error with file", format!("{:?}", e), &file.tag))?;

    let reader = ParquetReader::new(r);

    reader
        .finish()
        .map_err(|e| ShellError::labeled_error("Error with file", format!("{:?}", e), &file.tag))
}

fn from_json(args: EvaluatedCommandArgs) -> Result<polars::prelude::DataFrame, ShellError> {
    let file: Tagged<PathBuf> = args.req(0)?;

    let r = File::open(&file.item)
        .map_err(|e| ShellError::labeled_error("Error with file", format!("{:?}", e), &file.tag))?;

    let reader = JsonReader::new(r);

    reader
        .finish()
        .map_err(|e| ShellError::labeled_error("Error with file", format!("{:?}", e), &file.tag))
}

fn from_csv(args: EvaluatedCommandArgs) -> Result<polars::prelude::DataFrame, ShellError> {
    let file: Tagged<PathBuf> = args.req(0)?;
    let delimiter: Option<Tagged<String>> = args.get_flag("delimiter")?;
    let no_header: bool = args.has_flag("no_header");
    let infer_schema: Option<Tagged<usize>> = args.get_flag("infer_schema")?;
    let skip_rows: Option<Tagged<usize>> = args.get_flag("skip_rows")?;
    let columns: Option<Vec<Value>> = args.get_flag("columns")?;

    let csv_reader = CsvReader::from_path(&file.item).map_err(|e| {
        ShellError::labeled_error("Unable to parse file", format!("{}", e), &file.tag)
    })?;

    let csv_reader = match delimiter {
        None => csv_reader,
        Some(d) => {
            if d.item.len() != 1 {
                return Err(ShellError::labeled_error(
                    "Incorrect delimiter",
                    "Delimiter has to be one char",
                    &d.tag,
                ));
            } else {
                let delimiter = match d.item.chars().nth(0) {
                    Some(d) => d as u8,
                    None => unreachable!(),
                };
                csv_reader.with_delimiter(delimiter)
            }
        }
    };

    let csv_reader = if no_header {
        csv_reader.has_header(false)
    } else {
        csv_reader.has_header(true)
    };

    let csv_reader = match infer_schema {
        None => csv_reader.infer_schema(None),
        Some(r) => csv_reader.infer_schema(Some(r.item)),
    };

    let csv_reader = match skip_rows {
        None => csv_reader,
        Some(r) => csv_reader.with_skip_rows(r.item),
    };

    let csv_reader = match columns {
        None => csv_reader,
        Some(c) => {
            let columns = c
                .into_iter()
                .map(|value| match value.value {
                    UntaggedValue::Primitive(Primitive::String(s)) => Ok(s),
                    _ => Err(ShellError::labeled_error(
                        "Incorrect type for column",
                        "Only string as columns",
                        &value.tag,
                    )),
                })
                .collect::<Result<Vec<String>, ShellError>>();

            csv_reader.with_columns(Some(columns?))
        }
    };

    match csv_reader.finish() {
        Ok(csv_reader) => Ok(csv_reader),
        Err(e) => Err(ShellError::labeled_error(
            "Error while parsing dataframe",
            format!("{}", e),
            &file.tag,
        )),
    }
}
