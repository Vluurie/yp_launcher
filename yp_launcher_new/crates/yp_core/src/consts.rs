pub const APP_VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

pub const GITHUB_URL: &str = "https://github.com/Vluurie/yp_launcher";
pub const NAMS_GITLAB_URL: &str =
    "https://gitlab.yasupa.de/nams/nams-rs/-/tree/master/mods/yorha_protocol?ref_type=heads";
pub const GUIDE_URL: &str =
    "https://gitlab.yasupa.de/nams/yp-docs/-/blob/master/YoRHa_Protocol_Documentation.md";
pub const DISCORD_URL: &str = "https://discord.gg/Z5spWtF8qs";
pub const NAO_LAUNCHER_URL: &str = "https://www.nexusmods.com/nierautomata/mods/772?tab=files";
pub const CLI_DOCS_URL: &str = "https://gitlab.yasupa.de/nams/yp-docs/-/blob/master/YorHa_Protocol_Documentation.md#how-to-start-via-command-line";
pub const MULTI_OUTFIT_NPC_URL: &str = "https://www.nexusmods.com/nierautomata/mods/789";
pub const RESHADE_ME_URL: &str = "https://reshade.me";
pub const MIGOTO_URL: &str = "https://github.com/bo3b/3Dmigoto";

pub const ALLOWED_EXTENSION: &str = "exe";

pub const GAME_EXE_NAME: &str = "NieRAutomata.exe";
pub const GAME_SIGNATURE: &str = "PRJ_028";

pub const NAMS_EXE_NAME: &str = "NAMS.exe";
pub const NAMS_DLL_IDENTITY: &[u8] = b"NamsIdentity\0";
pub const YORHA_DLL_NAME: &str = "yorha_protocol.dll";
pub const PLUGINS_DIR_NAME: &str = "plugins";
pub const PLUGINS_DISABLED_DIR_NAME: &str = "plugins_disabled";
pub const LOGS_DIR_NAME: &str = "logs";
pub const NAMS_LOG_NAME: &str = "nams.log";

pub const THIRD_PARTY_DIR_NAME: &str = "thirdparty";
pub const RESHADE_DIR_NAME: &str = "reshade";
pub const MIGOTO_DIR_NAME: &str = "3dmigoto";
pub const GAME_MODS_DIR_NAME: &str = "game";
pub const GAME_MODS_CONFIG_NAME: &str = "game.toml";
pub const GRAPHIC_MODS_DIR_NAME: &str = "graphic";
pub const THIRD_PARTY_DISABLED_DIR_NAME: &str = "disabled";
pub const RESHADE_MOD_DIR_NAME: &str = "ReShade";
pub const MIGOTO_MOD_DIR_NAME: &str = "3DMigoto";
pub const THIRD_PARTY_DOC_PATH: &str = "docs/native/35_third_party.md";

pub const ARG_RUN: &str = "run";
pub const ARG_NIER_PATH: &str = "--nier-path";
pub const ARG_VANILLA: &str = "--vanilla";
pub const ARG_CONSOLE: &str = "--console";
pub const ARG_VERIFY: &str = "verify";
pub const ARG_CONFIG: &str = "config";
pub const ARG_JSON: &str = "--json";

pub const SHORTCUT_NAME: &str = "NieR Automata (YoRHa Protocol)";
pub const SHORTCUT_DESCRIPTION: &str = "Launch NieR:Automata with YoRHa Protocol";

pub const TAB_MODLOADER_ID: &str = "modloader";
pub const TAB_YORHA_ID: &str = "yorha";

pub fn is_nams_dll(bytes: &[u8]) -> bool {
    bytes.windows(NAMS_DLL_IDENTITY.len()).any(|window| window == NAMS_DLL_IDENTITY)
}
