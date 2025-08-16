mod json;

pub use json::Json;

use crate::authentication::identity::Identity;

pub(crate) trait CustomValidateTrait {
    async fn validate(&self, identity: &Identity) -> Result<(), String>;
}

pub(crate) mod source {
    use std::collections::HashSet;

    use crate::authentication::identity::Identity;

    pub(crate) async fn validate_many(
        identity: &Identity,
        sources: &HashSet<String>,
    ) -> Result<(), String> {
        Ok(())
    }
}
