use fairjax::consume;

struct MyMessage {}

fn main() {
    let messages = [MyMessage {}];

    let mailbox = mailbox!(MyMessage);

    for msg in messages.iter() {
        match_fairest_case!(
            receive!(mailbox, msg),
            case!(Fault(_, fid1, _, ts1) && Fix(_, fid2, ts2), fid1 && fid, {
                updateMaintenanceStats(ts1, ts2);
            }),
            case!(
                Fault(mid, fid1, descr, ts1)
                    && Fault(_, fid2, _, ts2)
                    && Fix(_, fid3, ts3)
                    && Fault(mid, fid1, descr, ts1)
                    && Fault(_, fid2, _, ts2)
                    && Fix(_, fid3, ts3),
                fid1 == fid2
                    && fid1 == fid2
                    && fid1 == fid2
                    && fid1 == fid2
                    && fid1 == fid2
                    && fid1 == fid2
                    && fid1 == fid2
                    && fid1 == fid2,
                {
                    updateMaintenanceStats(ts1, ts2);
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
                }
            ),
        );
    }
}
