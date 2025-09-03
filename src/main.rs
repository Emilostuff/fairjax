use fairjax::definition::{Definition, JoinDefinition};
use fairjax::matchgroup::MatchGroup;
use fairjax::message::Message;
use fairjax::pattern::PatternMatcher;
use fairjax::permute::Element;
use fairjax::{BodyFn, GuardFn, MessageId};

///////////////////////////// User specified

#[derive(Clone, Debug)]
enum MyMessage {
    Fix { id: usize },
    Fault { id: usize, timestamp: usize },
}

impl Message for MyMessage {}

///////////////////////////// To be generated

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

fn main() {
    // Messages
    let messages = [
        MyMessage::Fault {
            id: 1,
            timestamp: 1035,
        },
        MyMessage::Fault {
            id: 2,
            timestamp: 1039,
        },
        MyMessage::Fault {
            id: 3,
            timestamp: 1056,
        },
        MyMessage::Fix { id: 3 },
    ];

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
    let faultfaultfix_body: BodyFn<MyMessage> = Box::new(|_: &Vec<&MyMessage>| None);
    let faultfix_body: BodyFn<MyMessage> = Box::new(|_: &Vec<&MyMessage>| None);

    // Create patterns
    let faultfaultfix =
        PatternMatcher::<FaultFaultFix, MyMessage>::new(faultfaultfix_guard, faultfaultfix_body);
    let faultfix = PatternMatcher::<FaultFix, MyMessage>::new(faultfix_guard, faultfix_body);

    // Create join definition
    let mut def = JoinDefinition::new(vec![Box::new(faultfix), Box::new(faultfaultfix)]);

    // test
    for message in messages {
        println!("\n{:?}:", message);
        def.consume(message);
    }
}
