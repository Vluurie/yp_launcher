# How to turn a mod off without deleting it

`nams\disabled_mods.toml`:

```toml
disabled = [
  "mods/my_first_mod",
]
```

Rules:

- Paths are relative to the `nams\` folder.
- Use forward slashes.
- Listing a folder also disables everything inside it.
- A disabled mod folder is skipped completely, its `data\`,
  `cutscenes\`, `textures\` and plugin included.
- Restart after editing.

At startup the log shows `[NAMS] skipping disabled mod: "<folder>"`
for every mod folder on the list. NAMS creates the file with an empty
list when it is missing. A file with a syntax error logs
`[NAMS] disabled_mods.toml has a syntax error: <message>` and the list
NAMS had before stays in use; at startup that list is empty.
