# NOTES

- [X] Event Store Interface
    - `fn state() -> State`
    - `fn history() -> Vec<Event>`
    - `fn emit(event: Event)`
    - `fn replay(events: Vec<Event>)`

- Built-in Event Stores
    - [X] InMemoryStore
    - [ ] FsStore
    - [ ] PostgresStore

- [ ] Replayability
    - How to deal with external dependencies?
    - Integration to testing frameworks?

- [ ] Subscriptions
    - [ ] `fn subscribe(handler: Handler)`
    - [ ] `fn unsubscribe(handler: Handler)`

- [ ] Async/await
    - How to deal with asynchronous systems?
