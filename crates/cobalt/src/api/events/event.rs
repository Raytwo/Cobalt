use std::collections::BTreeMap;

use cobapi::Event;

#[repr(C)]
pub struct EventPublisher<E> {
    handlers: BTreeMap<usize, extern "C" fn(&Event<E>)>,
}

impl<E> EventPublisher<E> {
    pub fn new() -> EventPublisher<E> {
        EventPublisher {
            handlers: BTreeMap::<usize, extern "C" fn(&Event<E>)>::new(),
        }
    }

    pub fn subscribe_handler(&mut self, handler: extern "C" fn(&Event<E>)) {
        let key = handler as usize;
        self.handlers.insert(key, handler);
    }

    pub fn unsubscribe_handler(&mut self, handler: extern "C" fn(&Event<E>)) -> bool {
        let key = handler as usize;
        self.handlers.remove(&key).is_some()
    }

    pub fn publish_event(&self, event: &Event<E>) {
        let snapshot: Vec<extern "C" fn(&Event<E>)> = self.handlers.values().copied().collect();

        for handler in snapshot {
            handler(event);
        }
    }
}

impl<E> Default for EventPublisher<E> {
    fn default() -> Self {
        Self::new()
    }
}
