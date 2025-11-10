setup:
    # Initialize cargo-husky
    cargo clean
    cargo test -r
    cargo install cargo-nextest --locked

expand_macro:
    cd test_suite && cargo expand --tests

clear:
    printf '\033c'

test arg="":
    cargo nextest run -r {{arg}}

test-heavy:
    cargo nextest run -r --run-ignored=only

baseline benchname="":
    @if [ -n "{{benchname}}" ]; then \
        echo "\033[1mSetting baseline for: {{benchname}}\033[0m"; \
        cargo bench --bench "{{benchname}}" -- --save-baseline baseline; \
    else \
        for f in test_suite/benches/*.rs; do \
            name=$(basename "$f" .rs); \
            echo "\033[1mSetting baseline for: $name\033[0m"; \
            cargo bench --bench "$name" -- --save-baseline baseline; \
        done; \
    fi


bench benchname="":
    @if [ -n "{{benchname}}" ]; then \
    echo "\033[1mBenchmarking: {{benchname}}\033[0m"; \
        cargo bench --bench "{{benchname}}" -- --baseline baseline; \
    else \
        for f in test_suite/benches/*.rs; do \
            name=$(basename "$f" .rs); \
            echo "\033[1mBenchmarking: $name\033[0m"; \
            cargo bench --bench "$name" -- --baseline baseline; \
        done; \
    fi

example name:
    cargo r -p fairjax --example {{name}}
