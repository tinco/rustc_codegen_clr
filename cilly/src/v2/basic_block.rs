use serde::{Deserialize, Serialize};

use super::{opt, Assembly, CILNode, CILRoot, RootIdx};
use crate::basic_block::BasicBlock as V1Block;
#[derive(Hash, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct BasicBlock {
    roots: Vec<RootIdx>,
    block_id: u32,
    handler: Option<Vec<Self>>,
}

impl BasicBlock {
    #[must_use]
    pub fn new(roots: Vec<RootIdx>, block_id: u32, handler: Option<Vec<Self>>) -> Self {
        Self {
            roots,
            block_id,
            handler,
        }
    }

    #[must_use]
    pub fn roots(&self) -> &[RootIdx] {
        &self.roots
    }

    #[must_use]
    pub fn block_id(&self) -> u32 {
        self.block_id
    }
    pub fn iter_roots(&self) -> impl Iterator<Item = RootIdx> + '_ {
        let handler_iter: Box<dyn Iterator<Item = RootIdx>> = match self.handler() {
            Some(handler) => Box::new(handler.iter().flat_map(BasicBlock::iter_roots)),
            None => Box::new(std::iter::empty()),
        };
        self.roots().iter().copied().chain(handler_iter)
    }
    /// Remaps all the roots in this block using `root_map` and `node_root`
    /// Iterates trough the roots of this block and its handlers
    pub fn iter_roots_mut(&mut self) -> impl Iterator<Item = &mut RootIdx> + '_ {
        let handler_iter: Box<dyn Iterator<Item = &mut RootIdx>> = match self.handler.as_mut() {
            Some(handler) => Box::new(handler.iter_mut().flat_map(BasicBlock::iter_roots_mut)),
            None => Box::new(std::iter::empty()),
        };
        self.roots.iter_mut().chain(handler_iter)
    }
    /// Modifies all nodes and roots in this `BasicBlock`
    pub fn map_roots(
        &mut self,
        asm: &mut Assembly,
        root_map: &mut impl Fn(CILRoot, &mut Assembly) -> CILRoot,
        node_map: &mut impl Fn(CILNode, &mut Assembly) -> CILNode,
    ) {
        self.iter_roots_mut().for_each(|root| {
            let get_root = asm.get_root(*root).clone();
            let val = get_root.map(asm, root_map, node_map);
            *root = asm.alloc_root(val);
        });
    }
    #[must_use]
    pub fn handler(&self) -> Option<&[BasicBlock]> {
        self.handler.as_ref().map(std::convert::AsRef::as_ref)
    }
    pub fn handler_mut(&mut self) -> Option<&mut Vec<BasicBlock>> {
        self.handler.as_mut()
    }
    pub fn roots_mut(&mut self) -> &mut Vec<RootIdx> {
        &mut self.roots
    }
    pub fn handler_and_root_mut(&mut self) -> (Option<&mut [BasicBlock]>, &mut Vec<RootIdx>) {
        (
            self.handler.as_mut().map(std::convert::AsMut::as_mut),
            &mut self.roots,
        )
    }
    /// Checks if this basic block consists of nothing more than an unconditional jump to another block
    #[must_use]
    pub fn is_direct_jump(&self, asm: &Assembly) -> Option<(u32, u32)> {
        let mut meningfull_root = self.meaningfull_roots(asm);
        let root = meningfull_root.next()?;
        let CILRoot::Branch(binfo) = asm.get_root(root) else {
            return None;
        };
        if opt::is_branch_unconditional(binfo) && meningfull_root.next().is_none() {
            Some((binfo.0, binfo.1))
        } else {
            None
        }
    }
    /// Checks if this basic block consists of nothing more thaan an uncondtional rethrow
    #[must_use]
    pub fn is_only_rethrow(&self, asm: &Assembly) -> bool {
        let mut meningfull_root = self.meaningfull_roots(asm);
        let Some(root) = meningfull_root.next() else {
            return false;
        };
        CILRoot::ReThrow == *asm.get_root(root) && meningfull_root.next().is_none()
    }

    pub fn meaningfull_roots<'s, 'asm: 's>(
        &'s self,
        asm: &'asm Assembly,
    ) -> impl Iterator<Item = RootIdx> + 's {
        self.iter_roots().filter(move |root| {
            !matches!(
                asm.get_root(*root),
                CILRoot::Nop | CILRoot::SourceFileInfo { .. }
            )
        })
    }

    pub fn remove_handler(&mut self) {
        self.handler = None;
    }
}
impl BasicBlock {
    pub fn from_v1(v1: &V1Block, asm: &mut Assembly) -> Self {
        let handler: Option<Vec<Self>> = v1.handler().map(|handler| {
            handler
                .as_blocks()
                .unwrap()
                .iter()
                .map(|block| Self::from_v1(block, asm))
                .collect()
        });
        Self::new(
            v1.trees()
                .iter()
                .map(|root| {
                    let root = CILRoot::from_v1(root.root(), asm);
                    asm.alloc_root(root)
                })
                .collect(),
            v1.id(),
            handler,
        )
    }
}
#[test]
fn is_direct_jump() {
    let asm = &mut Assembly::default();
    let block = BasicBlock::new(vec![], 0, None);
    // A Block which is empty is not a direwct jump anywhere.'
    assert!(block.is_direct_jump(asm).is_none());
}
#[test]
fn is_only_rethrow() {
    let asm = &mut Assembly::default();
    let block = BasicBlock::new(vec![], 0, None);
    // A Block which is empty is not a rethrow.
    assert!(!block.is_only_rethrow(asm));
    let rethrow = asm.alloc_root(CILRoot::ReThrow);
    let block = BasicBlock::new(vec![rethrow], 0, None);
    // A Block which is just a rethrow is, well, a rethrow.
    assert!(block.is_only_rethrow(asm));
    let dbg_break = asm.alloc_root(CILRoot::Break);
    let block = BasicBlock::new(vec![dbg_break, rethrow], 0, None);
    // A dbg break has side effects, this should return false
    assert!(!block.is_only_rethrow(asm));
    let dbg_break = asm.alloc_root(CILRoot::Break);
    let block = BasicBlock::new(vec![rethrow, dbg_break], 0, None);
    // A dbf break has side effects, this should return false
    assert!(!block.is_only_rethrow(asm));
}
