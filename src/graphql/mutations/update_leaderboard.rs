use async_graphql::{Context, Object, Result as GqlResult};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct LeaderboardMutation;

#[Object]
impl LeaderboardMutation {
    pub async fn update_leaderboard(&self, ctx: &Context<'_>) -> GqlResult<bool> {
        let pool = ctx
            .data::<Arc<PgPool>>()
            .map_err(|_| async_graphql::Error::new("Failed to access the database pool"))?;

        let leetcode_stats = sqlx::query!(
            "SELECT member_id, problems_solved, easy_solved, medium_solved, hard_solved, 
                    contests_participated, best_rank
             FROM leetcode_stats"
        )
        .fetch_all(pool.as_ref())
        .await
        .map_err(|e| async_graphql::Error::new(format!("Failed to fetch LeetCode stats: {e:?}")))?;

        let codeforces_stats = sqlx::query!(
            "SELECT member_id, codeforces_rating, max_rating, contests_participated
             FROM codeforces_stats"
        )
        .fetch_all(pool.as_ref())
        .await
        .map_err(|e| {
            async_graphql::Error::new(format!("Failed to fetch Codeforces stats: {e:?}"))
        })?;

        let cf_lookup: HashMap<i32, (i32, i32, i32)> = codeforces_stats
            .iter()
            .map(|row| {
                (
                    row.member_id,
                    (
                        row.codeforces_rating,
                        row.max_rating,
                        row.contests_participated,
                    ),
                )
            })
            .collect();

        for row in &leetcode_stats {
            let leetcode_score = (5 * row.easy_solved)
                + (10 * row.medium_solved)
                + (20 * row.hard_solved)
                + (2 * row.contests_participated)
                + (100 - row.best_rank / 10).max(0);

            let (codeforces_score, unified_score) = cf_lookup
                .get(&row.member_id)
                .map(|(rating, max_rating, contests)| {
                    let cf_score = (rating / 10) + (max_rating / 20) + (5 * contests);
                    (cf_score, leetcode_score + cf_score)
                })
                .unwrap_or((0, leetcode_score));

            let result = sqlx::query!(
                "INSERT INTO leaderboard (member_id, leetcode_score, codeforces_score, unified_score, last_updated)
                 VALUES ($1, $2, $3, $4, NOW())
                 ON CONFLICT (member_id) DO UPDATE SET
                     leetcode_score = EXCLUDED.leetcode_score,
                     codeforces_score = EXCLUDED.codeforces_score,
                     unified_score = EXCLUDED.unified_score,
                     last_updated = NOW()",
                row.member_id,
                0,codeforces_score,
                unified_score
            )
            .execute(pool.as_ref())
            .await;

            if let Err(e) = result {
                eprintln!(
                    "Failed to update leaderboard for member ID {}: {:?}",
                    row.member_id, e
                );
            }
        }

        for row in &codeforces_stats {
            if leetcode_stats
                .iter()
                .any(|lc| lc.member_id == row.member_id)
            {
                continue;
            }

            let codeforces_score = (row.codeforces_rating / 10)
                + (row.max_rating / 20)
                + (5 * row.contests_participated);

            let unified_score = codeforces_score;

            let result = sqlx::query!(
                "INSERT INTO leaderboard (member_id, leetcode_score, codeforces_score, unified_score, last_updated)
                 VALUES ($1, $2, $3, $4, NOW())
                 ON CONFLICT (member_id) DO UPDATE SET
                     leetcode_score = EXCLUDED.leetcode_score,
                     codeforces_score = EXCLUDED.codeforces_score,
                     unified_score = EXCLUDED.unified_score,
                     last_updated = NOW()",
                row.member_id,
                0,codeforces_score,
                unified_score
            )
            .execute(pool.as_ref())
            .await;

            if let Err(e) = result {
                eprintln!(
                    "Failed to update leaderboard for Codeforces-only member ID {}: {:?}",
                    row.member_id, e
                );
            }
        }

        Ok(true)
    }
}
