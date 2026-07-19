use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use swarm_engine_api::prelude::{
    API_VERSION, BodyPart, ConfigFieldDescriptor, ConfigValidator, ConfigValueType,
    DESCRIPTOR_SCHEMA_VERSION, PlayerId, PluginDescriptor, RoomId, SystemDescriptor, TickPhase,
};
use swarm_engine_plugin_sdk::components::{BodyPartRegistry, Drone, Position, Resource};
use swarm_engine_plugin_sdk::traits::SwarmPlugin;

pub const NPC_OWNER: PlayerId = 0;

#[derive(Component, Debug, Clone)]
pub struct RoomConfig {
    pub room: RoomId,
    pub spawn_origin: Position,
    pub pve_enabled: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct WorldConfig {
    pub pve_only: bool,
    pub pvp_enabled: bool,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            pve_only: true,
            pvp_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcKind {
    Neutral,
    ResourceWave,
    Elite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcState {
    Guard,
    Patrol,
    Chase,
    Return,
}

#[derive(Component, Debug, Clone)]
pub struct NpcAI {
    pub kind: NpcKind,
    pub state: NpcState,
    pub home: Position,
    pub target: Option<Entity>,
    pub drop_table: BTreeMap<String, u32>,
}

#[derive(Resource, Debug, Clone)]
pub struct PveSpawningConfig {
    pub spawn_interval: u32,
    pub max_npcs_per_room: u32,
    pub npc_drone_body: Vec<BodyPart>,
    pub npc_drop_table: BTreeMap<String, u32>,
}

impl Default for PveSpawningConfig {
    fn default() -> Self {
        Self {
            spawn_interval: 300,
            max_npcs_per_room: 50,
            npc_drone_body: vec![BodyPart::Attack, BodyPart::Move, BodyPart::Move],
            npc_drop_table: BTreeMap::from([("Energy".to_string(), 50)]),
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct PveSpawnClock {
    pub tick: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PveSpawningModPlugin;

impl Plugin for PveSpawningModPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldConfig>()
            .init_resource::<PveSpawningConfig>()
            .init_resource::<PveSpawnClock>()
            .add_systems(
                Update,
                (pve_spawn_system, npc_ai_system, pve_drop_system).chain(),
            );
    }
}

impl SwarmPlugin for PveSpawningModPlugin {
    fn descriptor() -> PluginDescriptor {
        PluginDescriptor {
            id: "pve-spawning".to_string(),
            version: "0.1.0".to_string(),
            api_version: API_VERSION.to_string(),
            dependencies: Vec::new(),
            config: vec![
                ConfigFieldDescriptor {
                    key: "spawn_interval".to_string(),
                    value_type: ConfigValueType::U32,
                    default: 300_u32.into(),
                    required: false,
                    validator: Some(ConfigValidator::Positive),
                },
                ConfigFieldDescriptor {
                    key: "max_npcs_per_room".to_string(),
                    value_type: ConfigValueType::U32,
                    default: 50_u32.into(),
                    required: false,
                    validator: Some(ConfigValidator::Positive),
                },
                ConfigFieldDescriptor {
                    key: "npc_drone_body".to_string(),
                    value_type: ConfigValueType::Array {
                        item_type: "BodyPart".to_string(),
                    },
                    default: serde_json::json!(["Attack", "Move", "Move"]),
                    required: false,
                    validator: Some(ConfigValidator::NonEmptyArray),
                },
                ConfigFieldDescriptor {
                    key: "npc_drop_table".to_string(),
                    value_type: ConfigValueType::Map {
                        key_type: "Resource".to_string(),
                        value_type: "U32".to_string(),
                    },
                    default: serde_json::json!({ "Energy": 50 }),
                    required: false,
                    validator: None,
                },
            ],
            systems: vec![
                SystemDescriptor {
                    system_id: "pve-spawning.spawn".to_string(),
                    version: "0.1.0".to_string(),
                    phase: TickPhase::Update,
                    order: 0,
                    reads: vec![
                        "PveSpawningConfig".to_string(),
                        "WorldConfig".to_string(),
                        "RoomConfig".to_string(),
                        "NpcAI".to_string(),
                        "Position".to_string(),
                    ],
                    writes: vec![
                        "PveSpawnClock".to_string(),
                        "EntityLifecycle".to_string(),
                        "Drone".to_string(),
                        "Position".to_string(),
                        "NpcAI".to_string(),
                    ],
                    produces_buffers: Vec::new(),
                    consumes_buffers: Vec::new(),
                    deterministic_iteration: vec!["RoomId".to_string()],
                },
                SystemDescriptor {
                    system_id: "pve-spawning.npc-ai".to_string(),
                    version: "0.1.0".to_string(),
                    phase: TickPhase::Update,
                    order: 1,
                    reads: vec!["Drone".to_string()],
                    writes: vec!["NpcAI".to_string(), "Position".to_string()],
                    produces_buffers: Vec::new(),
                    consumes_buffers: Vec::new(),
                    deterministic_iteration: vec!["Entity".to_string()],
                },
                SystemDescriptor {
                    system_id: "pve-spawning.drop".to_string(),
                    version: "0.1.0".to_string(),
                    phase: TickPhase::Update,
                    order: 2,
                    reads: vec![
                        "NpcAI".to_string(),
                        "Drone".to_string(),
                        "Position".to_string(),
                    ],
                    writes: vec!["EntityLifecycle".to_string(), "Resource".to_string()],
                    produces_buffers: Vec::new(),
                    consumes_buffers: Vec::new(),
                    deterministic_iteration: vec!["Entity".to_string()],
                },
            ],
            actions: Vec::new(),
            descriptor_schema_version: DESCRIPTOR_SCHEMA_VERSION.to_string(),
        }
    }
}

pub fn pve_spawn_system(
    mut commands: Commands,
    mut clock: ResMut<PveSpawnClock>,
    config: Res<PveSpawningConfig>,
    world: Res<WorldConfig>,
    rooms: Query<&RoomConfig>,
    npcs: Query<(&NpcAI, &Position)>,
) {
    clock.tick = clock.tick.saturating_add(1);
    let spawn_interval = config.spawn_interval.max(1);
    if !clock.tick.is_multiple_of(spawn_interval) || !world.pve_only && world.pvp_enabled {
        return;
    }
    let mut counts: BTreeMap<RoomId, u32> = BTreeMap::new();
    for (_, pos) in &npcs {
        *counts.entry(pos.room).or_default() += 1;
    }
    for room in &rooms {
        if !room.pve_enabled
            || counts.get(&room.room).copied().unwrap_or(0) >= config.max_npcs_per_room
        {
            continue;
        }
        let elite_interval = spawn_interval.saturating_mul(10).max(1);
        let resource_interval = spawn_interval.saturating_mul(3).max(1);
        let kind = if clock.tick.is_multiple_of(elite_interval) {
            NpcKind::Elite
        } else if clock.tick.is_multiple_of(resource_interval) {
            NpcKind::ResourceWave
        } else {
            NpcKind::Neutral
        };
        commands.spawn((
            npc_drone(
                &config.npc_drone_body,
                if kind == NpcKind::Elite { 500 } else { 100 },
            ),
            room.spawn_origin,
            NpcAI {
                kind,
                state: NpcState::Guard,
                home: room.spawn_origin,
                target: None,
                drop_table: config.npc_drop_table.clone(),
            },
        ));
    }
}

pub fn npc_ai_system(
    mut npcs: Query<(&mut NpcAI, &mut Position)>,
    drones: Query<(Entity, &Drone, &Position)>,
) {
    let player_positions: Vec<_> = drones
        .iter()
        .filter(|(_, drone, _)| drone.owner != NPC_OWNER)
        .map(|(entity, _, position)| (entity, *position))
        .collect();
    for (mut ai, mut pos) in &mut npcs {
        let target = player_positions
            .iter()
            .filter(|(_, target_pos)| target_pos.room == pos.room)
            .min_by_key(|(_, target_pos)| {
                pos.x.abs_diff(target_pos.x) + pos.y.abs_diff(target_pos.y)
            })
            .copied();
        match (ai.state, target) {
            (_, Some((entity, target_pos))) => {
                ai.state = NpcState::Chase;
                ai.target = Some(entity);
                step_toward(&mut pos, target_pos);
            }
            (NpcState::Chase, None) => ai.state = NpcState::Return,
            (NpcState::Return, None) if pos.x != ai.home.x || pos.y != ai.home.y => {
                step_toward(&mut pos, ai.home)
            }
            (NpcState::Return, None) => ai.state = NpcState::Guard,
            (NpcState::Guard, None) => ai.state = NpcState::Patrol,
            (NpcState::Patrol, None) => pos.x = pos.x.saturating_add(1),
        }
    }
}

pub fn pve_drop_system(
    mut commands: Commands,
    dead_npcs: Query<(Entity, &NpcAI, &Position, &Drone)>,
) {
    let mut spawned = BTreeSet::new();
    for (entity, ai, position, drone) in &dead_npcs {
        if drone.owner == NPC_OWNER && drone.hits == 0 && spawned.insert(entity) {
            commands.spawn((
                Resource {
                    amounts: ai.drop_table.clone().into_iter().collect(),
                },
                *position,
            ));
            commands.entity(entity).despawn();
        }
    }
}

fn npc_drone(body: &[BodyPart], hits: u32) -> Drone {
    let registry = BodyPartRegistry::default();
    let mut drone = Drone::new(NPC_OWNER, body.to_vec(), &registry);
    drone.hits = hits;
    drone.hits_max = hits;
    drone
}

fn step_toward(pos: &mut Position, target: Position) {
    pos.x += (target.x - pos.x).signum();
    pos.y += (target.y - pos.y).signum();
}

#[cfg(test)]
mod tests {
    use super::*;
    use swarm_engine_api::ids::RoomId;

    #[test]
    fn default_spawning_config_has_npc_body_and_drop() {
        let config = PveSpawningConfig::default();

        assert_eq!(config.spawn_interval, 300);
        assert_eq!(config.max_npcs_per_room, 50);
        assert_eq!(config.npc_drone_body.len(), 3);
        assert_eq!(config.npc_drop_table.get("Energy"), Some(&50));
    }

    #[test]
    fn step_toward_moves_one_tile_per_axis() {
        let mut pos = Position {
            x: 0,
            y: 4,
            room: RoomId(0),
        };

        step_toward(
            &mut pos,
            Position {
                x: 3,
                y: 2,
                room: RoomId(0),
            },
        );

        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 3);
    }

    #[test]
    fn descriptor_is_valid_and_identifies_pve_spawning() {
        let descriptor = PveSpawningModPlugin::descriptor();
        swarm_engine_api::validation::assert_valid_descriptor(&descriptor);
        assert_eq!(descriptor.id, "pve-spawning");
        assert!(descriptor.dependencies.is_empty());
        assert_eq!(descriptor.config.len(), 4);
        assert_eq!(descriptor.systems.len(), 3);
        assert_eq!(
            descriptor
                .config
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            [
                "spawn_interval",
                "max_npcs_per_room",
                "npc_drone_body",
                "npc_drop_table"
            ]
        );
        assert_eq!(
            descriptor
                .systems
                .iter()
                .map(|system| system.system_id.as_str())
                .collect::<Vec<_>>(),
            [
                "pve-spawning.spawn",
                "pve-spawning.npc-ai",
                "pve-spawning.drop"
            ]
        );
    }
}
