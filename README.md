# 📢 Emitter RS

[![Crates.io](https://img.shields.io/crates/v/emitter-rs.svg)](https://crates.io/crates/emitter-rs)
[![License](https://img.shields.io/crates/l/emitter-rs.svg)](https://opensource.org/licenses/MIT)

> 📢 Emitter RS is a simple EventEmitter implementation for Rust and Wasm, providing easy event subscription and firing capabilities.

## 🚀 Getting Started

To start using `emitter-rs`, add it to your `Cargo.toml`:

```toml
[dependencies]
emitter-rs = "0.0.4"
```

Then, you can use it in your code:

```rust
use emitter_rs::EventEmitter;

fn main() {
    let mut event_emitter = EventEmitter::new();

    event_emitter.on("Say Hello", |value: String| {
        println!("{}", value);
    });

    event_emitter.emit("Say Hello", "Hello world!".to_string());
}
```

## 💡 Basic Usage

You can emit and listen to values of any type as long as they implement the `serde` `Serialize` and `Deserialize` traits. A single `EventEmitter` instance can have listeners to values of multiple types.

```rust
use emitter_rs::EventEmitter;

fn main() {
    let mut event_emitter = EventEmitter::new();

    event_emitter.on("Add three", |number: f32| println!("{}", number + 3.0));

    event_emitter.emit("Add three", 5.0 as f32);
    event_emitter.emit("Add three", 4.0 as f32);
}
// >> "8"
// >> "7"
```

Using a more advanced value type such as a struct by implementing the `serde` traits:

```rust
use emitter_rs::EventEmitter;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Date {
    month: String,
    day: String,
}

fn main() {
    let mut event_emitter = EventEmitter::new();

    event_emitter.on("LOG_DATE", |date: Date| {
        println!("Month: {} - Day: {}", date.month, date.day)
    });

    event_emitter.emit("LOG_DATE", Date {
        month: "January".to_string(),
        day: "Tuesday".to_string()
    });
}
// >> "Month: January - Day: Tuesday"
```

Removing listeners is also easy:

```rust
use emitter_rs::EventEmitter;

fn main() {
    let mut event_emitter = EventEmitter::new();

    let listener_id = event_emitter.on("Hello", |_: ()| println!("Hello World"));
    match event_emitter.remove_listener(&listener_id) {
        Some(_listener_id) => println!("Removed event listener!"),
        None => println!("No event listener of that id exists")
    }
}

// >> "Removed event listener!"
```

## 🌐 Creating a Global EventEmitter

You can create a global `EventEmitter` instance that can be shared across files:

```rust
// global_event_emitter.rs
use std::sync::Mutex;
use emitter_rs::EventEmitter;
use lazy_static::lazy_static;

// Use lazy_static! because the size of EventEmitter is not known at compile time
lazy_static! {
    // Export the emitter with `pub` keyword
    pub static ref EVENT_EMITTER: Mutex<EventEmitter> = Mutex::new(EventEmitter::new());
}
```

Then import this instance into multiple files:

```ignore
// main.rs
mod global_event_emitter;
use global_event_emitter::EVENT_EMITTER;

fn main() {
    // We need to maintain a lock through the mutex to avoid data races
    EVENT_EMITTER.lock().unwrap().on("Hello", |_: ()| println!("hello there!"));
    EVENT_EMITTER.lock().unwrap().emit("Hello", ());
}
```

And in another file, you can listen to the `"Hello"` event in `main.rs` by adding a listener to the global event emitter:

```rust
// some_random_file.rs
use emitter_rs::event_emitter_file::EVENT_EMITTER;

fn random_function() {
    // When the "Hello" event is emitted in `main.rs`, print "Random stuff!"
    EVENT_EMITTER.lock().unwrap().on("Hello", |_: ()| println!("Random stuff!"));
}
```

## 🌟 Usage in WASM

`Emitter RS` can be seamlessly integrated into WebAssembly (WASM) projects, allowing you to create event-driven applications in the browser. Consider the following as an example:

```ignore
use emitter_rs::EventEmitter;
use std::sync::{Arc, Mutex};
use wasm_bindgen_futures::spawn_local;

fn run() {
    let mut event_emitter = EventEmitter::new();
    let result = Arc::new(Mutex::new(String::new()));

    let cloned_result = Arc::clone(&result);

    spawn_local(async move {
        event_emitter.on("some_event", move |value: String| {
            let mut result = cloned_result.lock().unwrap();
            result.push_str(&value);
        });

        event_emitter.emit("some_event", "Hello, world!".to_string());

        assert_eq!(*result.lock().unwrap(), "Hello, world!");
    });
}
```

> [!NOTE]
Emitter RS is a maintained fork of [`event-emitter-rs`](https://crates.io/crates/event-emitter-rs) crate.

## 📄 License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT).
