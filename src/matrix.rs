use fxhash::FxHashMap;
use splitmut::SplitMut;
use std::{fmt, hash::Hash};

use crate::{BitSet, IndexSet, IndexedDomain, IndexedValue, PointerFamily, ToIndex};

/// An unordered collections of pairs `(R, C)`, implemented with a sparse bit-matrix.
///
/// "Sparse" means "hash map from rows to bit-sets of columns". Subsequently, only column types `C` must be indexed,
/// while row types `R` only need be hashable.
pub struct IndexMatrix<R, C: IndexedValue, S: BitSet, P: PointerFamily> {
    pub(crate) matrix: FxHashMap<R, IndexSet<C, S, P>>,
    empty_set: IndexSet<C, S, P>,
    col_domain: P::Pointer<IndexedDomain<C>>,
}

impl<R, C, S, P> IndexMatrix<R, C, S, P>
where
    R: PartialEq + Eq + Hash + Clone,
    C: IndexedValue,
    S: BitSet,
    P: PointerFamily,
{
    /// Creates an empty matrix.
    pub fn new(col_domain: &P::Pointer<IndexedDomain<C>>) -> Self {
        IndexMatrix {
            matrix: FxHashMap::default(),
            empty_set: IndexSet::new(col_domain),
            col_domain: col_domain.clone(),
        }
    }

    pub(crate) fn ensure_row(&mut self, row: R) -> &mut IndexSet<C, S, P> {
        self.matrix
            .entry(row)
            .or_insert_with(|| self.empty_set.clone())
    }

    /// Inserts a pair `(row, col)` into the matrix, returning true if `self` changed.
    pub fn insert<M>(&mut self, row: R, col: impl ToIndex<C, M>) -> bool {
        let col = col.to_index(&self.col_domain);
        self.ensure_row(row).insert(col)
    }

    /// Adds all elements of `from` into the row `into`.
    pub fn union_into_row(&mut self, into: R, from: &IndexSet<C, S, P>) -> bool {
        self.ensure_row(into).union_changed(from)
    }

    /// Adds all elements from the row `from` into the row `into`.
    pub fn union_rows(&mut self, from: R, to: R) -> bool {
        if from == to {
            return false;
        }

        self.ensure_row(from.clone());
        self.ensure_row(to.clone());

        // SAFETY: `from` != `to` therefore this is a disjoint mutable borrow
        let (from, to) = unsafe { self.matrix.get2_unchecked_mut(&from, &to) };
        to.union_changed(from)
    }

    /// Returns an iterator over the elements in `row`.
    pub fn row(&self, row: &R) -> impl Iterator<Item = &C> + '_ {
        self.matrix.get(row).into_iter().flat_map(|set| set.iter())
    }

    /// Returns an iterator over all rows in the matrix.
    pub fn rows(&self) -> impl Iterator<Item = (&R, &IndexSet<C, S, P>)> + '_ {
        self.matrix.iter()
    }

    /// Returns the [`IndexSet`] for a particular `row`.
    pub fn row_set(&self, row: &R) -> &IndexSet<C, S, P> {
        self.matrix.get(row).unwrap_or(&self.empty_set)
    }

    /// Clears all the elements from the `row`.
    pub fn clear_row(&mut self, row: &R) {
        self.matrix.remove(row);
    }

    /// Returns the [`IndexedDomain`] for the column type.
    pub fn col_domain(&self) -> &P::Pointer<IndexedDomain<C>> {
        &self.col_domain
    }
}

impl<R, C, S, P> PartialEq for IndexMatrix<R, C, S, P>
where
    R: PartialEq + Eq + Hash,
    C: IndexedValue,
    S: BitSet,
    P: PointerFamily,
{
    fn eq(&self, other: &Self) -> bool {
        self.matrix == other.matrix
    }
}
impl<R, C, S, P> Eq for IndexMatrix<R, C, S, P>
where
    R: PartialEq + Eq + Hash,
    C: IndexedValue,
    S: BitSet,
    P: PointerFamily,
{
}

impl<R, C, S, P> Clone for IndexMatrix<R, C, S, P>
where
    R: PartialEq + Eq + Hash + Clone,
    C: IndexedValue,
    S: BitSet,
    P: PointerFamily,
{
    fn clone(&self) -> Self {
        Self {
            matrix: self.matrix.clone(),
            empty_set: self.empty_set.clone(),
            col_domain: self.col_domain.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        for col in self.matrix.values_mut() {
            col.clear();
        }

        for (row, col) in source.matrix.iter() {
            self.ensure_row(row.clone()).clone_from(col);
        }

        self.empty_set = source.empty_set.clone();
        self.col_domain = source.col_domain.clone();
    }
}

impl<R, C, S, P> fmt::Debug for IndexMatrix<R, C, S, P>
where
    R: PartialEq + Eq + Hash + Clone + fmt::Debug,
    C: IndexedValue + fmt::Debug,
    S: BitSet,
    P: PointerFamily,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.rows()).finish()
    }
}

#[cfg(test)]
mod test {
    use crate::{test_utils::TestIndexMatrix, IndexedDomain};
    use std::rc::Rc;

    #[test]
    fn test_indexmatrix() {
        let col_domain = Rc::new(IndexedDomain::from_iter(["a", "b", "c"]));
        let mut mtx = TestIndexMatrix::new(&col_domain);
        mtx.insert(0, "b");
        mtx.insert(1, "c");
        assert_eq!(mtx.row(&0).collect::<Vec<_>>(), vec![&"b"]);
        assert_eq!(mtx.row(&1).collect::<Vec<_>>(), vec![&"c"]);

        assert!(mtx.union_rows(0, 1));
        assert_eq!(mtx.row(&1).collect::<Vec<_>>(), vec![&"b", &"c"]);
    }
}
