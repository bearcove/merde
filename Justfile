check:
    #!/bin/bash -eux
    cargo check --example simple --no-default-features --features=json
    cargo run --example simple --features=core,json
    cargo check --no-default-features
    cargo hack --feature-powerset check
    cargo test -F full

    pushd zerodeps-example
    cargo check
    cargo check --features=merde
    cargo tree --prefix none --no-dedupe | grep -v compact_str
    cargo tree --prefix none --no-dedupe --features=merde | grep compact_str
    popd
