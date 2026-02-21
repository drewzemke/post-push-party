// these defs are here so that examples work without having to turn
// the whole thing into a library
pub mod party {
    pub mod fireworks {
        mod renderer;
        mod runner;
        mod sim;

        pub use runner::run;
    }
}
