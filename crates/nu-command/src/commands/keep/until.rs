use crate::prelude::*;
use nu_engine::evaluate_baseline_expr;
use nu_engine::WholeStreamCommand;
use nu_errors::ShellError;
use nu_parser::ParserScope;
use nu_protocol::{
    hir::{CapturedBlock, ClassifiedCommand},
    Signature, SyntaxShape,
};

pub struct SubCommand;

impl WholeStreamCommand for SubCommand {
    fn name(&self) -> &str {
        "keep until"
    }

    fn signature(&self) -> Signature {
        Signature::build("keep until")
            .required(
                "condition",
                SyntaxShape::RowCondition,
                "The condition that must be met to stop keeping rows",
            )
            .filter()
    }

    fn usage(&self) -> &str {
        "Keeps rows until the condition matches."
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        let ctx = Arc::new(EvaluationContext::from_args(&args));
        let tag = args.call_info.name_tag.clone();

        let block: CapturedBlock = args.req(0)?;
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

        Ok(args
            .input
            .take_while(move |item| {
                let condition = condition.clone();
                let ctx = ctx.clone();
                ctx.scope.enter_scope();
                ctx.scope.add_vars(&block.captured.entries);
                if let Some((arg, _)) = block.block.params.positional.first() {
                    ctx.scope.add_var(arg.name(), item.clone());
                }

                let result = evaluate_baseline_expr(&*condition, &*ctx);
                ctx.scope.exit_scope();

                !matches!(result, Ok(ref v) if v.is_true())
            })
            .to_output_stream())
    }
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
