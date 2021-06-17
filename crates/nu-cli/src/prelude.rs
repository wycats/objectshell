#[macro_export]
macro_rules! return_err {
    ($expr:expr) => {
        match $expr {
            Err(_) => return,
            Ok(expr) => expr,
        };
    };
}

#[macro_export]
macro_rules! trace_out_stream {
    (target: $target:tt, $desc:tt = $expr:expr) => {{
        if log::log_enabled!(target: $target, log::Level::Trace) {
            let objects = $expr.inspect(move |o| {
                trace!(
                    target: $target,
                    "{} = {}",
                    $desc,
                    match o {
                        Err(err) => format!("{:?}", err),
                        Ok(value) => value.display(),
                    }
                );
            });

            nu_stream::OutputStream::new(objects)
        } else {
            $expr
        }
    }};
}

pub(crate) use nu_engine::Host;
#[allow(unused_imports)]
pub(crate) use nu_errors::ShellError;
#[allow(unused_imports)]
pub(crate) use nu_protocol::outln;
#[allow(unused_imports)]
pub(crate) use nu_value_ext::ValueExt;
#[allow(unused_imports)]
pub(crate) use std::sync::atomic::Ordering;
