use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

/// Represents a single event listener.
pub struct Listener {
    pub callback: Arc<dyn Fn(Vec<u8>) + Sync + Send + 'static>,
    pub limit: Option<u64>,
    pub id: String,
}

/// Manages event listeners and event emissions.
#[derive(Default)]
pub struct EventEmitter {
    pub listeners: HashMap<String, Vec<Listener>>,
}

impl EventEmitter {
    /// Creates a new `EventEmitter` instance.
    ///
    /// # Returns
    ///
    /// A new `EventEmitter` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use emitter_rs::EventEmitter;
    /// let event_emitter = EventEmitter::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an event listener with a callback that will be called whenever the given event is emitted.
    ///
    /// # Arguments
    ///
    /// * `event` - The name of the event to listen for.
    /// * `callback` - The callback function to execute when the event is emitted.
    ///
    /// # Returns
    ///
    /// The ID of the newly added listener.
    ///
    /// # Examples
    ///
    /// ```
    /// use emitter_rs::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    ///
    /// event_emitter.on("some_event", |value: String| {
    ///     println!("Received event with value: {}", value);
    /// });
    /// ```
    pub fn on<F, T>(&mut self, event: &str, callback: F) -> String
    where
        for<'de> T: Deserialize<'de>,
        F: Fn(T) + 'static + Sync + Send,
    {
        self.on_limited(event, None, callback)
    }

    /// Emits an event with the given parameters, executing each callback asynchronously by spawning a new thread for each callback.
    ///
    /// # Arguments
    ///
    /// * `event` - The name of the event to emit.
    /// * `value` - The value to pass to the event listeners.
    ///
    /// # Examples
    ///
    /// ```
    /// use emitter_rs::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    ///
    /// event_emitter.emit("some_event", "Hello, world!".to_string());
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn emit<T>(&mut self, event: &str, value: T)
    where
        T: Serialize,
    {
        let mut callback_handlers = Vec::new();

        if let Some(listeners) = self.listeners.get_mut(event) {
            let bytes = serde_json::to_vec(&value).unwrap();

            let mut listeners_to_remove = Vec::new();
            for (index, listener) in listeners.iter_mut().enumerate() {
                let cloned_bytes = bytes.clone();
                let callback = Arc::clone(&listener.callback);

                match listener.limit {
                    None => {
                        callback_handlers.push(thread::spawn(move || {
                            callback(cloned_bytes);
                        }));
                    }
                    Some(limit) => {
                        if limit != 0 {
                            callback_handlers.push(thread::spawn(move || {
                                callback(cloned_bytes);
                            }));
                            listener.limit = Some(limit - 1);
                        } else {
                            listeners_to_remove.push(index);
                        }
                    }
                }
            }

            for index in listeners_to_remove.into_iter().rev() {
                listeners.remove(index);
            }
        }

        for handler in callback_handlers {
            if let Err(e) = handler.join() {
                eprintln!("Thread error: {:?}", e);
            }
        }
    }

    /// Emits an event with the given parameters, executing each callback asynchronously using `spawn_local` for WebAssembly.
    ///
    /// # Arguments
    ///
    /// * `event` - The name of the event to emit.
    /// * `value` - The value to pass to the event listeners.
    ///
    /// # Examples
    ///
    /// ```
    /// use emitter_rs::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    ///
    /// event_emitter.emit("some_event", "Hello, world!".to_string());
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn emit<T>(&mut self, event: &str, value: T)
    where
        T: Serialize + 'static,
    {
        if let Some(listeners) = self.listeners.get_mut(event) {
            let bytes = serde_json::to_vec(&value).unwrap();
            let mut listeners_to_remove = Vec::new();

            for (index, listener) in listeners.iter_mut().enumerate() {
                let cloned_bytes = bytes.clone();
                let callback = Arc::clone(&listener.callback);

                match listener.limit {
                    None => {
                        let future = async move {
                            callback(cloned_bytes);
                        };
                        spawn_local(future);
                    }
                    Some(limit) => {
                        if limit != 0 {
                            let future = async move {
                                callback(cloned_bytes);
                            };
                            spawn_local(future);
                            listener.limit = Some(limit - 1);
                        } else {
                            listeners_to_remove.push(index);
                        }
                    }
                }
            }

            for &index in listeners_to_remove.iter().rev() {
                listeners.remove(index);
            }
        }
    }

