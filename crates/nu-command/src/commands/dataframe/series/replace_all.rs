use crate::{commands::dataframe::utils::parse_polars_error, prelude::*};
use nu_engine::WholeStreamCommand;
use nu_errors::ShellError;
use nu_protocol::{dataframe::NuDataFrame, Signature, SyntaxShape};
use nu_source::Tagged;
use polars::prelude::IntoSeries;

pub struct DataFrame;

impl WholeStreamCommand for DataFrame {
    fn name(&self) -> &str {
        "dataframe replace-all"
    }

    fn usage(&self) -> &str {
        "[Series] Replace all (sub)strings by a regex pattern"
    }

    fn signature(&self) -> Signature {
        Signature::build("dataframe replace")
            .required_named(
                "pattern",
                SyntaxShape::String,
                "Regex pattern to be matched",
                Some('p'),
            )
            .required_named(
                "replace",
                SyntaxShape::String,
                "replacing string",
                Some('r'),
            )
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        command(args)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Replaces string",
            example: "[abac abac abac] | dataframe to-df | dataframe replace-all -p a -r A",
            result: None,
        }]
    }
}

fn command(mut args: CommandArgs) -> Result<OutputStream, ShellError> {
    let tag = args.call_info.name_tag.clone();
    let pattern: Tagged<String> = args.req_named("pattern")?;
    let replace: Tagged<String> = args.req_named("replace")?;

    let (df, df_tag) = NuDataFrame::try_from_stream(&mut args.input, &tag.span)?;

    let series = df.as_series(&df_tag.span)?;
    let chunked = series.utf8().map_err(|e| {
        parse_polars_error::<&str>(
            &e,
            &df_tag.span,
            Some("The replace command can only be used with string columns"),
        )
    })?;

    let mut res = chunked
        .as_ref()
        .replace_all(pattern.as_str(), replace.as_str())
        .map_err(|e| parse_polars_error::<&str>(&e, &tag.span, None))?;

    res.rename(series.name());

    let df = NuDataFrame::try_from_series(vec![res.into_series()], &tag.span)?;
    Ok(OutputStream::one(df.into_value(df_tag)))
}
