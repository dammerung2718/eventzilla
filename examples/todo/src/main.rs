use axum::{
    extract::State,
    routing::{get, post},
    Json,
};
use eventzilla::{testing, Reducer, Store};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone, Serialize)]
enum Event {
    UserRegistered { username: String },
    UserCreatedTask { username: String, task: String },
    UserCompletedTask { username: String, task: String },
    UserDeletedTask { username: String, task: String },
}

#[derive(Clone, Serialize)]
pub struct Task {
    pub description: String,
    pub completed: bool,
}

#[derive(Clone)]
struct App {
    store: Arc<dyn Store<Event>>,
    users: Reducer<UsersData, Event>,
    tasks: Reducer<TasksData, Event>,
}

type UsersData = HashSet<String>;
type TasksData = HashMap<String, Vec<Task>>;

#[tokio::main]
async fn main() {
    // event store
    let mut store = testing::TestingStore::new();

    // aggregates
    let users_data: UsersData = Default::default();
    let users = Reducer::new(
        |data, event| {
            if let Event::UserRegistered { username } = event {
                data.insert(username);
            }
        },
        users_data,
    );
    store.add(users.subscription());

    let tasks_data: TasksData = Default::default();
    let tasks = Reducer::new(
        |data, event| match event {
            Event::UserCreatedTask { username, task } => {
                data.entry(username).or_default().push(Task {
                    description: task,
                    completed: false,
                });
            }
            Event::UserCompletedTask { username, task } => {
                if let Some(t) = data
                    .entry(username)
                    .or_default()
                    .iter_mut()
                    .find(|t| *t.description == task)
                {
                    t.completed = true;
                }
            }
            Event::UserDeletedTask { username, task } => {
                data.entry(username)
                    .or_default()
                    .retain(|t| t.description != task);
            }
            _ => {}
        },
        tasks_data,
    );
    store.add(tasks.subscription());

    // axum app
    let app = App {
        store: Arc::new(store),
        users,
        tasks,
    };

    // web server
    let app = axum::Router::new()
        // event endpoints
        .route("/history", get(history))
        .route("/users", get(users_))
        .route("/tasks", get(tasks_))
        // auth endpoints
        .route("/register", post(register))
        // task endpoints
        .route("/create_task", post(create_task))
        .route("/complete_task", post(complete_task))
        .route("/delete_task", post(delete_task))
        // store
        .with_state(app);

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap()
}

async fn history(State(app): State<App>) -> Json<Vec<Event>> {
    Json(app.store.history().lock().await.clone())
}

async fn users_(State(app): State<App>) -> Json<HashSet<String>> {
    Json(app.users.read().await)
}

async fn tasks_(State(app): State<App>) -> Json<HashMap<String, Vec<Task>>> {
    Json(app.tasks.read().await)
}

async fn register(State(app): State<App>, Json(payload): Json<RegisterPayload>) -> Json<String> {
    let users = app.users.read().await;
    if users.contains(&payload.username) {
        return Json("User already exists".into());
    }

    app.store.emit(Event::UserRegistered {
        username: payload.username,
    });

    Json("succeded".into())
}

#[derive(Deserialize)]
struct RegisterPayload {
    username: String,
}

async fn create_task(
    State(app): State<App>,
    Json(payload): Json<CreateTaskPayload>,
) -> Json<String> {
    let users = app.users.read().await;
    if !users.contains(&payload.username) {
        return Json("User does not exist".into());
    }

    app.store.emit(Event::UserCreatedTask {
        username: payload.username,
        task: payload.task,
    });

    Json("succeeded".into())
}

#[derive(Deserialize)]
struct CreateTaskPayload {
    username: String,
    task: String,
}

async fn complete_task(
    State(app): State<App>,
    Json(payload): Json<CompleteTaskPayload>,
) -> Json<String> {
    let users = app.users.read().await;
    if !users.contains(&payload.username) {
        return Json("User does not exist".into());
    }

    app.store.emit(Event::UserCompletedTask {
        username: payload.username,
        task: payload.task,
    });

    Json("succeeded".into())
}

#[derive(Deserialize)]
struct CompleteTaskPayload {
    username: String,
    task: String,
}

async fn delete_task(
    State(app): State<App>,
    Json(payload): Json<DeleteTaskPayload>,
) -> Json<String> {
    let users = app.users.read().await;
    if !users.contains(&payload.username) {
        return Json("User does not exist".into());
    }

    app.store.emit(Event::UserDeletedTask {
        username: payload.username,
        task: payload.task,
    });

    Json("succeeded".into())
}

#[derive(Deserialize)]
struct DeleteTaskPayload {
    username: String,
    task: String,
}
