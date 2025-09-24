use reqwest;
use reqwest::Client;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;

pub async fn fetch_and_update_codeforces_stats(
    pool: Arc<PgPool>,
    member_id: i32,
    username: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("https://codeforces.com/api/user.rating?handle={username}");
    let response = reqwest::get(&url).await?.text().await?;
    let data: Value = serde_json::from_str(&response)?;

    if data["status"] == "OK" {
        if let Some(results) = data["result"].as_array() {
            let contests_participated = results.len() as i32;

            // Calculate the user's current and max ratings
            let mut max_rating = 0;
            let mut codeforces_rating = 0;

            for contest in results {
                if let Some(new_rating) = contest["newRating"].as_i64() {
                    codeforces_rating = new_rating as i32;
                    max_rating = max_rating.max(codeforces_rating);
                }
            }

            let update_result = sqlx::query(
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
            )
            .bind(member_id)
            .bind(username)
            .bind(codeforces_rating)
            .bind(max_rating)
            .bind(contests_participated)
            .execute(pool.as_ref())
            .await;

            match update_result {
                Ok(_) => println!("Codeforces stats updated for member ID: {member_id}"),
                Err(e) => {
                    eprintln!("Failed to update Codeforces stats for member ID {member_id}: {e:?}")
                }
            }

            return Ok(());
        }
    }

    Err(format!("Failed to fetch stats for Codeforces handle: {username}").into())
}

pub async fn update_leaderboard_scores(pool: Arc<PgPool>) -> Result<(), sqlx::Error> {
    let leetcode_stats = sqlx::query!(
        "SELECT member_id, problems_solved, easy_solved, medium_solved, hard_solved, 
                contests_participated, best_rank
         FROM leetcode_stats"
    )
    .fetch_all(pool.as_ref())
    .await?;

    let codeforces_stats = sqlx::query!(
        "SELECT member_id, codeforces_rating, max_rating, contests_participated
         FROM codeforces_stats"
    )
    .fetch_all(pool.as_ref())
    .await?;

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
            leetcode_score,
            codeforces_score,
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

        let codeforces_score =
            (row.codeforces_rating / 10) + (row.max_rating / 20) + (5 * row.contests_participated);

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
            0,
            codeforces_score,
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

    Ok(())
}

pub async fn fetch_and_update_leetcode(
    pool: Arc<PgPool>,
    member_id: i32,
    username: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        .await?;

    let data: Value = response.json().await?;

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
    .await?;

    Ok(())
}
