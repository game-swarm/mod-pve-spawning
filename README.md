# pve-spawning

PvE 敌人生成模组。管理中立 NPC 生成、资源潮和公共事件。

## 职责

- 在指定房间按配置生成中立 NPC drone（PvE 威胁）
- NPC 类型：中立 drone（可攻击）、资源潮（大量弱怪）、精英怪（带特殊能力）
- NPC 在指定区域巡逻或守卫
- PvE 击杀掉落资源（通过 Resource Registry）
- PvE 行为由简单的状态机驱动（守卫 → 巡逻 → 追击 → 回归）
- beginner/soft_launch 世界默认 PvE-only，无 PvP

## 依赖

- bevy
- combat-core（NPC 使用同类战斗系统）

## 配置

以下配置由 Engine 从 `mods.lock` 严格解码并注入 `PveSpawningConfig`；NPC body 和掉落表保持 typed array/map，不经过字符串解析。

mod.toml:
```toml
[config]
spawn_interval = { type = "u32", default = 300 }
max_npcs_per_room = { type = "u32", default = 50 }
npc_drone_body = { type = "array<BodyPart>", default = ["Attack", "Move", "Move"] }
npc_drop_table = { type = "map<Resource,u32>", default = { Energy = 50 } }
```

## 事件

- 写入: `Drone`（NPC 实体）, `NpcAI`（NPC 行为组件）
- 读取: `Position`, `RoomConfig`, `WorldConfig`

## Standalone Development

This crate pins `swarm-engine-api` and `swarm-engine-plugin-sdk` to version `0.1.0` at the `v0.1.0` engine API Git tag. Cargo fetches both crates directly from that release.

```sh
git clone <this-mod-repository-url> pve-spawning
cd pve-spawning
cargo check
cargo test
```

To adopt a later API/SDK release, update both exact versions and both Git tags in `Cargo.toml` together.
