use crate::instruction::NestedInstructionResolutionStrategy;
use datex_macros_internal::Datex;
use serde::{Deserialize, Serialize};

#[derive(Datex, Debug, Serialize, Deserialize)]
#[datex(structural)]
pub struct DisassemblerOptions {
    #[serde(default)]
    pub tree: bool,
    #[serde(default)]
    pub colorized: bool,
    #[serde(default)]
    pub recursive: bool,
}

impl DisassemblerOptions {
    pub fn simple() -> DisassemblerOptions {
        DisassemblerOptions {
            tree: false,
            colorized: false,
            recursive: false,
        }
    }

    pub(crate) fn nested_instructions_resolution_strategy(
        &self,
    ) -> NestedInstructionResolutionStrategy {
        if self.recursive {
            NestedInstructionResolutionStrategy::ResolveNestedScopesTree // always resolve as tree, collapse later if needed for string display
        } else {
            NestedInstructionResolutionStrategy::None
        }
    }
}

impl Default for DisassemblerOptions {
    fn default() -> DisassemblerOptions {
        DisassemblerOptions {
            tree: true,
            colorized: true,
            recursive: true,
        }
    }
}
