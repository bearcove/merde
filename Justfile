check:
    #!/bin/bash -eux
    cargo test -F full

    pushd ./merde_json_types
    cargo check -F time-types
    cargo check -F time-serialize
    cargo check -F time-deserialize
    cargo check -F merde_json,time-types
    cargo check -F merde_json,time-serialize
    cargo check -F merde_json,time-deserialize
    popd

    pushd ./merde_json
    cargo run --example simple
    cargo run --example mixed
    cargo run --example to_static
    popd
