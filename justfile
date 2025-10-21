setup:
    # Initialize cargo-husky
    cargo clean
    cargo test -r
    cargo install cargo-nextest --locked

expand_macro:
    cd test_suite && cargo expand --tests

test arg="":
    cargo nextest run -r {{arg}}

test-heavy:
    cargo nextest run -r --run-ignored=only

baseline:
    for f in test_suite/benches/*.rs; do \
        name=$(basename "$f" .rs); \
        echo "===> Setting baseline for: $name"; \
        cargo bench --bench "$name" -- --save-baseline baseline; \
    done

bench benchname="":
    if [ -n "{{benchname}}" ]; then \
        echo "===> Benchmarking: {{benchname}}"; \
        cargo bench --bench "{{benchname}}" -- --baseline baseline; \
    else \
        for f in test_suite/benches/*.rs; do \
            name=$(basename "$f" .rs); \
            echo "===> Benchmarking: $name"; \
            cargo bench --bench "$name" -- --baseline baseline; \
        done; \
    fi
