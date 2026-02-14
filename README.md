# Post-Push Party 🎉

Throw a little party every time you push code! Earn points when you do! Spend points to upgrade your party! Spend points to unlock bonuses in order to earn more points! Keep pushing code, keep earning points, keep partying!


## How It Works

1. [Install](README#Installation) the app 

2. In your git/jj repo of choice, run `party init` to install either a git hook or a local `jj push` alias. 

3. Push code! You'll automatically start earning points based on how many commits you push.

4. Run `party` to open the Post-Push Party TUI where you can spend your points.


## Installation

### Prebuilt Binaries

Check out the [releases page](https://github.com/drewzemke/post-push-party/releases) for a prebuilt binary for your OS.

### Using `cargo`

Install [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html), then run:

```shell
cargo install post-push-party
```

### Using Homebrew

If you're on macOS, install [Homebrew](https://brew.sh/) and then run:

```shell
brew install drewzemke/tap/post-push-party
```


## Coming Soon

- *Color!* Unlock color palettes and apply your favorite ones to your party.

- *Packs!* Buy packs with points. Packs contain color palettes to beautify your party, mini-game tokens, maybe even more points!

- *Mini-games!* Play a round of snake while you're waiting for your tests to run. Earn points! Or have a go at the slot machine. Earn (or lose) points!
