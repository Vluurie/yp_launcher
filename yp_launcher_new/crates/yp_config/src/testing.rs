use std::sync::Mutex;

use indexmap::IndexMap;
use serde_json::{Map, Value as Json, json};

use crate::cli::{CliOutput, NamsCli};
use crate::error::Result;

pub fn sample_schema() -> Json {
    json!({
        "ok": true,
        "nams_version": "4.0.0",
        "config_version": 5,
        "lang": "en",
        "languages": ["en", "de", "zh", "th"],
        "categories": [
            { "id": "native", "title": "NAMS", "count": 3 },
            { "id": "content", "title": "Content features", "count": 1 },
            { "id": "cutscene", "title": "Cutscenes", "count": 1 },
            { "id": "graphics/backend", "title": "Graphics API", "count": 2 },
            { "id": "graphics/lod", "title": "Level of detail and culling", "count": 2 },
            { "id": "graphics/bloom", "title": "Bloom", "count": 1 },
            { "id": "graphics/shadows", "title": "Shadows", "count": 4 },
            { "id": "graphics/high_grids", "title": "High-res map grids", "count": 1 },
            { "id": "graphics/framerate", "title": "Frame rate", "count": 1 },
            { "id": "graphics/d3d12", "title": "Direct3D 12", "count": 4 },
            { "id": "input/mouse", "title": "Mouse and camera", "count": 2 },
            { "id": "input/keyboard", "title": "Key bindings", "count": 4 },
            { "id": "textures", "title": "Texture injection", "count": 2 },
            { "id": "mods", "title": "Mods", "count": 1 },
            { "id": "plugins.yorha_protocol", "title": "YoRHa Protocol", "count": 4 }
        ],
        "files": [
            { "file": "nams.toml", "format": "toml", "path": "nams/nams.toml" },
            { "file": "lodmod.toml", "format": "toml", "path": "nams/lodmod.toml" },
            { "file": "naiom.toml", "format": "toml", "path": "nams/naiom.toml" },
            { "file": "texture_injection.toml", "format": "toml", "path": "nams/texture_injection.toml" },
            { "file": "default_mods.toml", "format": "toml", "path": "nams/default_mods.toml" },
            { "file": "_internal/cache/settings.json", "format": "json", "path": "nams/_internal/cache/settings.json" }
        ],
        "plugins": [{ "plugin": "yorha_protocol", "title": "YoRHa Protocol", "source": "built-in", "category": "plugins.yorha_protocol" }],
        "plugin_errors": [],
        "settings": [
            { "id": "nams.disable_plugin_loading", "category": "native", "file": "nams.toml", "key": "disable_plugin_loading", "type": "bool", "default": false, "restart_required": true, "label": "Disable Plugin Loading", "description": "Load no plugins." },
            { "id": "nams.experimental_default_outfits", "category": "native", "file": "nams.toml", "key": "experimental_default_outfits", "type": "bool", "default": false, "restart_required": true, "label": "Default Outfits", "description": "Read default_mods.toml." },
            { "id": "nams.graphics_backend", "category": "graphics/backend", "file": "nams.toml", "key": "graphics_backend", "type": "enum", "default": "d3d11", "restart_required": true, "label": "Graphics API", "description": "The graphics API.", "choices": [{ "value": "d3d11", "label": "Direct3D 11" }, { "value": "d3d12", "label": "Direct3D 12" }] },
            { "id": "nams.content.custom_items", "category": "content", "file": "nams.toml", "key": "content.custom_items", "type": "bool", "default": true, "restart_required": true, "label": "Custom Items", "description": "Items from mods." },
            { "id": "nams.cutscene.enable_h264", "category": "cutscene", "file": "nams.toml", "key": "cutscene.enable_h264", "type": "bool", "default": false, "restart_required": true, "label": "H.264 Cutscenes", "description": "H.264 for USM." },
            { "id": "nams.skip_startup_logos", "category": "native", "file": "nams.toml", "key": "skip_startup_logos", "type": "bool", "default": false, "absent_value": true, "restart_required": true, "label": "Skip Startup Logos", "description": "Skip the logos." },
            { "id": "lodmod.enabled", "category": "graphics/lod", "file": "lodmod.toml", "key": "enabled", "type": "bool", "default": false, "restart_required": false, "label": "Enable LodMod Ext", "description": "Master switch." },
            { "id": "lodmod.lod_multiplier", "category": "graphics/lod", "file": "lodmod.toml", "key": "lod_multiplier", "type": "float", "default": 0.0, "restart_required": false, "label": "LOD Multiplier", "description": "LOD distance.", "range": { "min": 0.0, "max": 10.0 }, "ui": { "presets": [{ "label": "Best quality", "value": 0.0 }, { "label": "Vanilla", "value": 1.0 }] } },
            { "id": "lodmod.bloom_reference_height", "category": "graphics/bloom", "file": "lodmod.toml", "key": "bloom_reference_height", "type": "int", "default": 0, "restart_required": false, "label": "Bloom Reference Height", "description": "Bloom shimmer fix.", "type_range": { "min": 0, "max": 4294967295u64 }, "zero_or_range": { "min": 360, "max": 4320 }, "ui": { "step": 20, "presets": [{ "label": "Vanilla", "value": 0 }, { "label": "2017 footprint", "value": 900 }] } },
            { "id": "lodmod.shadow_resolution", "category": "graphics/shadows", "file": "lodmod.toml", "key": "shadow_resolution", "type": "int", "default": 2048, "restart_required": false, "label": "Shadow Resolution", "description": "Shadow map size.", "choices": [{ "label": "512", "value": 512 }, { "label": "1024", "value": 1024 }, { "label": "2048", "value": 2048 }, { "label": "4096", "value": 4096 }, { "label": "8192", "value": 8192 }], "type_range": { "min": 0, "max": 4294967295u64 } },
            { "id": "lodmod.shadow_distance_multiplier", "category": "graphics/shadows", "file": "lodmod.toml", "key": "shadow_distance_multiplier", "type": "float", "default": 1.0, "restart_required": false, "label": "Shadow Distance Multiplier", "description": "Shadow distance.", "ui": { "min": 1.0, "max": 3.0, "step": 0.1 } },
            { "id": "lodmod.shadow_blur_scale", "category": "graphics/shadows", "file": "lodmod.toml", "key": "shadow_blur_scale", "type": "list", "default": [1.0, 1.0, 1.0, 1.0], "restart_required": false, "label": "Shadow Blur Scale", "description": "Blur per cascade.", "items": { "type": "float" }, "length": 4, "range": { "min": 0.0, "max": 2.0 }, "ui": { "step": 0.05 } },
            { "id": "lodmod.shadow_force_all_filters", "category": "graphics/shadows", "file": "lodmod.toml", "key": "shadow_force_all_filters", "type": "list", "default": ["ENKEI"], "restart_required": false, "label": "Force All Filters", "description": "Mesh names.", "items": { "type": "string" } },
            { "id": "lodmod.high_grids.room_rings", "category": "graphics/high_grids", "file": "lodmod.toml", "key": "high_grids.room_rings", "type": "list", "default": [], "restart_required": false, "label": "Room Rings", "description": "Rings per room.", "items": { "fields": [{ "key": "room", "type": "int", "required": true, "description": "Room id." }, { "key": "rings", "type": "int", "required": true, "description": "Rings." }], "type": "table" } },
            { "id": "lodmod.d3d12_backend", "category": "graphics/backend", "file": "lodmod.toml", "key": "d3d12_backend", "type": "bool", "default": null, "optional": true, "restart_required": true, "label": "Direct3D 12", "description": "Unset = graphics_backend decides." },
            { "id": "lodmod.d3d12.sun_shadows", "category": "graphics/d3d12", "file": "lodmod.toml", "key": "d3d12.sun_shadows", "type": "bool", "default": false, "restart_required": false, "label": "Ray traced sun shadows", "description": "Needs DXR." },
            { "id": "lodmod.d3d12.ambient_occlusion", "category": "graphics/d3d12", "file": "lodmod.toml", "key": "d3d12.ambient_occlusion", "type": "bool", "default": false, "restart_required": false, "label": "Ray-Traced Ambient Occlusion", "description": "Needs DXR." },
            { "id": "lodmod.d3d12.ao_radius", "category": "graphics/d3d12", "file": "lodmod.toml", "key": "d3d12.ao_radius", "type": "float", "default": 1.0, "restart_required": false, "label": "RTAO Radius", "description": "Radius.", "range": { "min": 0.1, "max": 4.0 }, "ui": { "step": 0.1 } },
            { "id": "lodmod.d3d12.cull_casters", "category": "graphics/d3d12", "file": "lodmod.toml", "key": "d3d12.cull_casters", "type": "bool", "default": false, "advanced": true, "restart_required": false, "label": "Cull Shadow Casters", "description": "For performance comparisons." },
            { "id": "lodmod.fps_uncap_in_gameplay", "category": "graphics/framerate", "file": "lodmod.toml", "key": "fps_uncap_in_gameplay", "type": "bool", "default": false, "restart_required": false, "label": "Uncap FPS in gameplay", "description": "Uncap the frame rate in gameplay." },
            { "id": "naiom.sensitivity", "category": "input/mouse", "file": "naiom.toml", "key": "sensitivity", "type": "float", "default": 2.0, "restart_required": false, "label": "Sensitivity", "description": "Mouse sensitivity.", "ui": { "min": 0.25, "max": 10.0, "step": 0.25 } },
            { "id": "naiom.misc_custom_cursor_menu", "category": "input/mouse", "file": "naiom.toml", "key": "misc_custom_cursor_menu", "type": "string", "default": "", "restart_required": true, "label": "Menu Cursor", "description": "Cursor file.", "format": "path", "max_length": 1024 },
            { "id": "naiom.bindings.standard_jump", "category": "input/keyboard", "file": "naiom.toml", "key": "bindings.standard_jump", "type": "string", "default": "", "restart_required": false, "label": "Jump", "description": "Additional key for jumping.", "format": "key_binding", "keys": ["A", "B", "SPACE", "LSHIFT", "MOUSE1"], "modifiers": ["SHIFT", "CTRL", "ALT"] },
            { "id": "naiom.bindings.standard_fire", "category": "input/keyboard", "file": "naiom.toml", "key": "bindings.standard_fire", "type": "string", "default": "", "restart_required": false, "label": "Fire", "description": "Additional key for firing.", "format": "key_binding", "keys": ["A", "W", "SPACE", "MOUSE1", "MOUSE2", "CONTROLLER_A"], "modifiers": ["SHIFT", "CTRL", "ALT"] },
            { "id": "naiom.bindings.standard_move_forward", "category": "input/keyboard", "file": "naiom.toml", "key": "bindings.standard_move_forward", "type": "string", "default": "", "restart_required": false, "label": "Move Forward", "description": "Additional key for moving forward.", "format": "key_binding", "keys": ["A", "W", "SPACE"], "modifiers": ["SHIFT", "CTRL", "ALT"] },
            { "id": "naiom.misc_open_debug_menu", "category": "input/keyboard", "file": "naiom.toml", "key": "misc_open_debug_menu", "type": "int", "default": 0, "restart_required": true, "label": "Debug Menu Key", "description": "Virtual key code.", "format": "virtual_key", "type_range": { "min": 0, "max": 4294967295u64 }, "ui": { "min": 0, "max": 254 } },
            { "id": "texture_injection.load_order", "category": "textures", "file": "texture_injection.toml", "key": "load_order", "type": "list", "default": [], "restart_required": true, "label": "Texture Pack Load Order", "description": "Pack order.", "items": { "type": "string" } },
            { "id": "texture_injection.vram_budget_mb", "category": "textures", "file": "texture_injection.toml", "key": "vram_budget_mb", "type": "int", "default": 0, "restart_required": true, "label": "VRAM Budget", "description": "Budget in MB.", "type_range": { "min": 0, "max": 4294967295u64 }, "ui": { "min": 0, "max": 24576, "step": 256 } },
            { "id": "default_mods.outfit_bare", "category": "mods", "file": "default_mods.toml", "key": "outfit_bare", "type": "list", "default": [], "restart_required": true, "label": "Default Outfit", "description": "Outfit mods.", "items": { "any_of": [{ "type": "string" }, { "type": "table", "fields": [{ "key": "path", "type": "string", "required": true, "description": "Path." }] }], "type": "any_of" } },
            { "id": "plugins.yorha_protocol.keybinds.main.yorha_protocol", "category": "plugins.yorha_protocol", "group": "keybinds", "file": "_internal/cache/settings.json", "key": "keybinds.main.yorha_protocol", "type": "string", "default": "F1", "restart_required": false, "label": "Open Workspace", "description": "Opens the overlay.", "max_length": 32 },
            { "id": "plugins.yorha_protocol.firstPlaythrough", "category": "plugins.yorha_protocol", "group": "overlay", "file": "_internal/cache/settings.json", "key": "firstPlaythrough", "type": "bool", "default": true, "restart_required": false, "label": "First Playthrough Mode", "description": "No spoilers." },
            { "id": "plugins.yorha_protocol.cheatsConfig.infiniteHealth", "category": "plugins.yorha_protocol", "group": "cheats", "file": "_internal/cache/settings.json", "key": "cheatsConfig.infiniteHealth", "type": "bool", "default": false, "restart_required": false, "label": "Infinite Health", "description": "Take no damage." },
            { "id": "plugins.yorha_protocol.overlayLanguage", "category": "plugins.yorha_protocol", "group": "overlay", "file": "_internal/cache/settings.json", "key": "overlayLanguage", "type": "enum", "default": "system", "restart_required": false, "label": "Overlay Language", "description": "Language of the overlay.", "choices": [{ "value": "system", "label": "System" }, { "value": "en", "label": "English" }, { "value": "de", "label": "Deutsch" }] }
        ]
    })
}

