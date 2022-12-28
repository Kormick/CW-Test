use async_trait::async_trait;

use crate::statement::*;

pub const RETRIES: usize = 3;

#[async_trait]
pub trait Solution {
    async fn solve(repositories: Vec<ServerName>) -> Option<Binary>;
}
