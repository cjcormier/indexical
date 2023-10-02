extern crate rustc_driver;
pub extern crate rustc_index;
extern crate rustc_mir_dataflow;

use crate::{
    ArcFamily, BitSet, IndexMatrix, IndexSet, IndexedValue, PointerFamily, RcFamily, RefFamily,
};
use rustc_mir_dataflow::JoinSemiLattice;
use std::hash::Hash;

pub type RustcBitSet = rustc_index::bit_set::BitSet<usize>;

impl BitSet for RustcBitSet {
    type Iter<'a> = rustc_index::bit_set::BitIter<'a, usize>;

    fn empty(size: usize) -> Self {
        RustcBitSet::new_empty(size)
    }

    fn contains(&self, index: usize) -> bool {
        self.contains(index)
    }

    fn insert(&mut self, index: usize) -> bool {
        self.insert(index)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    fn intersect(&mut self, other: &Self) {
        self.intersect(other);
    }

    fn intersect_changed(&mut self, other: &Self) -> bool {
        self.intersect(other)
    }

    fn len(&self) -> usize {
        self.count()
    }

    fn union(&mut self, other: &Self) {
        self.union(other);
    }

    fn union_changed(&mut self, other: &Self) -> bool {
        self.union(other)
    }

    fn subtract(&mut self, other: &Self) {
        self.subtract(other);
    }

    fn subtract_changed(&mut self, other: &Self) -> bool {
        self.subtract(other)
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn invert(&mut self) {
        let mut inverted = RustcBitSet::new_filled(self.domain_size());
        inverted.subtract(self);
        *self = inverted;
    }

    fn insert_all(&mut self) {
        self.insert_all();
    }

    fn copy_from(&mut self, other: &Self) {
        self.clone_from(other);
    }
}

/// [`IndexSet`] specialized to the `rustc_index::bit_set::BitSet` implementation.
pub type RustcIndexSet<T> = IndexSet<T, RustcBitSet, RcFamily>;

/// [`IndexSet`] specialized to the `rustc_index::bit_set::BitSet` implementation with the [`ArcFamily`].
pub type RustcArcIndexSet<T> = IndexSet<T, RustcBitSet, ArcFamily>;

/// [`IndexSet`] specialized to the `rustc_index::bit_set::BitSet` implementation with the [`RefFamily`].
pub type RustcRefIndexSet<'a, T> = IndexSet<T, RustcBitSet, RefFamily<'a>>;

/// [`IndexMatrix`] specialized to the `rustc_index::bit_set::BitSet` implementation.
pub type RustcIndexMatrix<R, C> = IndexMatrix<R, C, RustcBitSet, RcFamily>;

/// [`IndexMatrix`] specialized to the `rustc_index::bit_set::BitSet` implementation with the [`ArcFamily`].
pub type RustcArcIndexMatrix<R, C> = IndexMatrix<R, C, RustcBitSet, ArcFamily>;

/// [`IndexMatrix`] specialized to the `rustc_index::bit_set::BitSet` implementation with the [`RefFamily`].
pub type RustcRefIndexMatrix<'a, R, C> = IndexMatrix<R, C, RustcBitSet, RefFamily<'a>>;

impl<T, S, P> JoinSemiLattice for IndexSet<T, S, P>
where
    T: IndexedValue,
    S: BitSet,
    P: PointerFamily,
{
    fn join(&mut self, other: &Self) -> bool {
        self.union_changed(other)
    }
}

impl<R, C, S, P> JoinSemiLattice for IndexMatrix<R, C, S, P>
where
    R: PartialEq + Eq + Hash + Clone,
    C: IndexedValue,
    S: BitSet,
    P: PointerFamily,
{
    fn join(&mut self, other: &Self) -> bool {
        let mut changed = false;
        for (row, col) in other.matrix.iter() {
            changed |= self.ensure_row(row.clone()).union_changed(col);
        }
        changed
    }
}

#[test]
fn test_rustc_bitset() {
    crate::test_utils::impl_test::<RustcBitSet>();
}
