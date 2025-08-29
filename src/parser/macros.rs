macro_rules! assert_token {
    ($val:expr, $($pat:pat_param)|+) => {
        // The `matches!` macro is a clean way to check if an expression
        // matches any of a series of patterns. This works for all enum
        // variants (unit, tuple, or struct) and is more robust than the
        // previous implementation.
        if !matches!($val, $($pat)|+) {
            // Using `bail!` from `anyhow` allows us to propagate the error
            // gracefully, as parser functions return a `Result`.
            ::anyhow::bail!(
                "Invalid token. Got: {:?}, Expected one of: {}",
                $val,
                stringify!($($pat)|+)
            );
        }
    };
}

pub(super) use assert_token;
