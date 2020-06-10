use crate::commands::WholeStreamCommand;
use crate::data::value;
use crate::prelude::*;
use crate::utils::data_processing::map_max;
use nu_errors::ShellError;
use nu_protocol::{Primitive, ReturnSuccess, Signature, SyntaxShape, UntaggedValue, Value};
use nu_source::Tagged;
use num_traits::cast::ToPrimitive;

pub struct MapMaxBy;

#[derive(Deserialize)]
pub struct MapMaxByArgs {
    column_name: Option<Tagged<String>>,
}

#[async_trait]
impl WholeStreamCommand for MapMaxBy {
    fn name(&self) -> &str {
        "map-max-by"
    }

    fn signature(&self) -> Signature {
        Signature::build("map-max-by").named(
            "column_name",
            SyntaxShape::String,
            "the name of the column to map-max the table's rows",
            Some('c'),
        )
    }

    fn usage(&self) -> &str {
        "Creates a new table with the data from the tables rows maxed by the column given."
    }

    async fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        map_max_by(args, registry).await
    }
}

pub async fn map_max_by(
    args: CommandArgs,
    registry: &CommandRegistry,
) -> Result<OutputStream, ShellError> {
    let registry = registry.clone();
    let name = args.call_info.name_tag.clone();
    let (MapMaxByArgs { column_name }, mut input) = args.process(&registry).await?;
    let values: Vec<Value> = input.collect().await;

    if values.is_empty() {
        Err(ShellError::labeled_error(
            "Expected table from pipeline",
            "requires a table input",
            name,
        ))
    } else {
        let map_by_column = if let Some(column_to_map) = column_name {
            Some(column_to_map.item().clone())
        } else {
            None
        };

        match map_max(&values[0], map_by_column, name) {
            Ok(table_maxed) => Ok(OutputStream::one(ReturnSuccess::value(table_maxed))),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MapMaxBy;

    #[test]
    fn examples_work_as_expected() {
        use crate::examples::test as test_examples;

        test_examples(MapMaxBy {})
    }
}
