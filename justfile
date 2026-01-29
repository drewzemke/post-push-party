cheat n:
    @cargo run --quiet --features dev -- cheat {{n}}

push n:
    @cargo run --quiet --features dev -- push {{n}}

reset:
    @cargo run --quiet --features dev -- reset

unlock track level:
    @cargo run --quiet --features dev -- unlock {{track}} {{level}}

install:
    cargo install --path .
