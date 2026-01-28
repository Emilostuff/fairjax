//! # Fairjax
//!
//! A crate for efficiently performing _fair_ join pattern matching i.e. the process of matching several
//! received messages in a mailbox against one of more patterns.
//! If we can find a combination of messages that matches a pattern and satisfies the guard,
//! the messages are consumed and the body of the match arm is executed.
//!
//! ## Fair and Deterministic Matching
//! `fairjax` implements join pattern matching in a fair and deterministic manner,
//! ensuring that we always match the oldest messages possible. Furthermore this ensures completely
//! deterministic and reproducible behavior.
//!
//! ## Example
//! ```
//! // Define message types
//! pub enum Msg {
//!     A(usize, f64),
//!     B(usize, f64),
//! }
//!
//! use Msg::*;
//!
//! // Declare state-keeping mailbox
//! let mut mailbox = fairjax_core::MailBox::default();
//!
//! // Simulate input messages
//! let messages = vec![
//!     A(0, 1.4),
//!     B(0, 1.8),
//!     B(1, 5.2),
//!     A(2, 42.1),
//!     A(1, 5.9),
//!     B(2, 42.5)
//! ];
//!
//! let mut matches = vec![];
//!
//! // Recieve message one by one
//! for msg in messages {
//!     // Declare join pattern matching
//!     fairjax::fairjax!(match msg >> [mailbox, Msg] {
//!         (A(id, ts1), B(id, ts2)) if ts1 < ts2 => {
//!             matches.push(id);
//!        }
//!    });
//!}
//!
//! assert_eq!(vec![0, 2], matches);
//! ```

///////////////////////////////////////////////////////////

// Allow macro code-gen tests increased recursion depth
#![cfg_attr(test, recursion_limit = "256")]

mod analyse {
    pub mod bundle;
    pub mod content;
    pub mod definition;
    pub mod groups;
    pub mod partition;
    pub mod strategy;
}

mod compile {
    pub mod case {
        pub mod accept;
        pub mod action;
        pub mod guard;
    }
    pub mod matchers;
    pub mod sections {
        pub mod action;
        pub mod setup;
    }
    pub mod pattern {
        pub mod full;
        pub mod sub;
    }

    pub mod top;
}

mod parse {
    pub mod case;
    pub mod context;
    pub mod definition;
    pub mod pattern;
    pub mod strategy;
    pub mod sub_pattern;
}

mod manager;
mod traits;

use crate::compile::sections::{action::ActionSection, setup::SetupSection};
use crate::compile::top::TopLevelCodeGen;
use crate::manager::FairjaxManagerDefinition;

#[proc_macro]
pub fn fairjax(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse::definition::RawJoinDefinition::parse(input.into()) {
        Ok(def) => match analyse::definition::JoinDefinition::analyse(def) {
            Ok(analysed_def) => {
                compile::top::TopLevel::generate::<ActionSection, SetupSection>(&analysed_def)
                    .into()
            }
            Err(e) => e.to_compile_error().into(),
        },
        Err(e) => return e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn fairjax_switch(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match FairjaxManagerDefinition::parse(input.into()) {
        Ok(def) => def.generate().into(),
        Err(e) => e.to_compile_error().into(),
    }
}
