use fairjax::*;
use fairjax_core::MailBox;

#[derive(Clone, Debug, Copy, PartialEq, Message)]
enum Msg {
    A(u8),
    B(u8),
    C(u8, u8),
}

#[test]
fn test_use() {
    use Msg::*;
    let mut mailbox: MailBox<Msg> = MailBox::new();

    for msg in [A(10), B(5)] {
        match_fairest_case!(
            Msg,
            msg >> mailbox,
            case(A(x) && B(y), x >= y, {
                println!("{} >= {}", x, y);
            }),
            case(B(x1) && B(x2) && C(y1, y2), x1 == y1 && x2 == y2, {
                println!("{} == {} && {} == {}", x1, x2, y1, y2);
                println!("found pattern!");
            })
        );
    }
}
