use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum MessageComponentData {
    Table,
    IdUuid,
    Payload,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MessageComponentData::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MessageComponentData::IdUuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(MessageComponentData::Payload)
                            .json_binary()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MessageComponentData::Table).to_owned())
            .await
    }
}
