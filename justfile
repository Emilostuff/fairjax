setup:
    # Initialize cargo-husky
    cargo test

expand_macro:
    cd test_suite && cargo expand --tests
