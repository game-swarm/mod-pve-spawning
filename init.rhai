// init.rhai — pve-spawning module initialization
// Executed once when the mod is loaded

let MOD_NAME = "pve-spawning";
let MOD_VERSION = "0.1.0";

// Read config from mod.toml (injected by engine)
// let config = actions.get_config();

actions.log_info(`${MOD_NAME} v${MOD_VERSION} loaded`);
