use emitter_rs::EventEmitter;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test_configure;
#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[test]
fn test_on() {
    let mut event_emitter = EventEmitter::new();
    let counter: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));

    let cloned_counter = Arc::clone(&counter);
    event_emitter.on("Set", move |value: u32| {
        *cloned_counter.lock().unwrap() = value;
    });

    event_emitter.emit("Set", 10 as u32);

    assert_eq!(
        10,
        *counter.lock().unwrap(),
        "Counter should have been set to the emitted value"
    );

    struct Container {
        list: Vec<String>,
    }

    let container: Arc<Mutex<Container>> = Arc::new(Mutex::new(Container { list: Vec::new() }));

    let cloned_container = Arc::clone(&container);
    event_emitter.on("Add Value To List", move |value: String| {
        let mut container = cloned_container.lock().unwrap();
        (*container).list.push(value);
    });

    event_emitter.emit("Add Value To List", "hello".to_string());

    assert_eq!(
        vec!["hello".to_string()],
        (*container.lock().unwrap()).list,
        "'hello' should have been pushed to the list after the 'Add Value To List' event was called with 'hello'"
    );
}

#[test]
fn test_remove_listener() {
    let mut event_emitter = EventEmitter::new();
    let listener_id = event_emitter.on("Hello rust!", |_: String| println!("Hello world!"));
    assert_eq!(
        1,
        event_emitter.listeners.get("Hello rust!").unwrap().len(),
        "Failed to add event emitter to listeners vector"
    );

    event_emitter.remove_listener(&"foobar".to_string());
    assert_eq!(
        1,
        event_emitter.listeners.get("Hello rust!").unwrap().len(),
        "Should not have removed listener"
    );

    event_emitter.remove_listener(&listener_id);
    assert_eq!(
        0,
        event_emitter.listeners.get("Hello rust!").unwrap().len(),
        "Should have removed listener"
    );
}

#[test]
fn test_on_limited() {
    let mut event_emitter = EventEmitter::new();
    let counter: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));

    let cloned_counter = Arc::clone(&counter);
    event_emitter.on_limited("Set", Some(2), move |value: u32| {
        *cloned_counter.lock().unwrap() = value;
    });
    assert_eq!(
        2,
        event_emitter
            .listeners
            .get("Set")
            .unwrap()
            .first()
            .unwrap()
            .limit
            .unwrap(),
        "Listener should have been added with a limit of 2 calls"
    );

    event_emitter.emit("Set", 10 as u32);
    assert_eq!(
        1,
        event_emitter
            .listeners
            .get("Set")
            .unwrap()
            .first()
            .unwrap()
            .limit
            .unwrap(),
        "Listener limit should have been reduced by 1"
    );

    event_emitter.emit("Set", 20 as u32);
    assert_eq!(
        0,
        event_emitter
            .listeners
            .get("Set")
            .unwrap()
            .first()
            .unwrap()
            .limit
            .unwrap(),
        "Listener should have 0 calls left"
    );
    assert_eq!(
        20,
        *counter.lock().unwrap(),
        "Counter should have been set to the emitted value"
    );

    event_emitter.emit("Set", 30 as u32);
    assert_eq!(
        0,
        event_emitter.listeners.get("Set").unwrap().len(),
        "Listener should have been removed after reaching its limit"
    );
}

#[test]
fn test_once() {
    let mut event_emitter = EventEmitter::new();
    let counter: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));

    let cloned_counter = Arc::clone(&counter);
    event_emitter.once("Set Once", move |value: u32| {
        *cloned_counter.lock().unwrap() = value;
    });

    event_emitter.emit("Set Once", 10 as u32);
    assert_eq!(
        10,
        *counter.lock().unwrap(),
        "Counter should have been set to the emitted value"
    );

    event_emitter.emit("Set Once", 20 as u32);
    assert_eq!(
        10,
        *counter.lock().unwrap(),
        "Counter should not have been changed after the first call"
    );
}

#[test]
fn test_global_emitter() {
    lazy_static! {
        static ref EVENT_EMITTER: Mutex<EventEmitter> = Mutex::new(EventEmitter::new());
    }

    EVENT_EMITTER
        .lock()
        .unwrap()
        .on("Hello", |_: ()| println!("hello there!"));
    EVENT_EMITTER.lock().unwrap().emit("Hello", ());
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_on_and_emit_wasm() {
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
