use crate::prelude::*;
use nu_engine::{script, WholeStreamCommand};

use nu_errors::ShellError;
use nu_protocol::{Signature, SyntaxShape};
use nu_source::Tagged;

pub struct Alias;

impl WholeStreamCommand for Alias {
    fn name(&self) -> &str {
        "alias"
    }

    fn signature(&self) -> Signature {
        Signature::build("alias")
            .required("name", SyntaxShape::String, "the name of the alias")
            .required("equals", SyntaxShape::String, "the equals sign")
            .rest(SyntaxShape::Any, "the expansion for the alias")
    }

    fn usage(&self) -> &str {
        "Alias a command to an expansion."
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        alias(args)
    }

    fn examples(&self) -> Vec<Example> {
        vec![]
    }
}

pub fn alias(args: CommandArgs) -> Result<OutputStream, ShellError> {
    Ok(OutputStream::empty())
}