pub struct FakeNamsCli {
    schema: Json,
    values: Mutex<IndexMap<String, Json>>,
    in_file: Mutex<Vec<String>>,
    broken: Mutex<Vec<(String, String)>>,
    rejected: Mutex<Vec<String>>,
    calls: Mutex<Vec<Vec<String>>>,
}

impl Default for FakeNamsCli {
    fn default() -> Self {
        Self::new(sample_schema())
    }
}

impl FakeNamsCli {
    pub fn new(schema: Json) -> Self {
        Self {
            schema,
            values: Mutex::new(IndexMap::new()),
            in_file: Mutex::new(Vec::new()),
            broken: Mutex::new(Vec::new()),
            rejected: Mutex::new(Vec::new()),
            calls: Mutex::new(Vec::new()),
        }
    }

    pub fn with(self, id: &str, value: Json) -> Self {
        self.values.lock().unwrap().insert(id.to_owned(), value);
        self.in_file.lock().unwrap().push(id.to_owned());
        self
    }

    pub fn break_file(&self, file: &str, error: &str) {
        self.broken.lock().unwrap().push((file.to_owned(), error.to_owned()));
    }

    pub fn put(&self, id: &str, value: Json) {
        self.values.lock().unwrap().insert(id.to_owned(), value);
        let mut in_file = self.in_file.lock().unwrap();
        if !in_file.iter().any(|known| known == id) {
            in_file.push(id.to_owned());
        }
    }

