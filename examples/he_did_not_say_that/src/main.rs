mod types;

use axum::{extract::State, routing::get, Json};
use eventzilla::{testing, Reducer, Store};
use serde::Serialize;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Clone)]
struct App {
    store: Arc<dyn Store<Event>>,
    quotes: Reducer<Vec<Quote>, Event>,
}

#[derive(Clone, Serialize)]
enum Event {
    CreatedQuote(Quote),
}

#[derive(Clone, Serialize)]
struct Quote {
    pub content: String,
    pub not_said_by: String,
}

#[tokio::main]
async fn main() {
    // event store
    let mut store = testing::TestingStore::new();

    // aggregates
    let quotes = Reducer::new(
        |data, event| {
            let Event::CreatedQuote(quote) = event;
            data.push(quote);
        },
        vec![],
    );
    store.add(quotes.subscription());

    // axum app
    let app = App {
        store: Arc::new(store),
        quotes,
    };

    // routes
    let app = axum::Router::new()
        .route("/history", get(history))
        .route("/quotes", get(quotes_))
        .route("/new", get(new))
        .with_state(app);

    // start
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn history(State(app): State<App>) -> Json<Vec<Event>> {
    let history = app.store.history().lock().await.clone();
    Json(history)
}

async fn quotes_(State(app): State<App>) -> Json<Vec<Quote>> {
    Json(app.quotes.read().await)
}

async fn new(State(app): State<App>) -> Json<Quote> {
    let random_person = reqwest::get("https://randomuser.me/api/")
        .await
        .unwrap()
        .json::<types::RandomPerson>()
        .await
        .unwrap()
        .results[0]
        .clone();

    let random_quote = reqwest::get("https://api.quotable.io/random")
        .await
        .unwrap()
        .json::<types::RandomQuote>()
        .await
        .unwrap();

    let quote = Quote {
        content: random_quote.content,
        not_said_by: format!("{} {}", random_person.name.first, random_person.name.last),
    };

    app.store.emit(Event::CreatedQuote(quote.clone()));
    Json(quote)
}
