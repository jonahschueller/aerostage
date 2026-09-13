use crate::{aerospace::AerospaceLayout, stage::StageWorkspaceLayout};

impl From<AerospaceLayout> for StageWorkspaceLayout {
    fn from(value: AerospaceLayout) -> Self {
        match value {
            AerospaceLayout::HTiles => StageWorkspaceLayout::HTiles,
            AerospaceLayout::VTiles => StageWorkspaceLayout::VTiles,
            AerospaceLayout::HAccordion => StageWorkspaceLayout::HAccordion,
            AerospaceLayout::VAccordion => StageWorkspaceLayout::VAccordion,
        }
    }
}

impl From<StageWorkspaceLayout> for AerospaceLayout {
    fn from(value: StageWorkspaceLayout) -> Self {
        match value {
            StageWorkspaceLayout::HTiles => AerospaceLayout::HTiles,
            StageWorkspaceLayout::VTiles => AerospaceLayout::VTiles,
            StageWorkspaceLayout::HAccordion => AerospaceLayout::HAccordion,
            StageWorkspaceLayout::VAccordion => AerospaceLayout::VAccordion,
        }
    }
}
