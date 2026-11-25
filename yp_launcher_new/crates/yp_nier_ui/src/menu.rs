use gpui::SharedString;

#[derive(Clone, Debug)]
pub struct MenuEntry {
    pub label: SharedString,
    pub help: SharedString,
    pub enabled: bool,
    pub children: Vec<MenuEntry>,
    pub confirmation: Option<SharedString>,
}

impl MenuEntry {
    pub fn new(label: impl Into<SharedString>, help: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            help: help.into(),
            enabled: true,
            children: Vec::new(),
            confirmation: None,
        }
    }
    pub fn children(mut self, entries: Vec<Self>) -> Self {
        self.children = entries;
        self
    }
    pub fn confirm(mut self, prompt: impl Into<SharedString>) -> Self {
        self.confirmation = Some(prompt.into());
        self
    }
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Clone, Debug)]
pub struct MenuPage {
    pub label: SharedString,
    pub help: SharedString,
    pub icon: Option<SharedString>,
    pub entries: Vec<MenuEntry>,
}
impl MenuPage {
    pub fn new(
        label: impl Into<SharedString>,
        help: impl Into<SharedString>,
        entries: Vec<MenuEntry>,
    ) -> Self {
        Self {
            label: label.into(),
            help: help.into(),
            entries,
            icon: None,
        }
    }
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuInput {
    Left,
    Right,
    Up,
    Down,
    Confirm,
    Back,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuAction {
    Activate { page: usize, path: Vec<usize> },
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Tabs,
    Rows,
    Dialog,
}

pub struct PauseMenu {
    pub(crate) pages: Vec<MenuPage>,
    pub(crate) page: usize,
    pub(crate) path: Vec<usize>,
    pub(crate) row: usize,
    pub(crate) focus: Focus,
    pub(crate) yes: bool,
}

impl PauseMenu {
    pub fn pages(&self) -> &[MenuPage] {
        &self.pages
    }
    pub fn page_index(&self) -> usize {
        self.page
    }
    pub fn row_index(&self) -> usize {
        self.row
    }
    pub fn focus(&self) -> Focus {
        self.focus
    }
    pub fn affirmative(&self) -> bool {
        self.yes
    }
    pub fn choose_answer(&mut self, yes: bool) {
        if self.focus == Focus::Dialog {
            self.yes = yes;
        }
    }
    pub fn new(pages: Vec<MenuPage>) -> Self {
        Self {
            pages,
            page: 0,
            path: Vec::new(),
            row: 0,
            focus: Focus::Tabs,
            yes: false,
        }
    }
    pub fn selected_page(&self) -> Option<&MenuPage> {
        self.pages.get(self.page)
    }
    pub fn entries(&self) -> &[MenuEntry] {
        let Some(page) = self.selected_page() else {
            return &[];
        };
        let mut entries = page.entries.as_slice();
        for &index in &self.path {
            let Some(entry) = entries.get(index) else {
                return &[];
            };
            entries = &entry.children;
        }
        entries
    }
    pub fn selected_entry(&self) -> Option<&MenuEntry> {
        self.entries().get(self.row)
    }
    pub fn title(&self) -> &str {
        let Some(page) = self.selected_page() else {
            return "";
        };
        let mut title = page.label.as_ref();
        let mut entries = page.entries.as_slice();
        for &index in &self.path {
            let Some(e) = entries.get(index) else { break };
            title = &e.label;
            entries = &e.children;
        }
        title
    }
    pub fn description(&self) -> &str {
        match self.focus {
            Focus::Tabs => self.selected_page().map(|p| p.help.as_ref()).unwrap_or(""),
            _ => self.selected_entry().map(|e| e.help.as_ref()).unwrap_or(""),
        }
    }
    fn first_enabled(&self) -> Option<usize> {
        self.entries().iter().position(|e| e.enabled)
    }
    pub fn tab(&mut self, i: usize) {
        if i >= self.pages.len() || self.focus == Focus::Dialog {
            return;
        }
        self.page = i;
        self.path.clear();
        self.focus = Focus::Tabs;
        self.row = self.first_enabled().unwrap_or(0);
    }
    pub fn select(&mut self, i: usize) {
        if self.focus != Focus::Dialog && self.entries().get(i).is_some_and(|e| e.enabled) {
            self.row = i;
            self.focus = Focus::Rows;
        }
    }
    fn advance(&mut self, step: isize) {
        let n = self.entries().len();
        for offset in 1..=n {
            let i = (self.row as isize + step * offset as isize).rem_euclid(n as isize) as usize;
            if self.entries()[i].enabled {
                self.row = i;
                break;
            }
        }
    }
    fn activated(&self) -> MenuAction {
        let mut path = self.path.clone();
        path.push(self.row);
        MenuAction::Activate {
            page: self.page,
            path,
        }
    }
    pub fn input(&mut self, key: MenuInput) -> Option<MenuAction> {
        use MenuInput::*;
        match (self.focus, key) {
            (Focus::Dialog, Left | Right) => self.yes = !self.yes,
            (Focus::Dialog, Back) if self.yes => self.yes = false,
            (Focus::Dialog, Back) => self.focus = Focus::Rows,
            (Focus::Dialog, Confirm) => {
                self.focus = Focus::Rows;
                if self.yes {
                    return Some(self.activated());
                }
            }
            (Focus::Tabs, Left | Right) if !self.pages.is_empty() => {
                let step = if key == Left { -1 } else { 1 };
                self.tab(
                    (self.page as isize + step).rem_euclid(self.pages.len() as isize) as usize,
                );
            }
            (Focus::Tabs, Confirm | Down) => {
                if let Some(first) = self.first_enabled() {
                    self.select(first);
                }
            }
            (Focus::Tabs, Back) => return Some(MenuAction::Close),
            (Focus::Rows, Down) => self.advance(1),
            (Focus::Rows, Up) => self.advance(-1),
            (Focus::Rows, Back) => {
                if let Some(parent) = self.path.pop() {
                    self.row = parent;
                } else {
                    self.focus = Focus::Tabs;
                }
            }
            (Focus::Rows, Confirm) => {
                let entry = self.selected_entry()?;
                if !entry.enabled {
                    return None;
                }
                if !entry.children.is_empty() {
                    self.path.push(self.row);
                    self.row = self.first_enabled().unwrap_or(0);
                } else if entry.confirmation.is_some() {
                    self.yes = false;
                    self.focus = Focus::Dialog;
                } else {
                    return Some(self.activated());
                }
            }
            _ => {}
        }
        None
    }
}
