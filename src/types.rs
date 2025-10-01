use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Opt {
    pub value: usize,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Question {
    pub id: String,
    pub prompt: String,
    pub options: Vec<Opt>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data {
    pub user: User,
    pub q: BTreeMap<String, usize>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestResultRecord {
    pub answer_txt: String,
    pub answer_value: usize,
    pub question_txt: String,
    pub question_id: String,
}
