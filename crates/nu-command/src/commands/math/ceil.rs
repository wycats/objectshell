use crate::commands::math::utils::run_with_numerical_functions_on_stream;
use crate::prelude::*;
use bigdecimal::One;
use nu_engine::WholeStreamCommand;
use nu_errors::ShellError;
use nu_protocol::{Signature, UntaggedValue, Value};

pub struct SubCommand;

impl WholeStreamCommand for SubCommand {
    fn name(&self) -> &str {
        "math ceil"
    }

    fn signature(&self) -> Signature {
        Signature::build("math ceil")
    }

    fn usage(&self) -> &str {
        "Applies the ceil function to a list of numbers"
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        run_with_numerical_functions_on_stream(
            RunnableContext::from_command_args(args),
            ceil_big_int,
            ceil_big_decimal,
            ceil_default,
        )
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Apply the ceil function to a list of numbers",
            example: "echo [1.5 2.3 -3.1] | math ceil",
            result: Some(vec![
                UntaggedValue::int(2).into(),
                UntaggedValue::int(3).into(),
                UntaggedValue::int(-3).into(),
            ]),
        }]
    }
}

fn ceil_big_int(val: BigInt) -> Value {
    UntaggedValue::int(val).into()
}

fn ceil_big_decimal(val: BigDecimal) -> Value {
    let mut maybe_ceiled = val.round(0);
    if maybe_ceiled < val {
        maybe_ceiled += BigDecimal::one();
    }
    let (ceiled, _) = maybe_ceiled.into_bigint_and_exponent();
    UntaggedValue::int(ceiled).into()
}

fn ceil_default(_: UntaggedValue) -> Value {
    UntaggedValue::Error(ShellError::unexpected(
        "Only numerical values are supported",
    ))
    .into()
}

#[cfg(test)]
mod tests {
    use super::ShellError;
    use super::SubCommand;

    #[test]
    fn examples_work_as_expected() -> Result<(), ShellError> {
        use crate::examples::test as test_examples;

        test_examples(SubCommand {})
    }
}
