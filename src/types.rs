use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data {
    pub user: User,
    pub q: BTreeMap<String, Option<usize>>,
}
