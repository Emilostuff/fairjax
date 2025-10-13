use fairjax_core::GuardEval;
use fairjax_core::Message;
use fairjax_core::MessageId;
use fairjax_core::mailbox::MailBox;
use fairjax_core::stateful_tree::{PartialMatch, StatefulTreeMatcher, permute::Element};

#[derive(Clone, Debug, Copy, PartialEq)]
enum Msg {
    Fix { id: usize },
    Fault { id: usize, timestamp: usize },
}

impl Msg {
    fn fix(id: usize) -> Self {
        Msg::Fix { id }
    }

    fn fault(id: usize, timestamp: usize) -> Self {
        Msg::Fault { id, timestamp }
    }
}

impl Message for Msg {}

#[derive(Default, Clone, Debug)]
pub struct FaultFaultFix {
    messages: [Option<MessageId>; 3],
    counter: u8,
}

impl PartialMatch<Msg> for FaultFaultFix {
    fn extend(&self, message: &Msg, id: MessageId) -> Option<Self> {
        let mut new_group = self.clone();
        let (i, j) = match message {
            Msg::Fault { .. } => (0, 2),
            Msg::Fix { .. } => (2, 3),
        };

        for slot in &mut new_group.messages[i..j] {
            if slot.is_none() {
                *slot = Some(id);
                new_group.counter += 1;
                return Some(new_group);
            }
        }
        None
    }

    fn is_complete(&self) -> bool {
        self.counter >= 3
    }

    fn message_ids(&self) -> Vec<MessageId> {
        self.messages.iter().filter_map(|x| *x).collect()
    }

    fn to_elements(&self) -> Vec<Element> {
        vec![
            Element::new(self.messages[0].unwrap(), vec![0, 1]),
            Element::new(self.messages[1].unwrap(), vec![0, 1]),
            Element::new(self.messages[2].unwrap(), vec![2]),
        ]
    }
}

#[derive(Default, Clone, Debug)]
pub struct FaultFix {
    messages: [Option<MessageId>; 2],
    counter: u8,
}

impl PartialMatch<Msg> for FaultFix {
    fn extend(&self, message: &Msg, id: MessageId) -> Option<Self> {
        let mut new_group = self.clone();
        let (i, j) = match message {
            Msg::Fault { .. } => (0, 1),
            Msg::Fix { .. } => (1, 2),
        };

        for slot in &mut new_group.messages[i..j] {
            if slot.is_none() {
                *slot = Some(id);
                new_group.counter += 1;
                return Some(new_group);
            }
        }
        None
    }

    fn is_complete(&self) -> bool {
        self.counter >= 2
    }

    fn message_ids(&self) -> Vec<MessageId> {
        self.messages.iter().filter_map(|x| *x).collect()
    }

    fn to_elements(&self) -> Vec<Element> {
        vec![
            Element::new(self.messages[0].unwrap(), vec![0]),
            Element::new(self.messages[1].unwrap(), vec![1]),
        ]
    }
}

fn get_join_definition() -> MailBox<Msg> {
    // Guards
    fn faultfaultfix_guard(messages: &Vec<&Msg>) -> GuardEval {
        match (messages[0], messages[1], messages[2]) {
            (
                Msg::Fault {
                    id: _,
                    timestamp: ts1,
                },
                Msg::Fault {
                    id: fid2,
                    timestamp: ts2,
                },
                Msg::Fix { id: fid3 },
            ) => {
                if fid2 == fid3 && *ts2 > *ts1 + 10 {
                    return GuardEval::True;
                } else {
                    return GuardEval::False;
                }
            }
            _ => unreachable!(),
        }
    }

    fn faultfix_guard(messages: &Vec<&Msg>) -> GuardEval {
        match (messages[0], messages[1]) {
            (
                Msg::Fault {
                    id: fid1,
                    timestamp: _,
                },
                Msg::Fix { id: fid2 },
            ) => {
                if fid1 == fid2 {
                    return GuardEval::True;
                } else {
                    return GuardEval::False;
                }
            }
            _ => unreachable!(),
        }
    }

    // Create patterns
    let faultfaultfix = StatefulTreeMatcher::<FaultFaultFix, Msg>::new(faultfaultfix_guard);
    let faultfix = StatefulTreeMatcher::<FaultFix, Msg>::new(faultfix_guard);

    // Return join definition
    let mut mailbox = MailBox::new();
    mailbox.add_case(Box::new(faultfix));
    mailbox.add_case(Box::new(faultfaultfix));
    mailbox
}

fn run(messages: Vec<Msg>, expected_responses: Vec<(usize, Vec<Msg>)>) {
    let mut def = get_join_definition();
    let mut output = Vec::new();

    for message in messages {
        if let Some(res) = def.consume(message) {
            output.push(res);
        }
    }

    assert_eq!(expected_responses, output);
}

// TESTS

#[test]
fn test_fault_fault_fix() {
    // Messages
    let m = vec![
        Msg::fault(1, 1035),
        Msg::fault(2, 1039),
        Msg::fault(3, 1056),
        Msg::fix(3),
        Msg::fix(2),
    ];

    // Output
    let expected = vec![(1, vec![m[0], m[2], m[3]]), (0, vec![m[1], m[4]])];

    run(m, expected);
}

#[test]
fn test_fault_fix() {
    // Messages
    let m = vec![
        Msg::fault(1, 1035),
        Msg::fault(2, 1039),
        Msg::fault(3, 1042),
        Msg::fix(3),
    ];

    // Output
    let expected = vec![(0, vec![m[2], m[3]])];

    run(m, expected);
}
