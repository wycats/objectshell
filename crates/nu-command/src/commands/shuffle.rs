use crate::prelude::*;
use nu_engine::WholeStreamCommand;
use nu_errors::ShellError;
use nu_protocol::{ReturnSuccess, Value};

use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct Shuffle;

impl WholeStreamCommand for Shuffle {
    fn name(&self) -> &str {
        "shuffle"
    }

    fn usage(&self) -> &str {
        "Shuffle rows randomly."
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        shuffle(args)
    }
}

fn shuffle(args: CommandArgs) -> Result<OutputStream, ShellError> {
    let input = args.input;
    let mut values: Vec<Value> = input.collect();

    values.shuffle(&mut thread_rng());

    Ok((values.into_iter().map(ReturnSuccess::value)).to_output_stream())
}

#[cfg(test)]
mod tests {
    use super::ShellError;
    use super::Shuffle;

    #[test]
    fn examples_work_as_expected() -> Result<(), ShellError> {
        use crate::examples::test as test_examples;

        test_examples(Shuffle {})
    }
}
