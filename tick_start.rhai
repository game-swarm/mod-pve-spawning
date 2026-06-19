// tick_start.rhai — pve-spawning
// Executed at the start of each tick, before combat/spawning
// Manages NPC spawning with difficulty zones, spawn rates, and grace periods
//
// state:  read-only world snapshot (global view, no fog_of_war)
// actions: write buffer (deduct_resource, add_resource, spawn_npc, etc.)

// Iterate all players and their rooms
for player in state.players() {
    for room in player.rooms() {
        for drone in room.drones() {
            // Mod-specific logic here
        }
    }
}
