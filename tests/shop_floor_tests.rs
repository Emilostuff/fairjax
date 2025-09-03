use fairjax::definition::{Definition, JoinDefinition};
use fairjax::matchgroup::MatchGroup;
use fairjax::message::Message;
use fairjax::pattern::PatternMatcher;
use fairjax::permute::Element;
use fairjax::{BodyFn, GuardFn, MessageId};

#[derive(Clone, Debug, Copy, PartialEq)]
enum MyMessage {
    Fix { id: usize },
    Fault { id: usize, timestamp: usize },
}

impl MyMessage {
    fn fix(id: usize) -> Self {
        MyMessage::Fix { id }
    }

    fn fault(id: usize, timestamp: usize) -> Self {
        MyMessage::Fault { id, timestamp }
    }
}

impl Message for MyMessage {}

#[derive(Default, Clone, Debug)]
pub struct FaultFaultFix {
    fault: [Option<MessageId>; 2],
    fix: [Option<MessageId>; 1],
    counter: u8,
}

impl MatchGroup<MyMessage> for FaultFaultFix {
    fn extend(&self, message: &MyMessage, id: MessageId) -> Option<Self> {
        let mut new_group = self.clone();
        match message {
            MyMessage::Fix { .. } => {
                for i in 0..new_group.fix.len() {
                    if new_group.fix[i].is_none() {
                        new_group.fix[i] = Some(id);
                        new_group.counter += 1;
                        return Some(new_group);
                    }
                }
            }
            MyMessage::Fault { .. } => {
                for i in 0..new_group.fault.len() {
                    if new_group.fault[i].is_none() {
                        new_group.fault[i] = Some(id);
                        new_group.counter += 1;
                        return Some(new_group);
                    }
                }
            }
        }
        None
    }

    fn is_complete(&self) -> bool {
        self.counter >= 3
    }

    fn message_ids(&self) -> Vec<MessageId> {
        let mut ids = Vec::new();
        ids.extend(self.fault.iter().filter_map(|&x| x));
        ids.extend(self.fix.iter().filter_map(|&x| x));
        ids
    }

    fn to_elements(&self) -> Vec<Element> {
        vec![
            Element::new(self.fault[0].unwrap(), vec![0, 1]),
            Element::new(self.fault[1].unwrap(), vec![0, 1]),
            Element::new(self.fix[0].unwrap(), vec![2]),
        ]
    }
}

#[derive(Default, Clone, Debug)]
pub struct FaultFix {
    fault: [Option<MessageId>; 1],
    fix: [Option<MessageId>; 1],
    counter: u8,
}

impl MatchGroup<MyMessage> for FaultFix {
    fn extend(&self, message: &MyMessage, id: MessageId) -> Option<Self> {
        let mut new_group = self.clone();
        match message {
            MyMessage::Fix { .. } => {
                for i in 0..new_group.fix.len() {
                    if new_group.fix[i].is_none() {
                        new_group.fix[i] = Some(id);
                        new_group.counter += 1;
                        return Some(new_group);
                    }
                }
            }
            MyMessage::Fault { .. } => {
                for i in 0..new_group.fault.len() {
                    if new_group.fault[i].is_none() {
                        new_group.fault[i] = Some(id);
                        new_group.counter += 1;
                        return Some(new_group);
                    }
                }
            }
        }
        None
    }

    fn is_complete(&self) -> bool {
        self.counter >= 2
    }

    fn message_ids(&self) -> Vec<MessageId> {
        let mut ids = Vec::new();
        ids.extend(self.fault.iter().filter_map(|&x| x));
        ids.extend(self.fix.iter().filter_map(|&x| x));
        ids
    }

    fn to_elements(&self) -> Vec<Element> {
        vec![
            Element::new(self.fault[0].unwrap(), vec![0]),
            Element::new(self.fix[0].unwrap(), vec![1]),
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Response {
    pattern_id: usize,
    matched_messages: Vec<MyMessage>,
}

impl Response {
    fn new(pattern_id: usize, matched_messages: Vec<MyMessage>) -> Self {
        Response {
            pattern_id,
            matched_messages,
        }
    }
}

fn get_join_definition() -> JoinDefinition<MyMessage, Response> {
    // Guards
    let faultfaultfix_guard: GuardFn<MyMessage> =
        Box::new(
            |messages: &Vec<&MyMessage>| match (messages[0], messages[1], messages[2]) {
                (
                    MyMessage::Fault {
                        id: _,
                        timestamp: ts1,
                    },
                    MyMessage::Fault {
                        id: fid2,
                        timestamp: ts2,
                    },
                    MyMessage::Fix { id: fid3 },
                ) => fid2 == fid3 && *ts2 > *ts1 + 10,
                _ => unreachable!(),
            },
        );

    let faultfix_guard: GuardFn<MyMessage> =
        Box::new(
            |messages: &Vec<&MyMessage>| match (messages[0], messages[1]) {
                (
                    MyMessage::Fault {
                        id: fid1,
                        timestamp: _,
                    },
                    MyMessage::Fix { id: fid2 },
                ) => fid1 == fid2,
                _ => unreachable!(),
            },
        );

    // Bodies
    let faultfix_body: BodyFn<MyMessage, Response> = Box::new(|msg: &Vec<&MyMessage>| {
        Some(Response {
            pattern_id: 0,
            matched_messages: msg.iter().map(|m| *m.to_owned()).collect(),
        })
    });
    let faultfaultfix_body: BodyFn<MyMessage, Response> = Box::new(|msg: &Vec<&MyMessage>| {
        Some(Response {
            pattern_id: 1,
            matched_messages: msg.iter().map(|m| *m.to_owned()).collect(),
        })
    });

    // Create patterns
    let faultfaultfix = PatternMatcher::<FaultFaultFix, MyMessage, Response>::new(
        faultfaultfix_guard,
        faultfaultfix_body,
    );
    let faultfix =
        PatternMatcher::<FaultFix, MyMessage, Response>::new(faultfix_guard, faultfix_body);

    // Return join definition
    JoinDefinition::new(vec![Box::new(faultfix), Box::new(faultfaultfix)])
}

fn run(messages: Vec<MyMessage>, expected_responses: Vec<Response>) {
    let mut def = get_join_definition();
    let mut output = Vec::new();

    for message in messages {
        if let Some(response) = def.consume(message) {
            output.push(response);
        }
    }

    assert_eq!(expected_responses, output);
}

// TESTS

#[test]
fn test_fault_fault_fix() {
    // Messages
    let messages = vec![
        MyMessage::fault(1, 1035),
        MyMessage::fault(2, 1039),
        MyMessage::fault(3, 1056),
        MyMessage::fix(3),
        MyMessage::fix(2),
    ];

    // Output
    let expected = vec![
        Response::new(
            1,
            vec![
                MyMessage::fault(1, 1035),
                MyMessage::fault(3, 1056),
                MyMessage::fix(3),
            ],
        ),
        Response::new(0, vec![MyMessage::fault(2, 1039), MyMessage::fix(2)]),
    ];

    run(messages, expected);
}
