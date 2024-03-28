use config::Config;

#[derive(Debug, Clone)]
pub struct GlobalContext {
    pub profile: String,
    pub region: String,
    pub branch: String,
    pub repository: String,
    pub config: Config,
}

impl GlobalContext {
    pub fn new(
        profile: String,
        region: String,
        config: Config,
        branch: String,
        repository: String,
    ) -> Self {
        Self {
            profile,
            region,
            branch,
            repository,
            config,
        }
    }
}
