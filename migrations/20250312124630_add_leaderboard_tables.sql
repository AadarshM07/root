-- Add migration script here

CREATE TABLE IF NOT EXISTS leaderboard (
    id SERIAL PRIMARY KEY,
    member_id INT UNIQUE NOT NULL,
    leetcode_score INT,
    codeforces_score INT,
    unified_score INT NOT NULL,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (member_id) REFERENCES member(member_id)
);

CREATE TABLE IF NOT EXISTS leetcode_stats (
    id SERIAL PRIMARY KEY,
    member_id INT NOT NULL,
    leetcode_username VARCHAR(255) NOT NULL,
    problems_solved INT NOT NULL,
    easy_solved INT NOT NULL,
    medium_solved INT NOT NULL,
    hard_solved INT NOT NULL,
    contests_participated INT NOT NULL,
    best_rank INT NOT NULL,
    total_contests INT NOT NULL,
    FOREIGN KEY (member_id) REFERENCES member(member_id)
);

CREATE TABLE IF NOT EXISTS codeforces_stats (
    id SERIAL PRIMARY KEY,
    member_id INT NOT NULL,
    codeforces_handle VARCHAR(255) NOT NULL,
    codeforces_rating INT NOT NULL,
    max_rating INT NOT NULL,
    contests_participated INT NOT NULL,
    FOREIGN KEY (member_id) REFERENCES member(member_id)
);

ALTER TABLE leetcode_stats ADD CONSTRAINT leetcode_stats_member_id_key UNIQUE (member_id);
ALTER TABLE codeforces_stats ADD CONSTRAINT codeforces_stats_member_id_key UNIQUE (member_id);