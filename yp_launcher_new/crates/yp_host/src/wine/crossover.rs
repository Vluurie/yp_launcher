use yp_core::consts::GAME_EXE_NAME;
use yp_core::wine_paths::{infer_wine_prefix, is_crossover_prefix, posix};
use yp_core::Os;

use super::{is_dir, is_file, subdir_names, WineContext, NIER_DIR_NAME};

pub const BOTTLE_VAR: &str = "CX_BOTTLE";
pub const PROGRAM_FILES_VARIANTS: [&str; 2] = ["Program Files (x86)", "Program Files"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossOverBottle {
    pub name: String,
    pub path: String,
    pub game_path: Option<String>,
}

impl CrossOverBottle {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            game_path: None,
        }
    }

    pub fn with_game_path(&self, value: Option<&str>) -> Self {
        Self {
            game_path: value.map(str::to_owned).or_else(|| self.game_path.clone()),
            ..self.clone()
        }
    }
}

pub fn find_crossover_wine(ctx: &WineContext) -> Option<String> {
    if let Some(command) = ctx.wine_command() {
        return is_file(command).then(|| command.to_owned());
    }
    ctx.crossover_wine_candidates().iter().find(|candidate| is_file(candidate)).cloned()
}

pub fn bottle_roots(ctx: &WineContext) -> Vec<String> {
    if let Some(roots) = ctx.configured_bottle_roots() {
        return roots.iter().filter(|root| !root.is_empty() && is_dir(root)).cloned().collect();
    }
    let home = ctx.home();
    if home.is_empty() {
        return Vec::new();
    }
    let candidates = match ctx.os() {
        Os::MacOs => vec![
            posix::join_all(&home, &["Library", "Application Support", "CrossOver", "Bottles"]),
            posix::join_all(&home, &["Library", "Application Support", "CrossOver Preview", "Bottles"]),
        ],
        Os::Linux | Os::Windows => vec![
            posix::join(&home, ".cxoffice"),
            posix::join_all(&home, &[".var", "app", "com.codeweavers.CrossOver", "data", "crossover", "bottles"]),
        ],
    };
    candidates.into_iter().filter(|root| is_dir(root)).collect()
}

pub fn list_bottles(ctx: &WineContext) -> Vec<CrossOverBottle> {
    let mut bottles = Vec::new();
    for root in bottle_roots(ctx) {
        for name in subdir_names(&root) {
            let path = posix::join(&root, &name);
            if is_dir(&posix::join(&path, "drive_c")) {
                bottles.push(CrossOverBottle::new(name, path));
            }
        }
    }
    bottles
}

pub fn bottle_from_game_path(game_path: &str) -> Option<CrossOverBottle> {
    let prefix = infer_wine_prefix(game_path)?;
    if !is_crossover_prefix(&prefix) {
        return None;
    }
    Some(CrossOverBottle {
        name: posix::basename(&prefix).to_owned(),
        path: prefix,
        game_path: Some(game_path.to_owned()),
    })
}

pub fn resolve_bottle(ctx: &WineContext, game_path: &str, search_containing_game: bool) -> Option<CrossOverBottle> {
    if let Some(bottle) = bottle_from_game_path(game_path) {
        return Some(bottle);
    }
    if let Some(name) = ctx.var(BOTTLE_VAR)
        && let Some(named) = bottle_by_name(ctx, name)
    {
        return Some(with_resolved_game_path(&named, game_path));
    }
    if !search_containing_game {
        return None;
    }
    bottle_containing_game(ctx, game_path)
}

pub fn nier_candidates_in_bottle(bottle_path: &str) -> Vec<String> {
    PROGRAM_FILES_VARIANTS
        .iter()
        .map(|program_files| {
            posix::join_all(
                bottle_path,
                &["drive_c", program_files, "Steam", "steamapps", "common", NIER_DIR_NAME, GAME_EXE_NAME],
            )
        })
        .collect()
}

fn bottle_by_name(ctx: &WineContext, name: &str) -> Option<CrossOverBottle> {
    let target = name.to_lowercase();
    list_bottles(ctx).into_iter().find(|bottle| bottle.name.to_lowercase() == target)
}

fn bottle_containing_game(ctx: &WineContext, game_path: &str) -> Option<CrossOverBottle> {
    let bottles = list_bottles(ctx);
    if let Some(bottle) = bottles.iter().find(|bottle| posix::is_within(game_path, &bottle.path)) {
        return Some(bottle.with_game_path(Some(game_path)));
    }
    let with_game: Vec<&CrossOverBottle> = bottles
        .iter()
        .filter(|bottle| nier_candidates_in_bottle(&bottle.path).iter().any(|candidate| is_file(candidate)))
        .collect();
    match with_game.as_slice() {
        [only] => Some(with_resolved_game_path(only, game_path)),
        _ => None,
    }
}

fn with_resolved_game_path(bottle: &CrossOverBottle, game_path: &str) -> CrossOverBottle {
    if posix::is_within(game_path, &bottle.path) {
        return bottle.with_game_path(Some(game_path));
    }
    match nier_candidates_in_bottle(&bottle.path).into_iter().find(|candidate| is_file(candidate)) {
        Some(candidate) => bottle.with_game_path(Some(&candidate)),
        None => bottle.clone(),
    }
}
