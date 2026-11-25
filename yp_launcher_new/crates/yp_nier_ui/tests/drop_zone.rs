#![cfg(debug_assertions)]

use gpui::{Context, IntoElement, Render, TestAppContext, Window, div, prelude::*, px};
use std::{cell::Cell, rc::Rc};
use yp_nier_ui::{MenuDropZone, Theme};

struct InstallPanel {
    width: f32,
    files: Rc<Cell<usize>>,
    folders: Rc<Cell<usize>>,
}

impl Render for InstallPanel {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let files = self.files.clone();
        let folders = self.folders.clone();
        div().w(px(self.width)).child(
            MenuDropZone::new("packages", "Drop a mod archive or folder", |_, _, _| {})
                .on_browse_files(move |_, _| files.set(files.get() + 1))
                .on_browse_folder(move |_, _| folders.set(folders.get() + 1)),
        )
    }
}

#[gpui::test]
fn browse_actions_fit_and_remain_clickable_as_the_panel_resizes(cx: &mut TestAppContext) {
    let files = Rc::new(Cell::new(0));
    let folders = Rc::new(Cell::new(0));
    let (panel, cx) = cx.add_window_view(|_, _| InstallPanel {
        width: 760.,
        files: files.clone(),
        folders: folders.clone(),
    });
    for (index, width) in [760., 390., 170., 760.].into_iter().enumerate() {
        panel.update(cx, |panel, cx| {
            panel.width = width;
            cx.notify();
        });
        cx.run_until_parked();
        let group = cx.debug_bounds("browse-actions").unwrap();
        let file = cx.debug_bounds("browse-files").unwrap();
        let folder = cx.debug_bounds("browse-folder").unwrap();
        assert!(group.right() <= px(width));
        for (button, label) in [(file, "Browse files"), (folder, "Browse folder")] {
            assert!(button.left() >= group.left() && button.top() >= group.top());
            assert!(
                button.right() <= group.right() + px(1.)
                    && button.bottom() <= group.bottom() + px(1.)
            );
            let scale = button.size.height / px(64.);
            let text_end = button.left()
                + px((85. + Theme::bundled().measure(label, 36, 0.8)[0] + 8.) * scale);
            assert!(
                text_end <= button.right() - px(22. * scale),
                "{label} exceeds the skinned button at width {width}"
            );
            cx.simulate_click(button.center(), Default::default());
        }
        assert_eq!((files.get(), folders.get()), (index + 1, index + 1));
        if width < 600. {
            assert!(file.bottom() <= folder.top());
        } else {
            assert_eq!(file.top(), folder.top());
            assert!(file.right() <= folder.left());
        }
    }
}
