#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum TabId {
    #[default]
    Launcher,
    Nams,
    LodMod,
    Textures,
    YorhaProtocol,
    Mods,
    Naiom,
    Cutscenes,
    LauncherSettings,
    ThirdParty,
    Docs,
    Tools,
}

impl TabId {
    pub const ALL: [TabId; 12] = [
        TabId::Launcher,
        TabId::Nams,
        TabId::LodMod,
        TabId::Textures,
        TabId::YorhaProtocol,
        TabId::Mods,
        TabId::Naiom,
        TabId::Cutscenes,
        TabId::LauncherSettings,
        TabId::ThirdParty,
        TabId::Docs,
        TabId::Tools,
    ];

    pub const fn index(self) -> usize {
        match self {
            TabId::Launcher => 0,
            TabId::Nams => 1,
            TabId::LodMod => 2,
            TabId::Textures => 3,
            TabId::YorhaProtocol => 4,
            TabId::Mods => 5,
            TabId::Naiom => 6,
            TabId::Cutscenes => 7,
            TabId::LauncherSettings => 8,
            TabId::ThirdParty => 9,
            TabId::Docs => 10,
            TabId::Tools => 11,
        }
    }

    pub fn from_index(index: usize) -> Option<TabId> {
        TabId::ALL.into_iter().find(|tab| tab.index() == index)
    }

    pub const fn overlay(self) -> Option<OverlayId> {
        match self {
            TabId::Docs => Some(OverlayId::Docs),
            _ => None,
        }
    }

    pub const fn section(self) -> SidebarSection {
        match self {
            TabId::Launcher | TabId::LauncherSettings => SidebarSection::General,
            TabId::Mods
            | TabId::Textures
            | TabId::Cutscenes
            | TabId::ThirdParty
            | TabId::Docs
            | TabId::Tools => SidebarSection::Mods,
            TabId::Nams | TabId::YorhaProtocol | TabId::LodMod | TabId::Naiom => {
                SidebarSection::Config
            }
        }
    }

    pub const fn is_tour_anchor(self) -> bool {
        matches!(self, TabId::Textures | TabId::Cutscenes)
    }

    pub const fn tour_anchor_id(self) -> &'static str {
        match self {
            TabId::Launcher => "sidebar.tab.launcher",
            TabId::Nams => "sidebar.tab.nams",
            TabId::LodMod => "sidebar.tab.lodmod",
            TabId::Textures => "sidebar.tab.textures",
            TabId::YorhaProtocol => "sidebar.tab.yorhaprotocol",
            TabId::Mods => "sidebar.tab.mods",
            TabId::Naiom => "sidebar.tab.naiom",
            TabId::Cutscenes => "sidebar.tab.cutscenes",
            TabId::LauncherSettings => "sidebar.tab.launchersettings",
            TabId::ThirdParty => "sidebar.tab.thirdparty",
            TabId::Docs => "sidebar.tab.docs",
            TabId::Tools => "sidebar.tab.tools",
        }
    }

    pub const fn needs_game_dir(self) -> bool {
        !matches!(
            self,
            TabId::Launcher | TabId::LauncherSettings | TabId::Docs
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SidebarSection {
    General,
    Mods,
    Config,
}

impl SidebarSection {
    pub const ALL: [SidebarSection; 3] = [
        SidebarSection::General,
        SidebarSection::Mods,
        SidebarSection::Config,
    ];

    pub const fn tabs(self) -> &'static [TabId] {
        match self {
            SidebarSection::General => &[TabId::Launcher, TabId::LauncherSettings],
            SidebarSection::Mods => &[
                TabId::Mods,
                TabId::Textures,
                TabId::Cutscenes,
                TabId::ThirdParty,
                TabId::Docs,
                TabId::Tools,
            ],
            SidebarSection::Config => &[
                TabId::Nams,
                TabId::YorhaProtocol,
                TabId::LodMod,
                TabId::Naiom,
            ],
        }
    }

    pub const fn collapsible(self) -> bool {
        matches!(self, SidebarSection::Config)
    }

    pub const fn collapsed_by_default(self) -> bool {
        matches!(self, SidebarSection::Config)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OverlayId {
    Docs,
    Inspector,
}
