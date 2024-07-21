use super::{Assembly, CILRoot, RootIdx};
use crate::basic_block::BasicBlock as V1Block;
pub struct BasicBlock {
    roots: Vec<RootIdx>,
    block_id: u32,
    handler: Option<Box<[Self]>>,
}

impl BasicBlock {
    pub fn new(roots: Vec<RootIdx>, block_id: u32, handler: Option<Box<[Self]>>) -> Self {
        Self {
            roots,
            block_id,
            handler,
        }
    }

    pub fn roots(&self) -> &[RootIdx] {
        &self.roots
    }

    pub fn block_id(&self) -> u32 {
        self.block_id
    }

    pub fn handler(&self) -> Option<&[BasicBlock]> {
        self.handler.as_ref().map(|b| b.as_ref())
    }
}
impl BasicBlock {
    pub fn from_v1(v1: &V1Block, asm: &mut Assembly) -> Self {
        let handler: Option<Box<[Self]>> = v1.handler().map(|handler| {
            handler
                .as_blocks()
                .unwrap()
                .iter()
                .map(|block| Self::from_v1(block, asm))
                .collect()
        });
        Self::new(
            v1.iter_tree_roots()
                .map(|root| {
                    let root = CILRoot::from_v1(root, asm);
                    asm.alloc_root(root)
                })
                .collect(),
            v1.id(),
            handler,
        )
    }
}
