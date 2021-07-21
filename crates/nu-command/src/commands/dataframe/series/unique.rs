use crate::{commands::dataframe::utils::parse_polars_error, prelude::*};
use nu_engine::WholeStreamCommand;
use nu_errors::ShellError;
use nu_protocol::{dataframe::NuDataFrame, Signature};

pub struct DataFrame;

impl WholeStreamCommand for DataFrame {
    fn name(&self) -> &str {
        "dataframe unique"
    }

    fn usage(&self) -> &str {
        "[Series] Returns unique values from a series"
    }

    fn signature(&self) -> Signature {
        Signature::build("dataframe unique")
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        command(args)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Returns unique values from a series",
            example: "[1 2 2 3 3] | dataframe to-df | dataframe unique",
            result: None,
        }]
    }
}

fn command(mut args: CommandArgs) -> Result<OutputStream, ShellError> {
    let tag = args.call_info.name_tag.clone();

    let (df, df_tag) = NuDataFrame::try_from_stream(&mut args.input, &tag.span)?;

    let res = df
        .as_series(&df_tag.span)?
        .unique()
        .map_err(|e| parse_polars_error::<&str>(&e, &tag.span, None))?;

    let df = NuDataFrame::try_from_series(vec![res], &tag.span)?;

    Ok(OutputStream::one(df.into_value(tag)))
}
