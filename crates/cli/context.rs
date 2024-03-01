use commands::aws::AWS;

#[derive(Debug, Clone)]
pub struct GlobalContext {
    pub aws: AWS,
    pub branch: String,
    pub repository: String,
}

impl GlobalContext {
    pub fn new(aws: AWS, branch: String, repository: String) -> Self {
        Self {
            aws,
            branch,
            repository,
        }
    }
}
