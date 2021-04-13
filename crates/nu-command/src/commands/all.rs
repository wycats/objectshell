use crate::prelude::*;
use nu_engine::evaluate_baseline_expr;
use nu_engine::WholeStreamCommand;
use nu_errors::ShellError;
use nu_protocol::{
    hir::CapturedBlock, hir::ClassifiedCommand, Signature, SyntaxShape, UntaggedValue,
};
use nu_stream::ToActionStream;

pub struct Command;

#[derive(Deserialize)]
pub struct Arguments {
    block: CapturedBlock,
}

impl WholeStreamCommand for Command {
    fn name(&self) -> &str {
        "all?"
    }

    fn signature(&self) -> Signature {
        Signature::build("all?").required(
            "condition",
            SyntaxShape::RowCondition,
            "the condition that must match",
        )
    }

    fn usage(&self) -> &str {
        "Find if the table rows matches the condition."
    }

    fn run_with_actions(&self, args: CommandArgs) -> Result<ActionStream, ShellError> {
        all(args)
    }

    fn examples(&self) -> Vec<Example> {
        use nu_protocol::Value;

        vec![
            Example {
                description: "Find if services are running",
                example: "echo [[status]; [UP] [UP]] | all? status == UP",
                result: Some(vec![Value::from(true)]),
            },
            Example {
                description: "Check that all values are even",
                example: "echo [2 4 6 8] | all? $(= $it mod 2) == 0",
                result: Some(vec![Value::from(true)]),
            },
        ]
    }
}

fn all(args: CommandArgs) -> Result<ActionStream, ShellError> {
    let ctx = Arc::new(EvaluationContext::from_args(&args));
    let tag = args.call_info.name_tag.clone();
    let (Arguments { block }, input) = args.process()?;

    let condition = {
        if block.block.block.len() != 1 {
            return Err(ShellError::labeled_error(
                "Expected a condition",
                "expected a condition",
                tag,
            ));
        }
        match block.block.block[0].pipelines.get(0) {
            Some(item) => match item.list.get(0) {
                Some(ClassifiedCommand::Expr(expr)) => expr.clone(),
                _ => {
                    return Err(ShellError::labeled_error(
                        "Expected a condition",
                        "expected a condition",
                        tag,
                    ));
                }
            },
            None => {
                return Err(ShellError::labeled_error(
                    "Expected a condition",
                    "expected a condition",
                    tag,
                ));
            }
        }
    };

    let init = Ok(InputStream::one(
        UntaggedValue::boolean(true).into_value(&tag),
    ));

    Ok(input
        .fold(init, move |acc, row| {
            let condition = condition.clone();
            let ctx = ctx.clone();
            ctx.scope.enter_scope();
            ctx.scope.add_vars(&block.captured.entries);
            ctx.scope.add_var("$it", row);

            let condition = evaluate_baseline_expr(&condition, &*ctx);
            ctx.scope.exit_scope();

            let curr = acc?.drain_vec();
            let curr = curr
                .get(0)
                .ok_or_else(|| ShellError::unexpected("No value to check with"))?;
            let cond = curr.as_bool()?;

            match condition {
                Ok(condition) => match condition.as_bool() {
                    Ok(b) => Ok(InputStream::one(
                        UntaggedValue::boolean(cond && b).into_value(&curr.tag),
                    )),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        })?
        .to_action_stream())
}

#[cfg(test)]
mod tests {
    use super::Command;
    use super::ShellError;

    #[test]
    fn examples_work_as_expected() -> Result<(), ShellError> {
        use crate::examples::test as test_examples;

        test_examples(Command {})
    }
}