    pub fn reject(&self, id: &str) {
        self.rejected.lock().unwrap().push(id.to_owned());
    }

    pub fn value(&self, id: &str) -> Option<Json> {
        self.values.lock().unwrap().get(id).cloned()
    }

    pub fn calls(&self) -> Vec<Vec<String>> {
        self.calls.lock().unwrap().clone()
    }

    pub fn count(&self, command: &str) -> usize {
        self.calls().iter().filter(|args| args[0] == command).count()
    }

    fn settings(&self) -> Vec<Json> {
        self.schema["settings"].as_array().cloned().unwrap_or_default()
    }

    fn current(&self) -> Map<String, Json> {
        let values = self.values.lock().unwrap();
        self.settings()
            .iter()
            .filter_map(|setting| {
                let id = setting["id"].as_str()?.to_owned();
                let value = values.get(&id).cloned().unwrap_or_else(|| {
                    setting.get("absent_value").cloned().unwrap_or_else(|| setting["default"].clone())
                });
                Some((id, value))
            })
            .collect()
    }

    fn known(&self, id: &str) -> bool {
        self.settings().iter().any(|setting| setting["id"] == json!(id))
    }

    fn get(&self) -> Json {
        let in_file = self.in_file.lock().unwrap().clone();
        let broken = self.broken.lock().unwrap().clone();
        let not_in_file: Vec<String> = self
            .settings()
            .iter()
            .filter_map(|setting| setting["id"].as_str().map(str::to_owned))
            .filter(|id| !in_file.contains(id))
            .collect();
        let files: Map<String, Json> = self.schema["files"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|file| {
                let name = file["file"].as_str()?.to_owned();
                let error = broken
                    .iter()
                    .find(|(broken, _)| *broken == name)
                    .map(|(_, error)| Json::String(error.clone()))
                    .unwrap_or(Json::Null);
                Some((name, json!({ "exists": true, "error": error, "format": file["format"], "path": file["path"] })))
            })
            .collect();
        json!({ "ok": true, "config_version": 5, "files": files, "not_in_file": not_in_file, "values": self.current() })
    }

