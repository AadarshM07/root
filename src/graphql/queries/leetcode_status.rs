use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

pub async fn fetch_leetcode_stats(
    pool: Arc<PgPool>,
    member_id: i32,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
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
                contestBadge {
                    name
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
        .await?;

    let data: Value = response.json().await?;
    let submissions = &data["data"]["matchedUser"]["submitStats"]["acSubmissionNum"];
    let mut problems_solved = 0;
    let mut easy_solved = 0;
    let mut medium_solved = 0;
    let mut hard_solved = 0;

    if let Some(stats) = submissions.as_array() {
        for stat in stats {
            let count = stat["count"].as_i64().unwrap_or(0) as i32;
            match stat["difficulty"].as_str().unwrap_or("") {
                "Easy" => easy_solved = count,
                "Medium" => medium_solved = count,
                "Hard" => hard_solved = count,
                "All" => problems_solved = count,
                _ => {}
            }
        }
    }

    let user_contest_info = &data["data"]["userContestRanking"];
    let contests_participated = user_contest_info["attendedContestsCount"]
        .as_i64()
        .unwrap_or(0) as i32;
    let rank = data["data"]["matchedUser"]["profile"]["ranking"]
        .as_i64()
        .map(|v| v as i32)
        .unwrap_or(0);

    let update_result = sqlx::query(
        "
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
        ",
    )
    .bind(member_id)
    .bind(username)
    .bind(problems_solved)
    .bind(easy_solved)
    .bind(medium_solved)
    .bind(hard_solved)
    .bind(contests_participated)
    .bind(rank)
    .bind(contests_participated)
    .execute(pool.as_ref())
    .await;

    match update_result {
        Ok(_) => println!("LeetCode stats updated for member ID: {}", member_id),
        Err(e) => eprintln!(
            "Failed to update LeetCode stats for member ID {}: {:?}",
            member_id, e
        ),
    }

    Ok(())
}