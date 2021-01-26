use log::debug;
use std::marker;

use crate::{lex::Token, parse::util::token_to_spanned_string};
use nu_errors::ParseError;
use nu_source::Span;

pub(crate) trait CheckedParse: Parse {}

pub(crate) trait Parse {
    type Output;
    fn parse(tokens: &[Token], i: usize) -> (Self::Output, usize, Option<ParseError>);

    fn parse_debug(tokens: &[Token], i: usize) -> (Self::Output, usize, Option<ParseError>) {
        let tokens_str = if i < tokens.len() {
            format!(
                "{:?}",
                &tokens[i..]
                    .iter()
                    .map(|t| t.contents.clone())
                    .collect::<Vec<_>>()
            )
        } else {
            "[]".to_owned()
        };
        debug!(
            r#"Parsing: {:?}
            Tokens: {:?}"#,
            Self::display_name(),
            tokens_str
        );

        Self::parse(tokens, i)
    }

    fn display_name() -> String;
    fn default_error_value() -> Self::Output;

    fn mismatch_error(token: &Token) -> Option<ParseError> {
        Some(ParseError::mismatch(
            Self::display_name(),
            token_to_spanned_string(token),
        ))
    }

    fn mismatch_default_return(
        token: &Token,
        i: usize,
    ) -> (Self::Output, usize, Option<ParseError>) {
        (Self::default_error_value(), i, Self::mismatch_error(token))
    }
}

// #[macro_export]
// macro_rules! parse_struct {
//     // (cwd: $cwd:expr, $path:expr, $($part:expr),*) => {{
//     (name: $name:ident, $($x:ident),*) => {
//             struct $name <
//             $(
//                 $x,
//             )*
//                 > {
//                 $(
//                     $x: marker::PhantomData<$x>,
//                 )*
//         }
//     };
// }

// parse_struct!(name: Test, A, B);

pub(crate) struct Expect<Value> {
    _marker: marker::PhantomData<*const Value>,
}

//Expect is always checked
impl<T: Parse> CheckedParse for Expect<T> {}

impl<Value: Parse> Parse for Expect<Value> {
    type Output = Value::Output;

    fn parse(tokens: &[Token], i: usize) -> (Self::Output, usize, Option<ParseError>) {
        if i < tokens.len() {
            debug!(
                "Expect<{:?}> {:?} {:?}",
                Value::display_name(),
                &tokens[i..],
                i
            );
            //Okay let underlying value parse tokens
            Value::parse_debug(tokens, i)
        } else {
            debug!("Expect<{:?}> but no tokens", Value::display_name(),);
            //No tokens are present --> Error out
            let last_span = if let Some(last_token) = tokens.last() {
                last_token.span
            } else {
                Span::unknown()
            };

            (
                Value::default_error_value(),
                i,
                Some(ParseError::unexpected_eof(Value::display_name(), last_span)),
            )
        }
    }

    fn display_name() -> String {
        Value::display_name()
    }

    fn default_error_value() -> Value::Output {
        Value::default_error_value()
    }
}

pub(crate) struct Maybe<Value> {
    _marker: marker::PhantomData<*const Value>,
}

//Always Checked because accepts only checked
impl<Value: CheckedParse> CheckedParse for Maybe<Value> {}

impl<Value: CheckedParse> Parse for Maybe<Value> {
    type Output = Option<Value::Output>;

    fn parse(tokens: &[Token], i: usize) -> (Self::Output, usize, Option<ParseError>) {
        debug!("Parsing Maybe<{:?}>", Value::display_name());
        let (v, new_i, error) = Value::parse_debug(tokens, i);
        if error.is_some() {
            debug!("Maybe<{:?}> not present", Value::display_name());
            (None, i, None)
        } else {
            debug!("Maybe<{:?}> is present", Value::display_name());
            (Some(v), new_i, error)
        }
    }

    fn display_name() -> String {
        Value::display_name() + "?"
    }

    fn default_error_value() -> Self::Output {
        Some(Value::default_error_value())
    }
}

///Parse First and then Second
pub(crate) struct AndThen<First, Second> {
    _marker1: marker::PhantomData<*const First>,
    _marker2: marker::PhantomData<*const Second>,
}

impl<First: CheckedParse, Second: CheckedParse> Parse for AndThen<First, Second> {
    type Output = (First::Output, Second::Output);

    fn parse(tokens: &[Token], i: usize) -> (Self::Output, usize, Option<ParseError>) {
        let (first, i, err_first) = First::parse(tokens, i);
        let (second, i, err_second) = Second::parse(tokens, i);
        ((first, second), i, err_first.or(err_second))
    }

    fn display_name() -> String {
        First::display_name() + " >> " + &Second::display_name()
    }

    fn default_error_value() -> Self::Output {
        (First::default_error_value(), Second::default_error_value())
    }
}

//Always Checked because accepts only checked
impl<T1: CheckedParse, T2: CheckedParse> CheckedParse for AndThen<T1, T2> {}

pub(crate) struct IfSuccessThen<Maybe, AndThen> {
    _marker1: marker::PhantomData<*const Maybe>,
    _marker2: marker::PhantomData<*const AndThen>,
}

//Always Checked because accepts only checked
impl<Try: CheckedParse, AndThen: CheckedParse> CheckedParse for IfSuccessThen<Try, AndThen> {}

impl<Try: CheckedParse, AndThen: CheckedParse> Parse for IfSuccessThen<Try, AndThen> {
    type Output = Option<(Try::Output, AndThen::Output)>;

    fn parse(tokens: &[Token], i: usize) -> (Self::Output, usize, Option<ParseError>) {
        let (try_, new_i, err_try) = Maybe::<Try>::parse(tokens, i);
        if let Some(try_v) = try_ {
            //Succeeded at parsing Maybe. Now AndThen has to follow
            let (and_then, new_i, err_second) = Expect::<AndThen>::parse(tokens, new_i);
            (Some((try_v, and_then)), new_i, err_try.or(err_second))
        } else {
            //Okay Couldn't parse Try
            (None, i, None)
        }
    }

    fn display_name() -> String {
        "(".to_string() + &Try::display_name() + " >> " + &AndThen::display_name() + ")?"
    }

    fn default_error_value() -> Self::Output {
        Some((Try::default_error_value(), AndThen::default_error_value()))
    }
}

pub(crate) struct Discard<Value> {
    _marker: marker::PhantomData<*const Value>,
}

//Always Checked because accepts only checked
impl<Value: CheckedParse> CheckedParse for Discard<Value> {}

impl<Value: CheckedParse> Parse for Discard<Value> {
    type Output = ();

    fn parse(tokens: &[Token], i: usize) -> (Self::Output, usize, Option<ParseError>) {
        let (_, i, err) = Value::parse(tokens, i);
        ((), i, err)
    }

    fn display_name() -> String {
        Value::display_name()
    }

    fn default_error_value() -> Self::Output {
        ()
    }
}
