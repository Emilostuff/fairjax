use actix::prelude::*;
use fairjax::*;
use fairjax_core::MailBox;

/// Define message enum
#[derive(Debug)]
enum Event {
    A(usize),
    B(usize),
}

/// Implement Message trait for Event
impl Message for Event {
    type Result = Option<usize>;
}

/// Custom Actor
#[derive(Default)]
struct MyActor {
    mailbox: MailBox<Event>,
}

/// Declare actor and its context
impl Actor for MyActor {
    type Context = Context<Self>;
}

/// Handler for `Event` type messages
impl Handler<Event> for MyActor {
    type Result = Option<usize>;

    fn handle(&mut self, msg: Event, _: &mut Context<Self>) -> Self::Result {
        fairjax!(match msg >> [self.mailbox, Event] {
            (Event::A(x), Event::B(x)) => return Some(x),
        });
        None
    }
}

#[actix::main]
async fn main() {
    // Init and start new actor
    let addr = MyActor::start_default();

    // Send messages and process response
    for event in [Event::A(1), Event::A(2), Event::A(3), Event::B(1)] {
        print!("Sending: {:?} ... ", &event);
        match addr.send(event).await {
            Err(e) => println!("Error: {}", e),
            Ok(Some(matched_id)) => println!("Match: id = {:?}", matched_id),
            Ok(None) => println!("No match"),
        }
    }

    // Stop system and exit
    System::current().stop();
}