    fn set(&self, input: &[u8]) -> Json {
        let input: IndexMap<String, Json> = serde_json::from_slice(input).unwrap_or_default();
        let rejected = self.rejected.lock().unwrap().clone();
        let problems: Vec<Json> = input
            .keys()
            .filter(|id| rejected.contains(id) || !self.known(id))
            .map(|id| json!({ "code": "invalid_value", "severity": "error", "message": format!("{id} rejected"), "id": id }))
            .collect();
        if !problems.is_empty() {
            return json!({ "ok": false, "code": "validation_failed", "error": "rejected", "exit_code": 65, "problems": problems });
        }
        let before = self.current();
        let mut values = self.values.lock().unwrap();
        let mut in_file = self.in_file.lock().unwrap();
        let mut changed = Vec::new();
        let mut unchanged = Vec::new();
        for (id, value) in input {
            let old = before.get(&id).cloned().unwrap_or(Json::Null);
            if !in_file.contains(&id) {
                in_file.push(id.clone());
            }
            values.insert(id.clone(), value.clone());
            if old == value {
                unchanged.push(Json::String(id));
            } else {
                changed.push(json!({ "id": id, "old": old, "new": value, "restart_required": false }));
            }
        }
        json!({ "ok": true, "dry_run": false, "changed": changed, "unchanged": unchanged, "files_written": [] })
    }
}

fn answer(document: Json) -> Result<CliOutput> {
    Ok(CliOutput {
        exit_code: Some(if document["ok"] == json!(false) { 65 } else { 0 }),
        stdout: document.to_string(),
        ..CliOutput::default()
    })
}

impl NamsCli for FakeNamsCli {
    fn run(&self, args: &[String], stdin: Option<&[u8]>) -> Result<CliOutput> {
        self.calls.lock().unwrap().push(args.to_vec());
        match args.first().map(String::as_str) {
            Some("schema") => answer(self.schema.clone()),
            Some("get") => answer(self.get()),
            Some("set") => answer(self.set(stdin.unwrap_or_default())),
            Some("migrate") => answer(json!({ "ok": true, "dry_run": false, "up_to_date": true, "to_version": 5, "recorded_version": 5, "from_version": {}, "changes": [], "backups": [], "files_written": [], "skipped": [] })),
            Some("validate") => answer(json!({ "ok": true, "valid": true, "errors": 0, "warnings": 0, "problems": [] })),
            _ => answer(json!({ "ok": false, "code": "unknown_command", "error": "unknown command" })),
        }
    }
}
