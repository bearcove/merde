check:
    #!/bin/bash -eux
    cargo test -F full
    cargo hack --feature-powerset check
