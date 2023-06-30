// use entity::entities::challenge;
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
    Active,
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
                    .col(
                        ColumnDef::new(Challenge::Title)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Challenge::Category).string().not_null())
                    .col(ColumnDef::new(Challenge::Description).string().not_null())
                    .col(ColumnDef::new(Challenge::Link).string().not_null())
                    .col(ColumnDef::new(Challenge::Points).integer().not_null())
                    .col(ColumnDef::new(Challenge::Flag).string().not_null())
                    .col(ColumnDef::new(Challenge::Author).string().not_null())
                    .col(ColumnDef::new(Challenge::Active).boolean().not_null())
                    .to_owned(),
            )
            .await?;

        let _db = manager.get_connection();
        // let transaction = db.begin().await?;

        // // Add sample challenges
        // challenge::ActiveModel {
        //     title: Set("Challenge 1".to_owned()),
        //     category: Set("Category 1".to_owned()),
        //     description: Set("Description 1".to_owned()),
        //     link: Set("Link 1".to_owned()),
        //     points: Set(100),
        //     flag: Set("flag{test1}".to_owned()),
        //     author: Set("Author 1".to_owned()),
        //     ..Default::default()
        // }
        // .insert(db)
        // .await?;

        // challenge::ActiveModel {
        //     title: Set("Challenge 2".to_owned()),
        //     category: Set("Category 2".to_owned()),
        //     description: Set("Description 2".to_owned()),
        //     link: Set("Link 2".to_owned()),
        //     points: Set(200),
        //     flag: Set("flag{test2}".to_owned()),
        //     author: Set("Author 2".to_owned()),
        //     ..Default::default()
        // }
        // .insert(db)
        // .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Challenge::Table).to_owned())
            .await
    }
}
