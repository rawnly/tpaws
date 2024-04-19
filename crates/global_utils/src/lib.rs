#[macro_export]
macro_rules! print_dbg {
    ( $( $x:expr ),* ) => {
        $(
            if
                cfg!(debug_assertions)
                || std::env::var("TPAWS_DEBUG")
                    .ok()
                    .is_some()
            {
                dbg!($x);
            }
        )*
    };
}

#[macro_export]
macro_rules! is_debug {
    () => {
        cfg!(debug_assertions) || std::env::var("TPAWS_DEBUG").ok().is_some()
    };
}
