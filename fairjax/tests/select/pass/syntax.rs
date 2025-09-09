use fairjax::consume;

struct MyMessage {}

fn main() {
    let messages = [MyMessage {}];

    //let matcher = matcher!(MyMessage);

    for msg in messages.iter() {
        consume!(msg => matcher {
            (Fault(_, fid1, _, ts1), Fix(_, fid2, ts2)) if fid1 == fid2 => {
                updateMaintenanceStats(ts1, ts2);
                Continue
            },
            (Fault(mid, fid1, descr, ts1), Fault(_, fid2, _, ts2), Fix(_, fid3, ts3)) if fid2 == fid3 && ts2 > ts1 + TEN_MIN => {
                updateMaintenanceStats(ts2, ts3);
                Continue
            },
            (DelayedFault(_, fid1, _, ts1), Fix(_, fid2, ts2)) if fid1 == fid2 => {
                updateMaintenanceStats(ts1, ts2);
                Continue
            },
            (Shutdown()) => {
                Stop
            }
        });
    }
}
