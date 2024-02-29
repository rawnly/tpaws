pub mod aws;
pub mod git;

#[macro_export]
macro_rules! is_installed {
    ($command:expr) => {
        std::process::Command::new($command)
            .stderr(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .spawn()
            .is_ok()
    };
}

#[macro_export]
macro_rules! command {
    ($command:expr) => (tokio::process::Command::new($command));

    ($command:expr, $($x:expr),+) => {{
        let mut args;

        #[allow(clippy::vec_init_then_push)]
        {
            args = Vec::new();

            $(
                args.push($x);
            )*
        }

        if cfg!(debug_assertions) {
            dbg!(&args);
        }

        tokio::process::Command::new($command)
        .args(&args.clone())
    }};
}

#[macro_export]
macro_rules! spawn_command {
    ($command:expr) => (tokio::process::Command::new($command).spawn());
    ($command:expr, $($args:expr),*) => {{
        let mut args;

        #[allow(clippy::vec_init_then_push)]
        {
            args = Vec::new();
            $( args.push($args); )*
            if cfg!(debug_assertions) {
                dbg!(&args);
            }
        }

        tokio::process::Command::new($command).args(&args.clone()).spawn()
    }}
}
