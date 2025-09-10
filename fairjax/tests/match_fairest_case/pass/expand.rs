use fairjax::consume;

struct MyMessage {}

fn main() {
    let messages = [MyMessage {}];

    let mailbox = MailBox::<MyMessage>::new(); // includes list of boxed dyn Patterns

    for msg in messages.iter() {
        match_fairest_case!(
            msg,
            mailbox,
            {
                [Fault(_, fid1, _, ts1), Fix(_, fid2, ts2)],{fid1 && fid},{updateMaintenanceStats(ts1, ts2);}
            },
            {
                [Fault(_, fid1, _, ts1), Fix(_, fid2, ts2)],{fid1 && fid},{updateMaintenanceStats(ts1, ts2);}
            }
        );

        if mailbox.is_not_init() {
            mailbox.add_pattern(
                // Generate pattern struct for pattern 0
            );
            mailbox.add_pattern(
                // Generate pattern struct for pattern 1
            );
            mailbox.lock();
        }

        // receive step
        if let Some(pattern_id, message_vec) = mailbox.receive(msg.clone()) {
            match pattern_id {
                0 => {
                    let input = (message_vec[0], message_vec[1]);
                    match input {
                        (Fault(_, fid1, _, ts1), Fix(_, fid2, ts2)) => {
                            updateMaintenanceStats(ts1, ts2);
                        }
                    }
                }
                1 => {
                    let input = (message_vec[0], message_vec[1]);
                    match input {
                        (Fault(_, fid1, _, ts1), Fix(_, fid2, ts2)) => {
                            updateMaintenanceStats(ts1, ts2);
                        }
                    }
                }
            }
        }
    }
}
