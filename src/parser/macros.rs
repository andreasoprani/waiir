macro_rules! assert_token {
    ($val:expr, $var:path) => {
        match $val {
            $var { .. } => true,
            _ => panic!("Invalid token"),
        }
    };
}

pub(super) use assert_token;
