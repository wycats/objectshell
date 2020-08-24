use super::deduction::VarDeclaration;
use crate::commands::deduction::VarSyntaxShapeDeductor;
use crate::commands::WholeStreamCommand;
use crate::context::CommandRegistry;
use crate::prelude::*;
use deduction_to_signature::DeductionToSignature;
use log::trace;
use nu_data::config;
use nu_errors::ShellError;
use nu_protocol::{
    hir::Block, CommandAction, ReturnSuccess, Signature, SyntaxShape, UntaggedValue, Value,
};
use nu_source::Tagged;

pub struct Alias;

#[derive(Deserialize)]
pub struct AliasArgs {
    pub name: Tagged<String>,
    pub args: Vec<Value>,
    pub block: Block,
    pub _infer: Option<bool>,
    pub save: Option<bool>,
}

#[async_trait]
impl WholeStreamCommand for Alias {
    fn name(&self) -> &str {
        "alias"
    }

    fn signature(&self) -> Signature {
        Signature::build("alias")
            .required("name", SyntaxShape::String, "the name of the alias")
            .required("args", SyntaxShape::Table, "the arguments to the alias")
            .required(
                "block",
                SyntaxShape::Block,
                "the block to run as the body of the alias",
            )
            .switch("infer", "infer argument types (experimental)", Some('i'))
            .switch("save", "save the alias to your config", Some('s'))
    }

    fn usage(&self) -> &str {
        "Define a shortcut for another command."
    }

    async fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        alias(args, registry).await
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "An alias without parameters",
                example: "alias say-hi [] { echo 'Hello!' }",
                result: None,
            },
            Example {
                description: "An alias with a single parameter",
                example: "alias l [x] { ls $x }",
                result: None,
            },
            Example {
                description: "An alias with an variable amount of parameter",
                example: "alias l [x...] { ls $x }",
                result: None,
            },
            Example {
                description: "An alias with at least 1 parameter",
                example: "alias l [first, x...] { ls $first $x }",
                result: None,
            },
        ]
    }
}

//TODO where to put these 2 funcs?
pub fn var_arg_name(var_name: &str) -> String {
    let mut name = var_name.to_string();
    name.truncate(name.len() - 3);
    name
}
pub fn is_var_arg(var_name: &str) -> bool {
    var_name.ends_with("...")
}

