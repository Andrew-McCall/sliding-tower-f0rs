# Sliding Tower

## Description
### Introduction
Sliding Tower is a small stacking game written in Rust for the Flipper Zero device platform. The goal is to drop moving blocks onto a tower and stack them as accurately as possible. Each successful placement increases your score. If blocks are misaligned, the game ends.

The project is designed to run in a minimal embedded Rust environment.

### My Implementation
The game uses a simple state machine to manage menu navigation, gameplay, pause, and exit states. The screen is rendered on a small 128×64 canvas, and input is handled through device buttons.

Core gameplay features:
- Moving block that slides horizontally
- Block drop timing challenge
- Tower stacking progression
- Basic scoring system

## Controls
### Navigation
- **OK Button (Press)** – Drop block while playing.
- **Back Button (Long Press)** – Quit the game.

### Game Flow
- Start from menu screen.
- Press OK to begin.
- Try to align the moving block with the tower.
- Game ends if placement fails.

## Building

### Requirements
- Rust toolchain supporting embedded targets.
- Flipper Zero firmware development environment.

### Cargo Build
```bash
cargo build --release
```

Compiled output:
```
target/thumbv7em-none-eabihf/release/
```

### Running
Install the compiled application onto the Flipper Zero device and launch it from the application menu.

## License
MIT License
Copyright (c) 2025
