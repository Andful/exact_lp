use num::{Num, Signed};

use crate::expression::Expression;

/**
 * Contraint of an integer program
 */
#[derive(Clone)]
pub struct Constraint<N>
where
    N: Num + Clone,
{
    pub(crate) lhs: Expression<N>,
    pub(crate) ord: std::cmp::Ordering,
    pub(crate) rhs: Expression<N>,
}

impl<N> Constraint<N>
where
    N: Num + Clone + std::fmt::Display + Signed,
{
    pub fn to_normalized(self) -> Constraint<N> {
        let Constraint { lhs, ord, rhs } = self;

        let new_lhs = lhs - rhs;
        let v = new_lhs.0.clone().into_iter().filter(|e| e.1.is_some()).collect::<Vec<_>>();
        let c = new_lhs.0.into_iter().filter(|e| e.1.is_none()).fold(N::zero(), |r, (b, _)| r + b);
        Self {
            lhs: Expression(v),
            ord,
            rhs: Expression::from(-c)
        }
    }
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