pub async fn alias(
    args: CommandArgs,
    registry: &CommandRegistry,
) -> Result<OutputStream, ShellError> {
    let registry = registry.clone();
    let mut raw_input = args.raw_input.clone();
    let (
        AliasArgs {
            name,
            args: list,
            block,
            _infer,
            save,
        },
        _ctx,
    ) = args.process(&registry).await?;

    if let Some(true) = save {
        let mut result = nu_data::config::read(name.clone().tag, &None)?;

        // process the alias to remove the --save flag
        let left_brace = raw_input.find('{').unwrap_or(0);
        let right_brace = raw_input.rfind('}').unwrap_or_else(|| raw_input.len());
        let left = raw_input[..left_brace]
            .replace("--save", "") // TODO using regex (or reconstruct string from AST?)
            .replace("-si", "-i")
            .replace("-s ", "")
            .replace("-is", "-i");
        let right = raw_input[right_brace..]
            .replace("--save", "")
            .replace("-si", "-i")
            .replace("-s ", "")
            .replace("-is", "-i");
        raw_input = format!("{}{}{}", left, &raw_input[left_brace..right_brace], right);

        // create a value from raw_input alias
        let alias: Value = raw_input.trim().to_string().into();
        let alias_start = raw_input.find('[').unwrap_or(0); // used to check if the same alias already exists

        // add to startup if alias doesn't exist and replace if it does
        match result.get_mut("startup") {
            Some(startup) => {
                if let UntaggedValue::Table(ref mut commands) = startup.value {
                    if let Some(command) = commands.iter_mut().find(|command| {
                        let cmd_str = command.as_string().unwrap_or_default();
                        cmd_str.starts_with(&raw_input[..alias_start])
                    }) {
                        *command = alias;
                    } else {
                        commands.push(alias);
                    }
                }
            }
            None => {
                let table = UntaggedValue::table(&[alias]);
                result.insert("startup".to_string(), table.into_value(Tag::default()));
            }
        }
        config::write(&result, &None)?;
    }

    let mut processed_args: Vec<VarDeclaration> = vec![];
    for (idx, item) in list.iter().enumerate() {
        match item.as_string() {
            Ok(var_name) => {
                let (dollar_var_name, is_var_arg) = {
                    if is_var_arg(&var_name) {
                        //Var args are only allowed in last place
                        if (idx + 1) != list.len() {
                            return Err(ShellError::labeled_error(
                                "Var-args variables are only allowed as the last argument!",
                                "Var-arg",
                                item.tag.span,
                            ));
                        }
                        (format!("${}", var_arg_name(&var_name)), true)
                    } else {
                        (format!("${}", var_name), false)
                    }
                };
                processed_args.push(VarDeclaration {
                    name: dollar_var_name,
                    // type_decl: None,
                    is_var_arg,
                    span: item.tag.span,
                });
            }
            Err(_) => {
                return Err(ShellError::labeled_error(
                    "Expected a string",
                    "expected a string",
                    item.tag(),
                ));
            }
        }
    }
    trace!("Found vars: {:?}", processed_args);

    let inferred_shapes = VarSyntaxShapeDeductor::infer_vars(&processed_args, &block, &registry)?;
    let signature = DeductionToSignature::get(&name.item, &inferred_shapes);

    Ok(OutputStream::one(ReturnSuccess::action(
        CommandAction::AddAlias(signature, block),
    )))
}

#[cfg(test)]
mod tests {
    use super::Alias;

    #[test]
    fn examples_work_as_expected() {
        use crate::examples::test as test_examples;

        test_examples(Alias {})
    }
}

//TODO better naming
mod deduction_to_signature {
    use crate::commands::deduction::{VarDeclaration, VarShapeDeduction};
    use nu_protocol::{PositionalType, Signature, SyntaxShape};
    use nu_source::Span;

    pub struct DeductionToSignature {}
    impl DeductionToSignature {
        pub fn get(
            cmd_name: &str,
            deductions: &[(VarDeclaration, Option<Vec<VarShapeDeduction>>)],
        ) -> Signature {
            let deductions: Vec<(VarDeclaration, VarShapeDeduction)> = deductions
                .iter()
                .map(|(decl, deducs)| {
                    let default = VarShapeDeduction {
                        deduction: SyntaxShape::Any,
                        deducted_from: vec![Span::unknown()],
                        many_of_shapes: false,
                    };
                    let decl = decl.clone();
                    match deducs {
                        Some(deduc) => {
                            //Pick more general shapes over other shapes

                            //Pick any over anything
                            if let Some(any_shape) = deduc
                                .iter()
                                .find(|deduc| deduc.deduction == SyntaxShape::Any)
                            {
                                (decl, any_shape.clone())
                            }
                            //Pick math over other shapes
                            else if let Some(math_shape) = deduc
                                .iter()
                                .find(|deduc| deduc.deduction == SyntaxShape::Math)
                            {
                                (decl, math_shape.clone())
                            } else {
                                //Pick first shape
                                (decl, deduc[0].clone())
                            }
                        }
                        None => (decl, default),
                    }
                })
                .collect();

            let mut sig = Signature::build(cmd_name);
            for (var_decl, shape) in &deductions {
                //TODO pass in better description
                sig.positional.push((
                    PositionalType::mandatory(&var_decl.name, shape.deduction),
                    "".to_string(),
                ));
            }
            if let Some(last_arg) = deductions.last() {
                if last_arg.0.is_var_arg {
                    sig.positional.pop();
                    sig = sig.rest(last_arg.1.deduction, last_arg.0.name.clone());
                }
            }

            sig
        }
    }
}
