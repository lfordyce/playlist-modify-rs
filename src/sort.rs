use crate::cli::Direction;
use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;
use std::cmp::Ordering;

pub struct SortField<T> {
    pub field: T,
    pub direction: Direction,
}
pub trait SortableByField<T> {
    fn sort(&self, rhs: &Self, field: &T) -> Ordering;
}

pub fn by_fields<T>(f: &mut [impl SortableByField<T>], sort_fields: &[SortField<T>]) {
    f.sort_by(|a, b| {
        sort_fields
            .iter()
            .fold_while(Ordering::Equal, |cmp, f| match cmp {
                Ordering::Equal => {
                    let cmp = a.sort(b, &f.field);
                    Continue(match f.direction {
                        Direction::Desc => cmp.reverse(),
                        _ => cmp,
                    })
                }
                _ => Done(cmp),
            })
            .into_inner()
    });
}
