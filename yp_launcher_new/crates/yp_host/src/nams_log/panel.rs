use std::collections::{BTreeMap, BTreeSet};

use super::{LogEntry, LogLevel, TailUpdate};

pub const MAX_ENTRIES_PER_TAB: usize = 5000;
pub const LOG_LEVELS: [LogLevel; 4] = [LogLevel::Error, LogLevel::Warn, LogLevel::Info, LogLevel::Debug];
pub const DEFAULT_HIDDEN_LEVELS: [LogLevel; 2] = [LogLevel::Info, LogLevel::Debug];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LogTab {
    #[default]
    Modloader,
    Yorha,
}

impl LogTab {
    pub fn id(self) -> &'static str {
        match self {
            LogTab::Modloader => yp_core::consts::TAB_MODLOADER_ID,
            LogTab::Yorha => yp_core::consts::TAB_YORHA_ID,
        }
    }
}

pub fn is_modloader_module(module: &str) -> bool {
    !module.split("::").next().is_some_and(|root| root.eq_ignore_ascii_case("yorha_protocol"))
}

pub fn append_capped(current: &mut Vec<LogEntry>, added: Vec<LogEntry>) {
    if added.is_empty() {
        return;
    }
    if added.len() >= MAX_ENTRIES_PER_TAB {
        let skip = added.len() - MAX_ENTRIES_PER_TAB;
        current.clear();
        current.extend(added.into_iter().skip(skip));
        return;
    }
    let total = current.len() + added.len();
    if total > MAX_ENTRIES_PER_TAB {
        current.drain(..total - MAX_ENTRIES_PER_TAB);
    }
    current.extend(added);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogPanel {
    pub modloader_entries: Vec<LogEntry>,
    pub yorha_entries: Vec<LogEntry>,
    pub is_loading: bool,
    pub active_tab: LogTab,
    pub search_query: String,
    pub hidden_levels: BTreeSet<LogLevel>,
}

impl Default for LogPanel {
    fn default() -> Self {
        Self {
            modloader_entries: Vec::new(),
            yorha_entries: Vec::new(),
            is_loading: false,
            active_tab: LogTab::Modloader,
            search_query: String::new(),
            hidden_levels: DEFAULT_HIDDEN_LEVELS.into_iter().collect(),
        }
    }
}

impl LogPanel {
    pub fn active_entries(&self) -> &[LogEntry] {
        match self.active_tab {
            LogTab::Modloader => &self.modloader_entries,
            LogTab::Yorha => &self.yorha_entries,
        }
    }

    pub fn is_level_shown(&self, level: LogLevel) -> bool {
        !self.hidden_levels.contains(&level)
    }

    pub fn level_counts(&self) -> BTreeMap<LogLevel, usize> {
        let mut counts = BTreeMap::new();
        for entry in self.active_entries() {
            *counts.entry(entry.level).or_insert(0) += 1;
        }
        counts
    }

    pub fn filtered_entries(&self) -> Vec<&LogEntry> {
        let query = self.search_query.trim().to_lowercase();
        self.active_entries()
            .iter()
            .filter(|entry| !self.hidden_levels.contains(&entry.level))
            .filter(|entry| {
                query.is_empty()
                    || entry.level.as_str().to_lowercase().contains(&query)
                    || entry.module.to_lowercase().contains(&query)
                    || entry.message.to_lowercase().contains(&query)
            })
            .collect()
    }

    pub fn reset(&mut self) {
        self.modloader_entries.clear();
        self.yorha_entries.clear();
    }

    pub fn apply_from(&mut self, tab: LogTab, update: TailUpdate) {
        let entries = match tab {
            LogTab::Modloader => &mut self.modloader_entries,
            LogTab::Yorha => &mut self.yorha_entries,
        };
        match update {
            TailUpdate::Unchanged => {}
            TailUpdate::Reset => entries.clear(),
            TailUpdate::Appended(added) => append_capped(entries, added),
        }
    }

    pub fn apply(&mut self, update: TailUpdate) {
        match update {
            TailUpdate::Unchanged => {}
            TailUpdate::Reset => self.reset(),
            TailUpdate::Appended(entries) => {
                let (modloader, yorha): (Vec<LogEntry>, Vec<LogEntry>) =
                    entries.into_iter().partition(|entry| is_modloader_module(&entry.module));
                append_capped(&mut self.modloader_entries, modloader);
                append_capped(&mut self.yorha_entries, yorha);
            }
        }
    }

    pub fn set_active_tab(&mut self, tab: LogTab) {
        self.active_tab = tab;
    }

    pub fn set_search_query(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
    }

    pub fn toggle_level(&mut self, level: LogLevel) {
        if !self.hidden_levels.remove(&level) {
            self.hidden_levels.insert(level);
        }
    }
}
