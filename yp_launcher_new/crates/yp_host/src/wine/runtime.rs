use std::fmt;

use yp_core::wine_paths::{infer_wine_prefix, posix};
use yp_core::Os;

use super::compat_tool::find_steam_configured_proton;
use super::crossover::{bottle_from_game_path, find_crossover_wine, resolve_bottle, CrossOverBottle};
use super::proton::{compat_data_path, find_any_proton_path, find_proton_path, has_any_proton};
use super::{is_file, WineContext, PROTON_PATH_VAR};

const WINE_BINARIES: [&str; 2] = ["wine64", "wine"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WineSource {
    CrossOver,
    Proton,
    Wine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WineRuntime {
    pub binary: String,
    pub source: WineSource,
    pub bottle: Option<CrossOverBottle>,
    pub prefix: Option<String>,
}

impl WineRuntime {
    pub fn label(&self) -> String {
        match (&self.source, &self.bottle) {
            (WineSource::CrossOver, Some(bottle)) => format!("CrossOver Wine ({})", bottle.name),
            (WineSource::CrossOver, None) => "CrossOver Wine".to_owned(),
            (WineSource::Proton, _) => "Proton".to_owned(),
            (WineSource::Wine, _) => "Wine".to_owned(),
        }
    }
}

impl fmt::Display for WineRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.label(), self.binary)
    }
}

pub fn detect(ctx: &WineContext, game_exe: Option<&str>) -> Option<WineRuntime> {
    if ctx.os() == Os::Windows {
        return None;
    }
    let game_path = game_exe.map(posix::normalize);

    if let Some(proton) = ctx.var(PROTON_PATH_VAR).filter(|path| is_file(path)) {
        return Some(proton_runtime(proton.to_owned(), &compat_data_path(ctx, game_path.as_deref())));
    }

    let wine = find_crossover_wine(ctx);

    if let Some(game_path) = game_path.as_deref() {
        if let (Some(wine), Some(bottle)) = (&wine, bottle_from_game_path(game_path)) {
            return Some(crossover(wine, bottle));
        }
        if let Some(steam) = ctx.infer_steam_context(game_path) {
            let proton = find_steam_configured_proton(ctx, &steam.steam_root, &steam.library_root)
                .or_else(|| find_proton_path(&steam.steam_root, &steam.library_root))
                .or_else(|| find_any_proton_path(ctx));
            if let Some(proton) = proton {
                return Some(proton_runtime(proton, &steam.compat_data));
            }
        }
        if let Some(wine) = &wine
            && let Some(resolved) = resolve_bottle(ctx, game_path, true)
        {
            return Some(crossover(wine, resolved));
        }
        if ctx.os() == Os::Linux
            && let Some(proton) = find_any_proton_path(ctx)
        {
            return Some(proton_runtime(proton, &compat_data_path(ctx, Some(game_path))));
        }
    }

    ctx.find_on_path(&WINE_BINARIES).map(|binary| WineRuntime {
        binary,
        source: WineSource::Wine,
        bottle: None,
        prefix: Some(plain_wine_prefix(ctx, game_path.as_deref())),
    })
}

pub fn is_available(ctx: &WineContext) -> bool {
    find_crossover_wine(ctx).is_some()
        || ctx.find_on_path(&WINE_BINARIES).is_some()
        || (ctx.os() == Os::Linux && has_any_proton(ctx))
}

fn proton_runtime(binary: String, compat_data: &str) -> WineRuntime {
    WineRuntime {
        binary,
        source: WineSource::Proton,
        bottle: None,
        prefix: Some(posix::join(compat_data, "pfx")),
    }
}

fn crossover(wine: &str, bottle: CrossOverBottle) -> WineRuntime {
    let prefix = bottle
        .game_path
        .as_deref()
        .and_then(infer_wine_prefix)
        .unwrap_or_else(|| bottle.path.clone());
    WineRuntime {
        binary: wine.to_owned(),
        source: WineSource::CrossOver,
        prefix: Some(prefix),
        bottle: Some(bottle),
    }
}

fn plain_wine_prefix(ctx: &WineContext, game_path: Option<&str>) -> String {
    if let Some(prefix) = game_path.and_then(infer_wine_prefix) {
        return prefix;
    }
    if let Some(prefix) = ctx.var("WINEPREFIX") {
        return prefix.to_owned();
    }
    posix::join(&ctx.home(), ".wine")
}
