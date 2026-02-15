# adds or subtracts n party points in local state
cheat n:
    @cargo run --quiet --package post-push-party --features dev -- cheat {{n}}

# simulates pushing n commits
push n:
    @cargo run --quiet --package post-push-party --features dev -- push {{n}}

# resets local game state
reset:
    @cargo run --quiet --package post-push-party --features dev -- reset

# unlocks a bonus track to a given level
bonus track level:
    @cargo run --quiet --package post-push-party --features dev -- bonus {{track}} {{level}}

# unlocks a party by id
party id:
    @cargo run --quiet --package post-push-party --features dev -- party {{id}}

# installs app to local machine
install:
    cargo install --path app

# tests fireworks party
fireworks:
    @cargo run --package fireworks
