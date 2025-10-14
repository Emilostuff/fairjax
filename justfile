setup:
    # Initialize cargo-husky
    cargo test

expand_macro:
    cd test_suite && cargo expand --tests

test:
    cargo test -r

test-big:
    cargo test -r -- --ignored --nocapture
