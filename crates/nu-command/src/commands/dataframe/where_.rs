use crate::prelude::*;
use nu_engine::{evaluate_baseline_expr, EvaluatedCommandArgs, WholeStreamCommand};
use nu_errors::ShellError;
use nu_protocol::{
    dataframe::{NuDataFrame, PolarsData},
    hir::{CapturedBlock, ClassifiedCommand, Expression, Literal, Operator, SpannedExpression},
    Primitive, Signature, SyntaxShape, UnspannedPathMember, UntaggedValue, Value,
};

use super::utils::parse_polars_error;
use polars::prelude::{ChunkCompare, Series};

pub struct DataFrame;

impl WholeStreamCommand for DataFrame {
    fn name(&self) -> &str {
        "pls where"
    }

    fn signature(&self) -> Signature {
        Signature::build("pls where").required(
            "condition",
            SyntaxShape::RowCondition,
            "the condition that must match",
        )
    }

    fn usage(&self) -> &str {
        "Filter dataframe to match the condition"
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        command(args)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "",
            example: "",
            result: None,
        }]
    }
}

fn command(args: CommandArgs) -> Result<OutputStream, ShellError> {
    let tag = args.call_info.name_tag.clone();
    let args = args.evaluate_once()?;

    let block: CapturedBlock = args.req(0)?;

    let expression = block
        .block
        .block
        .get(0)
        .and_then(|group| {
            group
                .pipelines
                .get(0)
                .and_then(|v| v.list.get(0))
                .and_then(|expr| match &expr {
                    ClassifiedCommand::Expr(expr) => match &expr.as_ref().expr {
                        Expression::Binary(expr) => Some(expr),
                        _ => None,
                    },
                    _ => None,
                })
        })
        .ok_or(ShellError::labeled_error(
            "Expected a condition",
            "expected a condition",
            &tag.span,
        ))?;

    let left_value = match &expression.left.expr {
        Expression::FullColumnPath(p) => p.as_ref().tail.get(0),
        _ => None,
    }
    .ok_or(ShellError::labeled_error(
        "No column name",
        "Not a column name found in left hand side of comparison",
        &expression.left.span,
    ))?;

    let (col_name, col_name_span) = match &left_value.unspanned {
        UnspannedPathMember::String(name) => Ok((name, &left_value.span)),
        _ => Err(ShellError::labeled_error(
            "No column name",
            "Not a string as column name",
            &left_value.span,
        )),
    }?;

    let right_value = evaluate_baseline_expr(&expression.right, &args.args.context)?;
    let right_condition = match &right_value.value {
        UntaggedValue::Primitive(primitive) => Ok(primitive),
        _ => Err(ShellError::labeled_error(
            "Incorrect argument",
            "Expected primitive values",
            &right_value.tag.span,
        )),
    }?;

    filter_dataframe(
        args,
        &col_name,
        &col_name_span,
        &right_condition,
        &expression.op,
    )
}

macro_rules! comparison_arm {
    ($comparison:expr,  $col:expr, $condition:expr, $span:expr) => {
        match $condition {
            Primitive::Int(val) => Ok($comparison($col, *val)),
            Primitive::BigInt(val) => Ok($comparison(
                $col,
                val.to_i64()
                    .expect("Internal error: protocol did not use compatible decimal"),
            )),
            Primitive::Decimal(val) => Ok($comparison(
                $col,
                val.to_f64()
                    .expect("Internal error: protocol did not use compatible decimal"),
            )),
            Primitive::String(val) => {
                let temp: &str = val.as_ref();
                Ok($comparison($col, temp))
            }
            _ => Err(ShellError::labeled_error(
                "Invalid datatype",
                format!(
                    "this operator cannot be used with the selected '{}' datatype",
                    $col.dtype()
                ),
                &$span,
            )),
        }
    };
}

// With the information extracted from the block we can filter the dataframe using
// polars operations
fn filter_dataframe(
    mut args: EvaluatedCommandArgs,
    col_name: &str,
    col_name_span: &Span,
    right_condition: &Primitive,
    operator: &SpannedExpression,
) -> Result<OutputStream, ShellError> {
    let df = args
        .input
        .next()
        .and_then(|value| match value.value {
            UntaggedValue::DataFrame(PolarsData::EagerDataFrame(nu)) => Some(nu),
            _ => None,
        })
        .ok_or(ShellError::labeled_error(
            "Incorrect stream input",
            "Expected dataframe in stream",
            &args.call_info.name_tag.span,
        ))?;

    let col = df
        .as_ref()
        .column(col_name)
        .map_err(|e| parse_polars_error::<&str>(&e, &col_name_span, None))?;

    let op = match &operator.expr {
        Expression::Literal(Literal::Operator(op)) => Ok(op),
        _ => Err(ShellError::labeled_error(
            "Incorrect argument",
            "Expected operator",
            &operator.span,
        )),
    }?;

    let mask = match op {
        Operator::Equal => comparison_arm!(Series::eq, col, right_condition, operator.span),
        Operator::NotEqual => comparison_arm!(Series::neq, col, right_condition, operator.span),
        Operator::LessThan => comparison_arm!(Series::lt, col, right_condition, operator.span),
        Operator::LessThanOrEqual => {
            comparison_arm!(Series::lt_eq, col, right_condition, operator.span)
        }
        Operator::GreaterThan => comparison_arm!(Series::gt, col, right_condition, operator.span),
        Operator::GreaterThanOrEqual => {
            comparison_arm!(Series::gt_eq, col, right_condition, operator.span)
        }
        _ => Err(ShellError::labeled_error(
            "Incorrect operator",
            "Not implemented operator for dataframes filter",
            &operator.span,
        )),
    }?;

    let res = df
        .as_ref()
        .filter(&mask)
        .map_err(|e| parse_polars_error::<&str>(&e, &args.call_info.name_tag.span, None))?;

    let value = Value {
        value: UntaggedValue::DataFrame(PolarsData::EagerDataFrame(NuDataFrame::new(res))),
        tag: args.call_info.name_tag.clone(),
    };

    Ok(OutputStream::one(value))
}
