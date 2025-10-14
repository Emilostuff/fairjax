setup:
    # Initialize cargo-husky
    cargo test -r
    cargo install cargo-nextest --locked

expand_macro:
    cd test_suite && cargo expand --tests

test:
    cargo nextest run -r

test-heavy:
    cargo nextest run -r --run-ignored=only
