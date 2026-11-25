use gpui::{
    AnyWindowHandle, App, Bounds, Context, IntoElement, Render, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::Duration,
};
use yp_nier_ui::uid::{LayerKind, UidDocument, UidElement, UidResources};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn load(path: &Path) -> Result<UidDocument> {
    let bytes = fs::read(path)?;
    if path
        .extension()
        .is_some_and(|s| s.eq_ignore_ascii_case("json"))
    {
        Ok(UidDocument::from_compiled(&bytes)?)
    } else {
        Ok(UidDocument::parse(&bytes)?)
    }
}
fn inspect(path: &Path, total: &mut [usize; 3]) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if entry.file_type()?.is_dir()
                || p.extension().is_some_and(|v| v.eq_ignore_ascii_case("uid"))
            {
                inspect(&p, total)?;
            }
        }
    } else {
        let doc = load(path).map_err(|e| format!("{}: {e}", path.display()))?;
        total[0] += 1;
        total[1] += doc.layers.len();
        total[2] += doc.clips.len();
    }
    Ok(())
}

fn resources(path: &Path, doc: &UidDocument) -> Result<UidResources> {
    let mut resources = UidResources::new();
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut docs = vec![(dir.to_path_buf(), Arc::new(doc.clone()))];
    let mut banks = vec![dir.to_path_buf()];
    let mut seen = HashSet::new();
    let mut wanted = HashSet::new();
    let stem = dir.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    let locale = stem
        .rsplit('_')
        .next()
        .filter(|s| matches!(*s, "us" | "de" | "es" | "fr" | "it" | "jp"));
    while let Some((local, current)) = docs.pop() {
        for layer in &current.layers {
            match &layer.kind {
                LayerKind::Image(g) => {
                    wanted.insert(g.graphic.key.texture);
                }
                LayerKind::NineGrid(g) => {
                    wanted.insert(g.graphic.key.texture);
                }
                LayerKind::Locator { name, part }
                    if !name.is_empty()
                        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                        && seen.insert((name.clone(), *part)) =>
                {
                    let filename = format!("{name}{part:04x}.uid");
                    let mut paths = vec![local.clone()];
                    if let Some(root) = dir.parent() {
                        if let Some(lang) = locale {
                            paths.push(root.join(format!("ui_{name}_{lang}.dat")));
                        }
                        paths.push(root.join(format!("ui_{name}.dat")));
                    }
                    if let Some(bank) = paths.into_iter().find(|d| d.join(&filename).is_file()) {
                        let child = Arc::new(load(&bank.join(filename))?);
                        resources.insert_document(name.clone(), *part, child.clone());
                        if !banks.contains(&bank) {
                            banks.push(bank.clone());
                        }
                        docs.push((bank, child));
                    }
                }
                _ => {}
            }
        }
    }
    for dir in banks {
        for entry in fs::read_dir(&dir)? {
            let file = entry?.path();
            if !file
                .extension()
                .is_some_and(|s| s.eq_ignore_ascii_case("uvd"))
            {
                continue;
            }
            resources.load_uvd(&fs::read(&file)?)?;
            for name in ["core_pt_00", "core_pt_01"] {
                if let Some(key) = resources.sprite_key(name) {
                    wanted.insert(key.texture);
                }
            }
            let index = file.with_extension("wta");
            let local = file.with_extension("wtp");
            let payload = if local.is_file() {
                local
            } else {
                dir.with_extension("dtt").join(local.file_name().unwrap())
            };
            if index.is_file() && payload.is_file() {
                resources.load_wta(&fs::read(index)?, &fs::read(payload)?, |id| {
                    wanted.contains(&id)
                })?;
            }
        }
    }
    Ok(resources)
}

struct Viewer {
    doc: Arc<UidDocument>,
    resources: Rc<UidResources>,
    clip: Option<u32>,
    time: Option<f32>,
}
impl Render for Viewer {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut element =
            UidElement::new("uid", self.doc.clone()).resources(self.resources.clone());
        if let Some(clip) = self.clip {
            element = element.animation(clip);
        }
        if let Some(time) = self.time {
            element = element.at_time(time);
        }
        div().size_full().bg(rgb(0xcdc8b0)).child(element)
    }
}

fn main() -> Result<()> {
    let mut args = env::args_os().skip(1);
    let usage = "expected inspect <file-or-directory>, compile <input.uid> <layout.json>, or view <input.uid> [--clip id | --time seconds] [--capture image.png]";
    let cmd = args.next().ok_or(usage)?;
    let path = PathBuf::from(args.next().ok_or(usage)?);
    if cmd == "inspect" {
        let mut totals = [0; 3];
        inspect(&path, &mut totals)?;
        println!(
            "{} UID files, {} layers, {} animation clips",
            totals[0], totals[1], totals[2]
        );
        return Ok(());
    }
    let doc = Arc::new(load(&path)?);
    if cmd == "compile" {
        let out = PathBuf::from(args.next().ok_or(usage)?);
        fs::write(out, serde_json::to_vec(doc.as_ref())?)?;
        return Ok(());
    }
    if cmd != "view" {
        return Err(usage.into());
    }
    let (mut clip, mut time, mut capture) = (None, None, None);
    while let Some(flag) = args.next() {
        let val = args.next().ok_or(usage)?;
        match flag.to_str() {
            Some("--clip") => clip = Some(val.to_str().ok_or(usage)?.parse::<u32>()?),
            Some("--time") => time = Some(val.to_str().ok_or(usage)?.parse::<f32>()?),
            Some("--capture") => capture = Some(PathBuf::from(val)),
            _ => return Err(usage.into()),
        }
    }
    if let Some(id) = clip
        && !doc.clips.iter().any(|c| c.id == id)
    {
        return Err(format!("missing UID clip {id}").into());
    }
    let extensions: std::collections::BTreeSet<_> = doc
        .layers
        .iter()
        .filter_map(|l| l.extension.as_ref().map(|e| e.id))
        .filter(|id| !matches!(id, 1000 | 1003 | 1020 | 1030 | 9020))
        .collect();
    if !extensions.is_empty() {
        eprintln!("UID extensions drawn without their game pass: {extensions:?}");
    }
    let resources = Rc::new(resources(&path, &doc)?);
    {
        let platform = gpui_platform::current_platform(false);
        yp_nier_ui::Theme::with_text_system(platform.text_system());
        gpui::Application::with_platform(platform)
    }
    .run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(960.), px(540.)), cx);
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    show: capture.is_none(),
                    focus: capture.is_none(),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|_| Viewer {
                        doc,
                        resources,
                        clip,
                        time,
                    })
                },
            )
            .expect("open UID view");
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        if let Some(out) = capture {
            let h = AnyWindowHandle::from(handle);
            h.update(cx, |_, window, _| window.resize(size(px(1280.), px(720.))))
                .unwrap();
            cx.spawn(async move |cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(350))
                    .await;
                h.update(cx, |_, window, cx| {
                    window.refresh();
                    window.draw(cx).clear(cx);
                    window
                        .render_to_image()
                        .expect("capture UID")
                        .save(out)
                        .expect("save UID render");
                    cx.quit();
                })
                .unwrap();
            })
            .detach();
        } else {
            cx.activate(true);
        }
    });
    Ok(())
}
