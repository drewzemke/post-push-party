# adds or subtracts n party points in local state
cheat n:
    @cargo run --quiet --features dev -- cheat {{n}}

# simulates pushing n commits with m lines each
push n m="10":
    @cargo run --quiet --features dev -- push {{n}} --lines {{m}}

# resets local game state
reset:
    @cargo run --quiet --features dev -- reset

# unlocks a bonus track to a given level
bonus track level:
    @cargo run --quiet --features dev -- bonus {{track}} {{level}}

# unlocks a party by id
party id:
    @cargo run --quiet --features dev -- party {{id}}

# unlocks all palettes for a party (or "all")
palette id:
    @cargo run --quiet --features dev -- palette {{id}}

# installs app to local machine
install:
    cargo install --path .

# tests fireworks party
fireworks:
    @cargo run --example fireworks
