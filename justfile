cheat n:
    @cargo run --quiet --features dev -- cheat {{n}}

push n:
    @cargo run --quiet --features dev -- push {{n}}

reset:
    @cargo run --quiet --features dev -- reset

bonus track level:
    @cargo run --quiet --features dev -- bonus {{track}} {{level}}

party id:
    @cargo run --quiet --features dev -- party {{id}}

install:
    cargo install --path .
