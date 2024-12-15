use std::collections::BTreeMap;

/// Enumerator of the Event type. Whatever type E of Event::Args you implement here is the type E that will be used for the EventPublisher.
#[repr(C)]
#[derive(Debug)]
pub enum Event<E> {
    Args(E),
    Missing,
}

// To deal with handler functions - F: Rc<Box<Fn(&event<E>)>>
/// EventPublisher. Works similarly to C#'s event publishing pattern. Event handling functions are subscribed to the publisher.
/// Whenever the publisher fires an event it calls all subscribed event handler functions.
/// Use event::EventPublisher::<E>::new() to construct
#[repr(C)]
pub struct EventPublisher<E> {
    //handlers: Vec<Rc<Box<Fn(&Event<E>) + 'static>>>,
    handlers: BTreeMap<usize, extern "C" fn(&Event<E>)>,
}

impl<E> EventPublisher<E> {

    /// Event publisher constructor.
    pub fn new() -> EventPublisher<E> {
        EventPublisher{ 
            handlers: BTreeMap::<usize, extern "C" fn(&Event<E>)>::new()
        }
    }
    /// Subscribes event handler functions to the EventPublisher.
    /// INPUT:  handler: fn(&Event<E>) handler is a pointer to a function to handle an event of the type E. The function must
    ///     be capable of handling references to the event type set up by the publisher, rather than the raw event itself.
    /// OUTPUT: void
    pub fn subscribe_handler(&mut self, handler: extern "C" fn(&Event<E>)){
        let p_handler: usize;
        unsafe{
            p_handler = *(handler as *const usize);
        }
        self.handlers.insert(p_handler, handler);
    }
    
    /// Unsubscribes an event handler from the publisher.
    /// INPUT:  handler: fn(&Event<E>) handler is a pointer to a function to handle an event of the type E.
    /// OUTPUT: bool    output is a bool of whether or not the function was found in the list of subscribed event handlers and subsequently removed.
    pub fn unsubscribe_handler(&mut self, handler: extern "C" fn(&Event<E>)) -> bool {
        let p_handler: usize;
        unsafe{
            p_handler = *(handler as *const usize);
        }
        self.handlers.remove(&p_handler).is_some()
    }
        
    // TODO: Implement this concurrently
    /// Publishes events, pushing the &Event<E> to all handler functions stored by the event publisher.
    /// INPUT: event: &Event<E>     Reference to the Event<E> being pushed to all handling functions.
    pub fn publish_event(&self, event: &Event<E>){
        for (_, handler) in self.handlers.iter(){
            handler(event);
        }
    }
}