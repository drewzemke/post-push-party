pub mod party {
    pub mod fireworks {
        mod renderer;
        mod runner;
        mod sim;

        pub use runner::run;
    }
}
