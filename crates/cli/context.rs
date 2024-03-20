use commands::aws::AWS;
use config::Config;

#[derive(Debug, Clone)]
pub struct GlobalContext {
    pub aws: AWS,
    pub branch: String,
    pub repository: String,
    pub config: Config,
}

impl GlobalContext {
    pub fn new(aws: AWS, config: Config, branch: String, repository: String) -> Self {
        Self {
            aws,
            branch,
            repository,
            config,
        }
    }
}
