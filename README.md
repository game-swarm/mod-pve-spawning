# Swarm Mod: pve-spawning

PvE NPC spawning system — spawn rates, difficulty zones, and NPC type configuration for Swarm
int
string
string

## Directory Structure

```
mod-pve-spawning/
├── mod.toml          # Mod metadata + configurable parameters
├── init.rhai         # Executed once on load
├── tick_start.rhai   # Executed at start of each tick
├── tick_end.rhai     # Executed at end of each tick
└── README.md
```

## Configuration

See `mod.toml` for all configurable parameters. Server operators can override via:

```bash
swarm mod config pve-spawning <key> <value>
```

Or in `world.toml`:

```toml
[mods.pve-spawning.config]
# key = value
```

## Engine API

Mods interact with the engine through the `actions` interface:

- `actions.deduct_resource(player_id, resource, amount)`
- `actions.add_resource(player_id, resource, amount)`
- `actions.spawn_npc(room_id, npc_type, position)`
- `actions.log_info(msg)` / `actions.log_warn(msg)` / `actions.log_error(msg)`
- `actions.emit_event(event_type, data)`

## Publishing

```bash
git tag v0.1.0
git push --tags
swarm mod pack
```
