use num::{Num, Signed};

use crate::expression::Expression;

/**
 * Contraint of an integer program
 */
pub struct Constraint<N>
where
    N: Num + Clone,
{
    pub(crate) lhs: Expression<N>,
    pub(crate) ord: std::cmp::Ordering,
    pub(crate) rhs: Expression<N>,
}

impl<N> std::fmt::Display for Constraint<N>
where
    N: Num + Clone + std::fmt::Display + Signed,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.lhs))?;
        match self.ord {
            std::cmp::Ordering::Greater => f.write_str(" >= ")?,
            std::cmp::Ordering::Equal => f.write_str(" = ")?,
            std::cmp::Ordering::Less => f.write_str(" <= ")?,
        }
        f.write_fmt(format_args!("{}", self.rhs))
    }
}
