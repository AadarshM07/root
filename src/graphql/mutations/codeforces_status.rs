use async_graphql::{Context, Object, Result};
use sqlx::PgPool;
use std::sync::Arc;
use super::leaderboard_api::fetch_and_update_codeforces_stats;

#[derive(Default)]
pub struct FetchCodeForces;

#[Object]
impl FetchCodeForces {
    pub async fn fetch_codeforces_stats(
        &self,
        ctx: &Context<'_>,
        member_id: i32,
        username: String,
    ) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        fetch_and_update_codeforces_stats(pool.clone(), member_id, &username).await?;
        Ok(true)
    }
}
