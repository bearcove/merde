check:
    #!/bin/bash -eux
    cargo check --example simple --no-default-features --features=json
    cargo run --example simple --features=core,json
    cargo hack --feature-powerset --exclude-features=default,full check -p merde
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
