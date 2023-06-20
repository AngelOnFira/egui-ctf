pub use sea_orm_migration::prelude::*;

mod m20230620_142127_team;
mod m20230620_143222_hacker;
mod m20230620_143225_challenge;
mod m20230620_143309_submission;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230620_142127_team::Migration),
            Box::new(m20230620_143222_hacker::Migration),
            Box::new(m20230620_143225_challenge::Migration),
            Box::new(m20230620_143309_submission::Migration),
        ]
    }
}
