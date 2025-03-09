check:
    #!/bin/bash -eux
    cargo check --all-targets
    cargo check --all-features --all-targets
    cargo hack --each-feature --exclude-features=default,full check

    # can't use cargo-nextest because we want to run doctests
    cargo test -F full

    pushd zerodeps-example
    cargo check
    cargo check --features=merde
    cargo tree --prefix none --no-dedupe | grep -v merde-core
    cargo tree --prefix none --no-dedupe --features=merde | grep merde_core
    popd

    pushd merde
    EXAMPLES=()
    for file in examples/*.rs; do
      EXAMPLES+=($(basename "${file}" .rs))
    done
    for example in "${EXAMPLES[@]}"; do
      cargo run --features full,ahash --example "$example"
    done
    popd

    just miri

miri:
    rustup +nightly component add miri
    cargo +nightly miri run --example opinions -F deserialize,json
    cargo +nightly miri test -p merde_core fieldslot
