# TUI Architecture

## Overview

The TUI is built with ratatui and follows a standard view-based architecture with clear separation between state, events, and rendering.

## Directory Structure

```
src/tui/
├── mod.rs              # entry point, terminal setup/teardown
├── app.rs              # App state, main loop, tab routing
├── event.rs            # KeyEvent → Action mapping
├── action.rs           # Action enum (Navigate, Select, Back, etc.)
│
├── views/
│   ├── mod.rs          # View trait definition
│   ├── store/
│   │   ├── mod.rs      # StoreView (grid selection state)
│   │   ├── grid.rs     # Grid selection screen
│   │   ├── upgrades.rs # Upgrades list sub-page
│   │   └── bonuses.rs  # Bonuses tier sub-page
│   ├── party.rs        # PartyView (feature toggles)
│   ├── packs.rs        # PacksView (stub)
│   └── games.rs        # GamesView (stub)
│
└── widgets/
    ├── mod.rs          # widget exports
    ├── card.rs         # bordered card with title + content
    ├── header.rs       # "POST-PUSH PARTY 🎉" + tabs
    └── footer.rs       # key hints + point balance
```

## Core Abstractions

### View Trait

```rust
pub trait View {
    fn render(&self, frame: &mut Frame, area: Rect, state: &State);
    fn handle(&mut self, action: Action, state: &mut State) -> ViewResult;
    fn key_hints(&self) -> Vec<(&str, &str)>;
}
```

### Action Enum

Decouples keyboard input from behavior:

```rust
pub enum Action {
    Up, Down, Left, Right,
    Select,      // Enter/Space
    Back,        // Esc
    Tab(usize),  // Jump to tab 1-4
    Quit,
}
```

### ViewResult

Tells App what to do after handling an action:

```rust
pub enum ViewResult {
    None,
    Redraw,
    Navigate(Route),
    Message(String),
    Exit,
}
```

### Route

Navigation targets:

```rust
pub enum Route {
    Store(StoreRoute),
    Party,
    Packs,
    Games,
}

pub enum StoreRoute {
    Grid,       // category selection
    Upgrades,   // party feature unlocks
    Bonuses,    // bonus tier upgrades
    Packs,      // pack purchasing (future)
    Games,      // game unlocks (future)
}
```

## App Structure

```rust
pub struct App {
    route: Route,
    message: Option<String>,

    // view states persist across navigation
    store: StoreView,
    party: PartyView,
    packs: PacksView,
    games: GamesView,
}
```

Each view owns its selection/scroll state so returning to a tab resumes where you left off.

## Event Flow

```
KeyEvent (crossterm)
    │
    ▼
event.rs (key mapping)
    │
    ▼
Action
    │
    ▼
View.handle()
    │
    ▼
ViewResult
    │
    ▼
App (updates route, message, triggers redraw)
```

## State Management

**Game state** (`src/state.rs`) - persisted to disk:
- party_points, commit_value_level
- unlocked_features, enabled_features
- bonuses HashMap

**UI state** (`app.rs`) - TUI-only, not persisted:
- Current route
- Per-view selection indices
- Transient messages

Views read game state via `&State` and mutate via `&mut State`. App calls `state::save()` after mutations.

## Screen Layout (from Figma)

All screens share this structure:

```
┌─────────────────────────────────────────────────────────┐
│ POST-PUSH PARTY 🎉                                      │  <- header
├─────────────────────────────────────────────────────────┤
│ [1] Store   [2] Party   [3] Packs   [4] Games           │  <- tabs
├─────────────────────────────────────────────────────────┤
│                                                         │
│                    (content area)                       │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ (key hints)                                      150 P  │  <- footer
└─────────────────────────────────────────────────────────┘
```

## Store Tab Screens

### Grid Selection (default)

2x2 grid of categories:

```
┌─────────────────┐  ┌─────────────────┐
│ Party Upgrades  │  │ Bonuses         │
│ (description)   │  │ (description)   │
└─────────────────┘  └─────────────────┘
┌─────────────────┐  ┌─────────────────┐
│ Packs           │  │ Games           │
│ (description)   │  │ (description)   │
└─────────────────┘  └─────────────────┘
```

Arrow keys navigate, Enter drills into sub-page.

### Upgrades Sub-page

List of party features to unlock:

```
                    Upgrades
┌─────────────────────────────────────────────────────────┐
│ Exclamation                                      100 P  │
│ Adds an excited shout to your party.                    │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│ Big Text                                         500 P  │
│ Finish your party with a full screen word. NICE!        │
└─────────────────────────────────────────────────────────┘
```

### Bonuses Sub-page

Tier-based upgrades with horizontal level selector:

```
                    Bonuses
┌─────────────────────────────────────────────────────────┐
│ Commit Value                                            │
│ How many party points you get per commit.               │
│   [1]    [2]    [3]    [4]    [5]                       │
│    ✓      ✓    100 P   1K P   10K P                     │
└─────────────────────────────────────────────────────────┘
```

## Party Tab

Configure unlocked features:

```
┌─────────────────────────────────────────────────────────┐
│ Basic Party                                             │
│ A simple summary of how many points you earned.         │
│ ✓ Enabled                                               │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│ Exclamation                                             │
│ An excited message to hype up your party.               │
│ ✗ Disabled                                              │
└─────────────────────────────────────────────────────────┘
```
