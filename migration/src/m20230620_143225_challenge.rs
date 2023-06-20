use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Challenge {
    Table,
    Id,
    Title,
    Category,
    Description,
    Link,
    Points,
    Flag,
    Author,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Challenge::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Challenge::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Challenge::Title).string().not_null())
                    .col(ColumnDef::new(Challenge::Category).string().not_null())
                    .col(ColumnDef::new(Challenge::Description).string().not_null())
                    .col(ColumnDef::new(Challenge::Link).string().not_null())
                    .col(ColumnDef::new(Challenge::Points).integer().not_null())
                    .col(ColumnDef::new(Challenge::Flag).string().not_null())
                    .col(ColumnDef::new(Challenge::Author).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Challenge::Table).to_owned())
            .await
    }
}
