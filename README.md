# Texas Hold'Soul

Multiplayer Texas Hold'em playable in the browser, a personal project to learn Rust, Leptos and WebSockets.

Not finished yet, but 100% playable.

## Tech stack

- Frontend: Leptos, WebAssembly, TailwindCSS, served by Nginx
- Backend: Rust (Axum, Tokio), WebSockets
- Shared `shared` crate between the two (game and message types)
- Deployed with Docker Compose

## Features

- Room creation and management
- Real-time bidirectional communication over WebSockets (blinds, hands, turn, pot)
- Actions: fold, check, call, raise, all-in
- Starting bankroll: 200 chips, up to 9 players per room

## How to play

- Create or join a room with at least 2 players
- Pick a username and start the game
- The game starts instantly, no time limit (for now)
- You need to create a new room to play another game (coming improvement)

## How to run

Docker Compose:

```bash
docker compose up -d
```

- Frontend: http://localhost:8080
- Backend (API + WebSocket `/ws`): http://localhost:3001
- Health check: http://localhost:3001/health

Local development:

```bash
cd backend
cargo run
```

```bash
cd frontend
trunk serve --port 8080
```

## Online version

Available at [poker.bsoul.fr](https://poker.bsoul.fr).