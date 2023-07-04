use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Hacker {
    Table,
    DiscordId,
    Username,
    FkTeamId,
}

#[derive(Iden)]
enum Team {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Hacker::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Hacker::DiscordId)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Hacker::Username).string().not_null())
                    .col(ColumnDef::new(Hacker::FkTeamId).integer().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("hacker_team_fk")
                            .from(Hacker::Table, Hacker::FkTeamId)
                            .to(Team::Table, Team::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Hacker::Table).to_owned())
            .await
    }
}
