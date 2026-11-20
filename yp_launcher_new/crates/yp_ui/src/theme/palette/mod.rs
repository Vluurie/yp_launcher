mod dark;
mod nier;

use crate::theme::{ThemeColors, ThemeId};

pub(crate) fn colors(id: ThemeId) -> ThemeColors {
    match id {
        ThemeId::Dark => dark::colors(),
        ThemeId::Nier => nier::colors(),
    }
}
