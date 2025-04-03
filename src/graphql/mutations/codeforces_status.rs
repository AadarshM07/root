use async_graphql::{Context, Object, Result};
use reqwest;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Default)]
pub struct FetchCodeForces;

#[Object]
impl FetchCodeForces {
    pub async fn fetch_codeforces_stats(
        &self,
        ctx: &Context<'_>,  // Retrieve the database pool from context
        member_id: i32,
        username: String,
    ) -> Result<String> {
        let pool = ctx.data::<Arc<PgPool>>()?;  // Get the PgPool from context

        let url = format!("https://codeforces.com/api/user.rating?handle={}", username);
        let response = reqwest::get(&url).await?.text().await?;
        let data: Value = serde_json::from_str(&response)?;

        if data["status"] == "OK" {
            if let Some(results) = data["result"].as_array() {
                let contests_participated = results.len() as i32;

                let mut max_rating = 0;
                let mut codeforces_rating = 0;

                for contest in results {
                    if let Some(new_rating) = contest["newRating"].as_i64() {
                        codeforces_rating = new_rating as i32;
                        max_rating = max_rating.max(codeforces_rating);
                    }
                }

                let update_result = sqlx::query!(
                    r#"
                    INSERT INTO codeforces_stats (
                        member_id, codeforces_handle, codeforces_rating, max_rating, contests_participated
                    )
                    VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT (member_id) DO UPDATE SET
                        codeforces_handle = EXCLUDED.codeforces_handle,
                        codeforces_rating = EXCLUDED.codeforces_rating,
                        max_rating = EXCLUDED.max_rating,
                        contests_participated = EXCLUDED.contests_participated
                    "#,
                    member_id,
                    username,
                    codeforces_rating,
                    max_rating,
                    contests_participated
                )
                .execute(pool.as_ref())
                .await;

                match update_result {
                    Ok(_) => Ok(format!(
                        "Codeforces stats updated successfully for member ID: {}",
                        member_id
                    )),
                    Err(e) => Err(format!(
                        "Failed to update Codeforces stats for member ID {}: {:?}",
                        member_id, e
                    )
                    .into()),
                }
            } else {
                Err("Invalid response from Codeforces API".into())
            }
        } else {
            Err("Codeforces API returned an error".into())
        }
    }
}
