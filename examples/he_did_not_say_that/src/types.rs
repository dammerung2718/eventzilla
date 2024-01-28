use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct RandomPerson {
    pub results: Vec<Person>,
}

#[derive(Clone, Deserialize)]
pub struct Person {
    pub name: Name,
}

#[derive(Clone, Deserialize)]
pub struct Name {
    pub first: String,
    pub last: String,
}

#[derive(Clone, Deserialize)]
pub struct RandomQuote {
    pub content: String,
}
