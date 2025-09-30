#![cfg(feature = "ssr")]
use crate::types::*;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use uuid::Uuid;

use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    RecordId, Surreal,
};

pub struct Db;
static DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);

#[derive(Debug, Deserialize, Serialize)]
pub struct UserRecord {
    pub id: RecordId,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TestRecord {
    pub id: RecordId,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QuestionRecord {
    pub id: RecordId,
    pub prompt: String,
    pub options: Vec<Opt>,
}

impl TryFrom<UserRecord> for User {
    type Error = color_eyre::eyre::Error;

    fn try_from(record: UserRecord) -> Result<User, Self::Error> {
        let id = record.id.key().clone().try_into()?;
        Ok(User { id })
    }
}

impl TryFrom<QuestionRecord> for Question {
    type Error = color_eyre::eyre::Error;

    fn try_from(record: QuestionRecord) -> Result<Question, Self::Error> {
        let id = record.id.key().clone().try_into()?;
        Ok(Question {
            id,
            prompt: record.prompt,
            options: record.options,
        })
    }
}

impl Db {
    pub async fn connect() -> Result<Self> {
        DB.connect::<Ws>("localhost:8000").await?;
        DB.signin(Root {
            username: "root",
            password: "root",
        })
        .await?;

        DB.use_ns("test").use_db("test").await?;

        DB.query("DEFINE TABLE IF NOT EXISTS user;").await?;
        let db = Self;
        db.generate_questions().await?;
        Ok(db)
    }

    async fn generate_questions(&self) -> Result<()> {
        let txt = include_str!("../data/aq.json");
        let questions: Vec<Question> = serde_json::from_str(&txt)?;
        DB.query("DEFINE TABLE OVERWRITE test; DELETE test;")
            .await?;
        DB.query("DEFINE TABLE OVERWRITE question; DELETE question;")
            .await?;
        let _: Option<TestRecord> = DB.create(("test", "aq")).await?;
        let records: Vec<QuestionRecord> = DB.insert("question").content(questions).await?;
        for r in records.into_iter() {
            DB.query("RELATE test:aq -> contains -> $ID;")
                .bind(("ID", r.id))
                .await?;
        }
        Ok(())
    }

    pub fn get() -> Self {
        Self
    }

    pub async fn create_user(&self) -> Result<User> {
        let user: Option<UserRecord> = DB.query("CREATE user:uuid();").await?.take(0)?;
        Ok(user.unwrap().try_into()?)
    }

    pub async fn get_questions(&self) -> Result<Vec<Question>> {
        let questions: Vec<QuestionRecord> = DB
            .query("SELECT * from test:aq -> contains -> question")
            .await?
            .take(0)?;
        questions.into_iter().map(|q| q.try_into()).collect()
    }
}
