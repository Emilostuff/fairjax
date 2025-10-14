setup:
    # Initialize cargo-husky
    cargo test -r
    cargo install cargo-nextest --locked

expand_macro:
    cd test_suite && cargo expand --tests

test arg="":
    cargo nextest run -r {{arg}}

test-heavy:
    cargo nextest run -r --run-ignored=only
