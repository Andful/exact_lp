use crate::{constraint::Constraint, variable::Variable};
use num::{Num, Signed};
use std::{fmt::Display, ops::*};

#[derive(Clone)]
pub struct Expression<N>(pub(crate) Vec<(N, Option<Variable<N>>)>)
where
    N: Num + Clone;

impl<N> Expression<N>
where
    N: Num + Clone,
{
    pub fn le(self, expr: impl Into<Expression<N>>) -> Constraint<N> {
        Constraint {
            lhs: self,
            ord: std::cmp::Ordering::Less,
            rhs: expr.into(),
        }
    }

    pub fn eq(self, expr: impl Into<Expression<N>>) -> Constraint<N> {
        Constraint {
            lhs: self,
            ord: std::cmp::Ordering::Equal,
            rhs: expr.into(),
        }
    }

    pub fn ge(self, expr: impl Into<Expression<N>>) -> Constraint<N> {
        Constraint {
            lhs: self,
            ord: std::cmp::Ordering::Greater,
            rhs: expr.into(),
        }
    }
}

impl<N> std::fmt::Display for Expression<N>
where
    N: Num + Clone + Display + Signed,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.iter().peekable();

        let Some(e) = iter.next() else { return Ok(()) };

        f.write_fmt(format_args!("{:.64}", e.0))?;
        if let Some(v) = &e.1 {
            f.write_fmt(format_args!(" {}", v))?;
        }
        for e in iter {
            if e.0 == e.0.abs() {
                f.write_fmt(format_args!(" + {:.64}", e.0))?;
            } else {
                f.write_fmt(format_args!(" - {:.64}", e.0.abs()))?;
            }
            if let Some(v) = &e.1 {
                f.write_fmt(format_args!(" {}", v))?;
            }
        }
        Ok(())
    }
}

impl<N> Default for Expression<N>
where
    N: Num + Clone,
{
    fn default() -> Self {
        Expression(vec![])
    }
}

impl<N> From<N> for Expression<N>
where
    N: Num + Clone,
{
    fn from(bias: N) -> Self {
        Expression(vec![(bias, None)])
    }
}

impl<N> From<Variable<N>> for Expression<N>
where
    N: Num + Clone,
{
    fn from(v: Variable<N>) -> Self {
        Expression(vec![(N::one(), Some(v))])
    }
}

impl<IntoExpression, N> Add<IntoExpression> for Expression<N>
where
    N: Num + Clone,
    IntoExpression: Into<Expression<N>>,
{
    type Output = Self;
    fn add(mut self, rhs: IntoExpression) -> Self::Output {
        self.0.extend(rhs.into().0);
        self
    }
}

impl<IntoExpression, N> Sub<IntoExpression> for Expression<N>
where
    N: Num + Clone,
    IntoExpression: Into<Expression<N>>,
{
    type Output = Self;
    fn sub(self, rhs: IntoExpression) -> Self::Output {
        self.add(rhs.into().neg())
    }
}

impl<N> Neg for Expression<N>
where
    N: Num + Clone,
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        self * (N::zero() - N::one())
    }
}

impl<N> Mul<N> for Expression<N>
where
    N: Num + Clone,
{
    type Output = Self;
    fn mul(mut self, rhs: N) -> Self::Output {
        self.0
            .iter_mut()
            .for_each(|(w, _)| *w = rhs.clone() * (w.clone()));
        self
    }
}

impl<N> Div<N> for Expression<N>
where
    N: Num + Clone,
{
    type Output = Self;
    fn div(self, rhs: N) -> Self::Output {
        self * (N::one() / rhs)
    }
}
