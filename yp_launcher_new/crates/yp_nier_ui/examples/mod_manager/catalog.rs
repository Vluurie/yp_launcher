use yp_nier_ui::{MenuEntry, MenuPage, PauseMenu};

pub struct Mod {
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub enabled: bool,
}

pub fn catalog() -> Vec<Mod> {
    [
        (
            "Interface Pack",
            "Interface",
            "Replaces menu icons and button prompts.",
            true,
        ),
        (
            "Costume Collection",
            "Appearance",
            "Adds alternate outfits for playable characters.",
            false,
        ),
        (
            "Camera Controls",
            "Camera",
            "Adds adjustable camera distance and field of view.",
            true,
        ),
        (
            "Lighting Preset",
            "Graphics",
            "Changes the lighting profile used during play.",
            false,
        ),
        (
            "Pod Skins",
            "Appearance",
            "Adds alternate finishes for your support pod.",
            true,
        ),
        (
            "Sound Mix",
            "Audio",
            "Adjusts the balance between music and effects.",
            false,
        ),
        (
            "Photo Mode",
            "Camera",
            "Adds camera controls for still photographs.",
            false,
        ),
        (
            "Controller Icons",
            "Interface",
            "Changes the button glyphs shown in menus.",
            true,
        ),
        (
            "Texture Pack",
            "Graphics",
            "Replaces selected character and object textures.",
            false,
        ),
    ]
    .into_iter()
    .map(|(name, category, description, enabled)| Mod {
        name,
        category,
        description,
        enabled,
    })
    .collect()
}

pub const MODELS: [(&str, &str); 2] = [
    (
        "Plug-in Chip",
        "A square processor module mounted on a ceramic carrier. Contacts along both edges connect the chip to its housing.",
    ),
    (
        "Memory Module",
        "Three memory packages share a two-sided connector. The inspector can display either finish and rotate the assembly.",
    ),
];

pub fn navigation(mods: &[Mod]) -> PauseMenu {
    PauseMenu::new(vec![
        MenuPage::new(
            "MODS",
            "Browse installed mods and change their status.",
            mods.iter()
                .map(|m| {
                    MenuEntry::new(m.name, m.description)
                        .confirm(format!("Change the status of {}?", m.name))
                })
                .collect(),
        )
        .icon("inventory"),
        MenuPage::new(
            "PROFILES",
            "Apply a group of mods to this session.",
            vec![
                MenuEntry::new("All mods", "Enable every mod in the list.")
                    .confirm("Enable all mods?"),
                MenuEntry::new("Unmodified", "Disable every mod in the list.")
                    .confirm("Disable all mods?"),
            ],
        )
        .icon("profiles"),
        MenuPage::new(
            "SETTINGS",
            "Choose a menu display effect.",
            vec![
                MenuEntry::new("Menu display", "Use the pause menu display."),
                MenuEntry::new("Eye loupe", "Use the radial lens display."),
                MenuEntry::new("Grid noise", "Use the stepped color display."),
                MenuEntry::new(
                    "Hacking static",
                    "Preview the hacking display interference.",
                ),
                MenuEntry::new("CRT lens", "Preview the cathode-ray lens curvature."),
                MenuEntry::new(
                    "Chromatic lens",
                    "Preview the separate red, green and blue channels.",
                ),
            ],
        )
        .icon("settings"),
        MenuPage::new(
            "INSTALL",
            "Review mod packages and installation activity.",
            vec![
                MenuEntry::new("Packages", "Drop a mod archive or folder to inspect it."),
                MenuEntry::new("Loading indicator", "Preview the loading animation."),
                MenuEntry::new("Saving indicator", "Preview the save animation."),
            ],
        )
        .icon("install"),
        MenuPage::new(
            "INTEL",
            "Select unit to view.",
            MODELS.iter().map(|&(n, d)| MenuEntry::new(n, d)).collect(),
        )
        .icon("profiles"),
    ])
}

pub const RESOLUTIONS: [&str; 23] = [
    "640x480",
    "720x480",
    "720x576",
    "800x600",
    "1024x576",
    "1024x768",
    "1152x864",
    "1176x664",
    "1280x720",
    "1280x768",
    "1280x800",
    "1280x960",
    "1280x1024",
    "1366x768",
    "1600x900",
    "1600x1024",
    "1600x1200",
    "1680x1050",
    "1920x1080",
    "1920x1200",
    "1920x1440",
    "2560x1440",
    "3840x2160",
];
