use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Token {
    Table,
    Id,
    Token,
    Expiry,
    FkHackerId,
}

#[derive(Iden)]
enum Hacker {
    Table,
    DiscordId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Token::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Token::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Token::Token).string().not_null())
                    .col(ColumnDef::new(Token::Expiry).date_time().not_null())
                    .col(ColumnDef::new(Token::FkHackerId).string().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("submission_hacker_fk")
                            .from(Token::Table, Token::FkHackerId)
                            .to(Hacker::Table, Hacker::DiscordId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Token::Table).to_owned())
            .await
    }
}
