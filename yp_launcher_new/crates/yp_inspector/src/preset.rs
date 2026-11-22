use std::collections::BTreeSet;

use crate::session::ModelMesh;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MeshSelection {
    pub custom_assembly: bool,
    pub visible_names: BTreeSet<String>,
    pub force_hide: BTreeSet<String>,
    pub force_show: BTreeSet<String>,
}

pub fn target_meshes(
    meshes: &[ModelMesh],
    selection: &MeshSelection,
    default_visible: &BTreeSet<u32>,
) -> BTreeSet<u32> {
    if selection.custom_assembly {
        return meshes
            .iter()
            .filter(|mesh| selection.visible_names.contains(&mesh.name))
            .map(|mesh| mesh.id)
            .collect();
    }
    let mut target = default_visible.clone();
    for mesh in meshes {
        if selection.force_hide.contains(&mesh.name) {
            target.remove(&mesh.id);
        }
        if selection.force_show.contains(&mesh.name) {
            target.insert(mesh.id);
        }
    }
    target
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mesh(id: u32, name: &str) -> ModelMesh {
        ModelMesh {
            id,
            name: name.to_string(),
            lod_name: "LOD0".to_string(),
            lod_index: 0,
            visible: true,
        }
    }

    fn names(values: &[&str]) -> BTreeSet<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn custom_assembly_shows_exactly_the_named_meshes() {
        let meshes = [
            mesh(0, "Body"),
            mesh(1, "Hair"),
            mesh(2, "Body"),
            mesh(3, "Skirt"),
        ];
        let selection = MeshSelection {
            custom_assembly: true,
            visible_names: names(&["Body", "Skirt"]),
            force_hide: names(&["Body"]),
            force_show: names(&["Hair"]),
        };
        let target = target_meshes(&meshes, &selection, &BTreeSet::from([1]));
        assert_eq!(target, BTreeSet::from([0, 2, 3]));
    }

    #[test]
    fn overrides_start_from_the_default_visibility() {
        let meshes = [
            mesh(0, "Body"),
            mesh(1, "Armor"),
            mesh(2, "Hair"),
            mesh(3, "Both"),
        ];
        let selection = MeshSelection {
            custom_assembly: false,
            visible_names: names(&["ignored"]),
            force_hide: names(&["Hair", "Both"]),
            force_show: names(&["Armor", "Both"]),
        };
        let target = target_meshes(&meshes, &selection, &BTreeSet::from([0, 2]));
        assert_eq!(target, BTreeSet::from([0, 1, 3]));
    }
}
