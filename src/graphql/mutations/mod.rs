pub mod attendance_mutations;
pub mod member_mutations;
pub mod project_mutations;
pub mod streak_mutations;
pub mod update_leaderboard;  //leaderboard
pub mod leetcode_status;
pub mod codeforces_status;



pub use attendance_mutations::AttendanceMutations;
pub use member_mutations::MemberMutations;
pub use project_mutations::ProjectMutations;
pub use streak_mutations::StreakMutations;
pub use leetcode_status::LeetCodeStats;
pub use codeforces_status::CodeforcesStats;
pub use update_leaderboard::LeaderboardMutation;


//use any mutations for leaderboard if needed