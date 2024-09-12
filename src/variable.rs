use std::ops::*;

use num::Num;

use crate::{constraint::Constraint, expression::Expression};

#[derive(Clone)]
pub struct Variable<N>
where
    N: Num + Clone,
{
    id: usize,
    name: Option<String>,
    pd: std::marker::PhantomData<N>,
}

impl<N> Variable<N>
where
    N: Num + Clone,
{
    pub fn new(id: usize, name: Option<String>) -> Self {
        Self {
            id,
            name,
            pd: Default::default(),
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> String {
        if let Some(name) = &self.name {
            name.clone()
        } else {
            format!("v{}", self.id)
        }
    }

    pub fn le(self, expr: impl Into<Expression<N>>) -> Constraint<N> {
        Expression::<N>::from(self).le(expr)
    }

    pub fn eq(self, expr: impl Into<Expression<N>>) -> Constraint<N> {
        Expression::<N>::from(self).eq(expr)
    }

    pub fn ge(self, expr: impl Into<Expression<N>>) -> Constraint<N> {
        Expression::<N>::from(self).ge(expr)
    }
}

impl<N> std::fmt::Display for Variable<N>
where
    N: Num + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            f.write_fmt(format_args!("{}", name))
        } else {
            f.write_fmt(format_args!("v{}", self.id))
        }
    }
}

impl<IntoExpression, N> Add<IntoExpression> for Variable<N>
where
    IntoExpression: Into<Expression<N>>,
    N: Num + Clone,
{
    type Output = Expression<N>;
    fn add(self, rhs: IntoExpression) -> Self::Output {
        Expression::from(self).add(rhs)
    }
}

impl<IntoExpression, N> Sub<IntoExpression> for Variable<N>
where
    IntoExpression: Into<Expression<N>>,
    N: Num + Clone,
{
    type Output = Expression<N>;
    fn sub(self, rhs: IntoExpression) -> Self::Output {
        Expression::from(self).sub(rhs)
    }
}

impl<N> Neg for Variable<N>
where
    N: Num + Clone,
{
    type Output = Expression<N>;
    fn neg(self) -> Self::Output {
        Expression::from(self)
    }
}

impl<N> Mul<N> for Variable<N>
where
    N: Num + Clone,
{
    type Output = Expression<N>;
    fn mul(self, rhs: N) -> Self::Output {
        Expression::from(self).mul(rhs)
    }
}

impl<N> Div<N> for Variable<N>
where
    N: Num + Clone,
{
    type Output = Expression<N>;
    fn div(self, rhs: N) -> Self::Output {
        Expression::from(self).div(rhs)
    }
}
