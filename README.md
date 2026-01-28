# Fairjax
A Rust crate for efficiently performing _fair_ join pattern matching i.e. the process of matching several
received messages in a mailbox against one of more patterns.
If we can find a combination of messages that matches a pattern and satisfies the guard,
the messages are consumed and the body of the match arm is executed.

## Fair and Deterministic Matching
`fairjax` implements join pattern matching in a fair and deterministic manner,
ensuring that we always match the oldest messages possible. Furthermore this ensures completely
deterministic and reproducible behavior.

## Example
```rust
// Define message types
pub enum Msg {
    A(usize, f64),
    B(usize, f64),
}

use Msg::*;

// Declare state-keeping mailbox
let mut mailbox = fairjax_core::MailBox::default();

// Simulate input messages
let messages = vec![
    A(0, 1.4),
    B(0, 1.8),
    B(1, 5.2),
    A(2, 42.1),
    A(1, 5.9),
    B(2, 42.5)
];

let mut matches = vec![];

// Recieve message one by one
for msg in messages {
    // Declare join pattern matching
    fairjax::fairjax!(match msg >> [mailbox, Msg] {
       (A(id, ts1), B(id, ts2)) if ts1 < ts2 => {
           matches.push(id);
       }
    });
}

assert_eq!(vec![0, 2], matches);
```
