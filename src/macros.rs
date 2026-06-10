//! Temporary workaround for deprecation of proc-macro-error2. This will be fixed once
//! Diagnostic is stabilized upstream.

macro_rules! abort_call_site {
    ($fmt:tt) => {
        let msg = $fmt;
        let _ = crate::ERROR_REPORTING.with(|o| {
            *o.borrow_mut() = Some(quote_spanned! { proc_macro2::Span::call_site() => compile_error!(#msg) });
        });
        std::panic::panic_any(())
    };
    ($fmt:tt, $($fmt_args:tt),+) => {
        let msg = format!($fmt, $($fmt_args),+);
        let _ = crate::ERROR_REPORTING.with(|o| {
            *o.borrow_mut() = Some(quote_spanned! { proc_macro2::Span::call_site() => compile_error!(#msg) });
        });
        std::panic::panic_any(())
    };
}

macro_rules! abort {
    ($span:expr, $fmt:tt) => {{
        let span = $span;
        let msg = $fmt.to_string();
        let _ = crate::ERROR_REPORTING.with(|o| {
            *o.borrow_mut() = Some(quote_spanned! { span => compile_error!(#msg) });
        });
        std::panic::panic_any(())
    }};
    ($span:expr, $fmt:tt, $($fmt_args:tt),+) => {{
        let span = $span;
        let msg = format!($fmt, $($fmt_args),+);
        let _ = crate::ERROR_REPORTING.with(|o| {
            *o.borrow_mut() = Some(quote_spanned! { span => compile_error!(#msg) });
        });
        std::panic::panic_any(())
    }};
}
