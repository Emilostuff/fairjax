use fairjax::Message;

#[derive(Debug, Clone, Message)]
struct MyMessage;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_struct() {
        let message = MyMessage;

        fn check_generic<T: fairjax_core::Message>(_msg: T) {}

        check_generic(message);
    }
}
