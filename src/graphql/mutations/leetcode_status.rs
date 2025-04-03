use async_graphql::{Context, Object, Result};
use reqwest::Client;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Default)]
pub struct FetchLeetCode;

#[Object]
impl FetchLeetCode {
    pub async fn fetch_leetcode_stats(
        &self,
        ctx: &Context<'_>,
        member_id: i32,
        username: String,
    ) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let client = Client::new();
        let url = "https://leetcode.com/graphql";
        let query = r#"
            query userProfile($username: String!) {
                userContestRanking(username: $username) {
                    attendedContestsCount
                }
                matchedUser(username: $username) {
                    profile {
                        ranking
                    }
                    submitStats {
                        acSubmissionNum {
                            difficulty
                            count
                        }
                    }
                }
            }
        "#;

        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "query": query,
                "variables": { "username": username }
            }))
            .send()
            .await
            .map_err(|e| async_graphql::Error::new(format!("Request error: {:?}", e)))?;

        let data: Value = response
            .json()
            .await
            .map_err(|e| async_graphql::Error::new(format!("JSON parsing error: {:?}", e)))?;

        let empty_vec = vec![]; 
        let submissions = data["data"]["matchedUser"]["submitStats"]["acSubmissionNum"]
            .as_array()
            .unwrap_or(&empty_vec);
            

        let mut problems_solved = 0;
        let mut easy_solved = 0;
        let mut medium_solved = 0;
        let mut hard_solved = 0;

        for stat in submissions {
            let count = stat["count"].as_i64().unwrap_or(0) as i32;
            match stat["difficulty"].as_str().unwrap_or("") {
                "Easy" => easy_solved = count,
                "Medium" => medium_solved = count,
                "Hard" => hard_solved = count,
                "All" => problems_solved = count,
                _ => {}
            }
        }

        let contests_participated = data["data"]["userContestRanking"]["attendedContestsCount"]
            .as_i64()
            .unwrap_or(0) as i32;
        let rank = data["data"]["matchedUser"]["profile"]["ranking"]
            .as_i64()
            .unwrap_or(0) as i32;

        sqlx::query!(
            r#"
            INSERT INTO leetcode_stats (
                member_id, leetcode_username, problems_solved, easy_solved, medium_solved,
                hard_solved, contests_participated, best_rank, total_contests
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (member_id) DO UPDATE SET
                leetcode_username = EXCLUDED.leetcode_username,
                problems_solved = EXCLUDED.problems_solved,
                easy_solved = EXCLUDED.easy_solved,
                medium_solved = EXCLUDED.medium_solved,
                hard_solved = EXCLUDED.hard_solved,
                contests_participated = EXCLUDED.contests_participated,
                best_rank = EXCLUDED.best_rank,
                total_contests = EXCLUDED.total_contests
            "#,
            member_id,
            username,
            problems_solved,
            easy_solved,
            medium_solved,
            hard_solved,
            contests_participated,
            rank,
            contests_participated
        )
        .execute(pool.as_ref())
        .await
        .map_err(|e| async_graphql::Error::new(format!("Database update error: {:?}", e)))?;

        Ok(true)
    }
}