    /// Removes an event listener with the given ID.
    ///
    /// # Arguments
    ///
    /// * `id_to_delete` - The ID of the listener to remove.
    ///
    /// # Returns
    ///
    /// An option containing the ID of the removed listener if found, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use emitter_rs::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    /// let listener_id = event_emitter.on("some_event", |value: String| {
    ///     println!("Received event with value: {}", value);
    /// });
    ///
    /// event_emitter.remove_listener(&listener_id);
    /// ```
    pub fn remove_listener(&mut self, id_to_delete: &str) -> Option<String> {
        for (_, event_listeners) in self.listeners.iter_mut() {
            if let Some(index) = event_listeners
                .iter()
                .position(|listener| listener.id == id_to_delete)
            {
                event_listeners.remove(index);
                return Some(id_to_delete.to_string());
            }
        }

        None
    }

    /// Adds an event listener that will execute the callback a limited number of times.
    ///
    /// # Arguments
    ///
    /// * `event` - The name of the event to listen for.
    /// * `limit` - The number of times the listener should be executed.
    /// * `callback` - The callback function to execute when the event is emitted.
    ///
    /// # Returns
    ///
    /// The ID of the newly added listener.
    ///
    /// # Examples
    ///
    /// ```
    /// use emitter_rs::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    ///
    /// event_emitter.on_limited("some_event", Some(3), |value: String| {
    ///     println!("Received event with value: {}", value);
    /// });
    /// ```
    pub fn on_limited<F, T>(&mut self, event: &str, limit: Option<u64>, callback: F) -> String
    where
        for<'de> T: Deserialize<'de>,
        F: Fn(T) + 'static + Sync + Send,
    {
        let id = Uuid::new_v4().to_string();
        let parsed_callback = move |bytes: Vec<u8>| {
            let value: T = serde_json::from_slice(&bytes).unwrap();
            callback(value);
        };

        let listener = Listener {
            id: id.clone(),
            limit,
            callback: Arc::new(parsed_callback),
        };

        match self.listeners.get_mut(event) {
            Some(callbacks) => {
                callbacks.push(listener);
            }
            None => {
                self.listeners.insert(event.to_string(), vec![listener]);
            }
        }

        id
    }

    /// Adds an event listener that will execute the callback only once.
    ///
    /// # Arguments
    ///
    /// * `event` - The name of the event to listen for.
    /// * `callback` - The callback function to execute when the event is emitted.
    ///
    /// # Returns
    ///
    /// The ID of the newly added listener.
    ///
    /// # Examples
    ///
    /// ```
    /// use emitter_rs::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    ///
    /// event_emitter.once("some_event", |value: String| {
    ///     println!("Received event with value: {}", value);
    /// });
    /// ```
    pub fn once<F, T>(&mut self, event: &str, callback: F) -> String
    where
        for<'de> T: Deserialize<'de>,
        F: Fn(T) + 'static + Sync + Send,
    {
        self.on_limited(event, Some(1), callback)
    }

    /// Emits an event with the given parameters synchronously, executing each callback in the order they were inserted.
    ///
    /// # Arguments
    ///
    /// * `event` - The name of the event to emit.
    /// * `value` - The value to pass to the event listeners.
    ///
    /// # Examples
    ///
    /// ```
    /// use emitter_rs::EventEmitter;
    /// let mut event_emitter = EventEmitter::new();
    ///
    /// event_emitter.on("some_event", |value: String| {
    ///     println!("1: {}", value);
    /// });
    /// event_emitter.on("some_event", |value: String| {
    ///     println!("2: {}", value);
    /// });
    ///
    /// event_emitter.sync_emit("some_event", "Hello, world!".to_string());
    /// ```
    pub fn sync_emit<T>(&self, event: &str, value: T)
    where
        T: Serialize,
    {
        if let Some(listeners) = self.listeners.get(event) {
            let bytes = serde_json::to_vec(&value).unwrap();

            for listener in listeners {
                let callback = Arc::clone(&listener.callback);
                callback(bytes.clone());
            }
        }
    }
}
