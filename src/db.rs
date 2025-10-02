#![cfg(feature = "ssr")]
use crate::types::*;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::LazyLock;
use uuid::Uuid;

use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    RecordId, Surreal,
};

#[derive(Debug)]
pub struct Db;

static DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserRecord {
    pub id: RecordId,
    pub retrieval_id: Uuid,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestRecord {
    pub id: RecordId,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QuestionRecord {
    pub id: RecordId,
    pub prompt: String,
    pub options: Vec<Opt>,
}

impl TryFrom<UserRecord> for User {
    type Error = color_eyre::eyre::Error;

    fn try_from(record: UserRecord) -> Result<User, Self::Error> {
        let id: String = record.id.key().to_string();
        let id = id.parse()?;
        Ok(User {
            id,
            retrieval_id: record.retrieval_id,
        })
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
    #[tracing::instrument(err)]
    pub async fn connect() -> Result<Self> {
        DB.connect::<Ws>("localhost:8000").await?;
        DB.signin(Root {
            username: "root",
            password: "root",
        })
        .await?;

        DB.use_ns("test").use_db("test").await?;

        let db = Self;
        db.setup().await?;
        Ok(db)
    }

    #[tracing::instrument(err)]
    async fn setup(&self) -> Result<()> {
        DB.query(
            r#"
            BEGIN;
            DEFINE TABLE IF NOT EXISTS user;
            DEFINE FIELD IF NOT EXISTS retrieval_id ON TABLE user TYPE uuid READONLY VALUE rand::uuid();
            DEFINE TABLE IF NOT EXISTS says TYPE RELATION IN user OUT question ENFORCED;
            DEFINE TABLE OVERWRITE test; DELETE test;
            DEFINE TABLE OVERWRITE question; DELETE question;
            COMMIT;
        "#,
        )
        .await?;

        let txt = include_str!("../data/aq.json");
        let questions: Vec<Question> = serde_json::from_str(&txt)?;

        let _: Option<TestRecord> = DB
            .create("test")
            .content(TestRecord {
                id: RecordId::from_table_key("test", "aq"),
                name: String::from("AQ"),
            })
            .await?;
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

    #[tracing::instrument(err)]
    pub async fn create_user(&self, tests: BTreeSet<String>) -> Result<User> {
        let user: Option<UserRecord> = DB
            .query(
                r#"
        LET $USER = CREATE user;
        RELATE $USER -> assigned -> (select * from test where meta::id(id) in $TESTS);
        RETURN $USER
        "#,
            )
            .bind(("TESTS", tests))
            .await?
            .take(2)?;
        Ok(user.unwrap().try_into()?)
    }

    #[tracing::instrument(err)]
    pub async fn get_questions(&self) -> Result<Vec<Question>> {
        let questions: Vec<QuestionRecord> = DB
            .query("SELECT * from test:aq -> contains -> question")
            .await?
            .take(0)?;

        questions.into_iter().map(|q| q.try_into()).collect()
    }

    #[tracing::instrument(err)]
    pub async fn submit_test(&self, data: Data) -> Result<()> {
        tracing::info!("Submitting test data");
        let user_id = data.user.id;
        for (q_id, choice) in data.q.into_iter() {
            DB.query(
                r#"
                    BEGIN;
                    LET $UID = type::thing("user", $USER_ID);
                    LET $QID = type::thing("question", $QUESTION_ID);
                    DELETE FROM says where in = $UID AND out = $QID;
                    RELATE ONLY $UID -> says -> $QID set choice = $CHOICE;
                    COMMIT;
                "#,
            )
            .bind(("USER_ID", user_id.clone()))
            .bind(("QUESTION_ID", q_id))
            .bind(("CHOICE", choice))
            .await?
            .check()?;
        }
        tracing::info!("Test submission saved!");
        Ok(())
    }

    #[tracing::instrument(err)]
    pub async fn evaluate_test(&self, user_id: String) -> Result<Vec<TestResultRecord>> {
        let res = DB.query(r#"
            SELECT 
                meta::id(out.id) AS question_id,
                out.prompt AS question_txt,
                (SELECT VALUE label FROM ONLY $this.out.options[$parent.choice]) AS answer_txt,
                (SELECT VALUE value FROM ONLY $this.out.options[$parent.choice]) AS answer_value
                FROM says WHERE meta::id(in) = $USER_ID AND out INSIDE (test:aq->contains->question) ORDER BY question_id NUMERIC
            ;
        "#).bind(("USER_ID", user_id.clone())).await?.take::<Vec<TestResultRecord>>(0)?;
        crate::report::generate_pdf(user_id, &res).await?;
        Ok(res)
    }

    #[tracing::instrument(err)]
    pub async fn get_assigned_tests(&self, user_id: String) -> Result<Vec<Test>> {
        Ok(DB
            .query(
                r#"
                SELECT * FROM type::thing("user", $USER) -> assigned -> test FETCH test;
                "#,
            )
            .bind(("USER", user_id))
            .await?
            .take::<Vec<TestRecord>>(0)?
            .into_iter()
            .map(Test::from)
            .collect())
    }

    #[tracing::instrument(err)]
    pub async fn get_available_tests(&self) -> Result<Vec<Test>> {
        Ok(DB
            .query("SELECT * from test")
            .await?
            .take::<Vec<TestRecord>>(0)?
            .into_iter()
            .map(Test::from)
            .collect())
    }
}

impl From<TestRecord> for Test {
    fn from(r: TestRecord) -> Test {
        Test {
            id: r.id.key().to_string(),
            name: r.name,
        }
    }
}
