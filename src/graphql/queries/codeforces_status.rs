use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

pub async fn fetch_codeforces_stats(
    pool: Arc<PgPool>,
    member_id: i32,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://codeforces.com/api/user.info?handles={}", username);
    let response = reqwest::get(&url).await?;
    let data: Value = response.json().await?;

    if let Some(user) = data["result"].as_array().and_then(|arr| arr.first()) {
        let rating = user["rating"].as_i64().unwrap_or(0) as i32;
        let max_rating = user["maxRating"].as_i64().unwrap_or(0) as i32;
        let rank = user["rank"].as_str().unwrap_or("Unrated").to_string();
        let max_rank = user["maxRank"].as_str().unwrap_or("Unrated").to_string();

        let update_result = sqlx::query(
            "
            INSERT INTO codeforces_stats (
                member_id, codeforces_username, rating, max_rating, rank, max_rank
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (member_id) DO UPDATE SET
                codeforces_username = EXCLUDED.codeforces_username,
                rating = EXCLUDED.rating,
                max_rating = EXCLUDED.max_rating,
                rank = EXCLUDED.rank,
                max_rank = EXCLUDED.max_rank
            "
        )
        .bind(member_id)
        .bind(username)
        .bind(rating)
        .bind(max_rating)
        .bind(rank)
        .bind(max_rank)
        .execute(pool.as_ref())
        .await;

        match update_result {
            Ok(_) => println!("Codeforces stats updated for member ID: {}", member_id),
            Err(e) => eprintln!(
                "Failed to update Codeforces stats for member ID {}: {:?}",
                member_id, e
            ),
        }
    }

    Ok(())
}
