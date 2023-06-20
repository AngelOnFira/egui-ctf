use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Submission {
    Table,
    Id,
    FkHackerId,
    FkChallengeId,
    Flag,
}

#[derive(Iden)]
enum Hacker {
    Table,
    Id,
}

#[derive(Iden)]
enum Challenge {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Submission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Submission::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Submission::Flag).string().not_null())
                    .col(ColumnDef::new(Submission::FkHackerId).big_integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("submission_hacker_fk")
                            .from(Submission::Table, Submission::FkHackerId)
                            .to(Hacker::Table, Hacker::Id),
                    )
                    .col(ColumnDef::new(Submission::FkChallengeId).big_integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("submission_challenge_fk")
                            .from(Submission::Table, Submission::FkChallengeId)
                            .to(Challenge::Table, Challenge::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Submission::Table).to_owned())
            .await
    }
}
