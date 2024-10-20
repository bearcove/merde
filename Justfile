check:
    #!/bin/bash -eux
    cargo check --all-targets
    cargo check --all-features --all-targets
    cargo hack --each-feature --exclude-features=default,full check

    cargo check --example simple --no-default-features --features=json
    cargo run --example simple --features=core,json

    # can't use cargo-nextest because we want to run doctests
    cargo test -F full

    pushd zerodeps-example
    cargo check
    cargo check --features=merde
    cargo tree --prefix none --no-dedupe | grep -v merde-core
    cargo tree --prefix none --no-dedupe --features=merde | grep merde_core
    popd

    pushd merde
    EXAMPLES=($(cd examples && for i in *; do echo "${i%.rs}"; done))
    for example in "${EXAMPLES[@]}"; do
      cargo run --features full,ahash --example "$example"
    done
    popd
