use async_graphql::MergedObject;
use mutations::{AttendanceMutations, MemberMutations, ProjectMutations, StreakMutations,FetchLeetCode,FetchCodeForces,LeaderboardMutation};
use queries::{AttendanceQueries, MemberQueries, ProjectQueries, StreakQueries, LeaderboardQueries};

pub mod mutations;
pub mod queries;

#[derive(MergedObject, Default)]
pub struct Query(
    MemberQueries,
    AttendanceQueries,
    StreakQueries,
    ProjectQueries,
    LeaderboardQueries,
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    MemberMutations,
    AttendanceMutations,
    StreakMutations,
    ProjectMutations,
    FetchLeetCode,
    FetchCodeForces,
    LeaderboardMutation,

);
