#[macro_export]
macro_rules! possibly {
    ($expression:expr, $pattern:pat $(if $guard:expr)? => $returned:expr) => {
        match $expression {
            $pattern $(if $guard)? => Some($returned),
            _ => None
        }
    };
}
