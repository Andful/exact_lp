#![feature(format_args_nl)]
#![feature(iterator_try_collect)]

mod constraint;
mod expression;
mod variable;
pub use constraint::Constraint;
pub use expression::Expression;
pub use variable::Variable;

use num::{BigInt, BigRational, Num, Signed};
use std::{collections::BTreeMap, fmt::Display, io::BufRead, str::FromStr};

enum VariableType {
    Binary,
    Integer,
    Continuous,
}

struct InternalVariable<N>
where
    N: Num,
{
    v_type: VariableType,
    name: Option<String>,
    lb: Option<N>,
    ub: Option<N>,
}

pub struct VariableBuilder<'a, N>
where
    N: Num + Clone,
{
    model: &'a mut Model<N>,
    variable: InternalVariable<N>,
}

impl<'a, N> VariableBuilder<'a, N>
where
    N: Num + Clone,
{
    pub fn new(model: &'a mut Model<N>) -> Self {
        Self {
            model,
            variable: InternalVariable {
                v_type: VariableType::Continuous,
                name: None,
                lb: None,
                ub: None,
            },
        }
    }

    pub fn binary(mut self) -> Self {
        self.variable.v_type = VariableType::Binary;
        self
    }

    pub fn integer(mut self) -> Self {
        self.variable.v_type = VariableType::Integer;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.variable.name = Some(name.into());
        self
    }

    pub fn lb(mut self, lb: N) -> Self {
        self.variable.lb = Some(lb);
        self
    }

    pub fn ub(mut self, ub: N) -> Self {
        self.variable.ub = Some(ub);
        self
    }

    pub fn build(self) -> Variable<N> {
        let Self { model, variable } = self;
        let result = Variable::new(model.variables.len(), variable.name.clone());
        model.variables.push(variable);
        result
    }
}

pub struct Solution<N>
where
    N: Num + Clone,
{
    values: BTreeMap<String, N>,
}

impl<N> Solution<N>
where
    N: Num + Clone,
{
    pub fn get_value(&self, e: impl Into<Expression<N>>) -> N {
        e.into()
            .0
            .iter()
            .map(|(w, v)| {
                v.as_ref()
                    .map(|i| {
                        self.values
                            .get(&i.name())
                            .map(Clone::clone)
                            .unwrap_or(N::zero())
                    })
                    .unwrap_or_else(N::one)
                    .clone()
                    * w.clone()
            })
            .reduce(|a, b| a + b)
            .unwrap_or_else(N::zero)
    }
}

pub enum OptimizationDirection {
    Maximize,
    Minimize,
}

impl Default for OptimizationDirection {
    fn default() -> Self {
        OptimizationDirection::Minimize
    }
}

pub struct Model<N>
where
    N: Num + Clone,
{
    commands: Vec<String>,
    objective: Expression<N>,
    direction: OptimizationDirection,
    variables: Vec<InternalVariable<N>>,
    constraints: Vec<Constraint<N>>,
}

impl<N> Model<N>
where
    N: Num + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn maximize(&mut self) {
        self.direction = OptimizationDirection::Maximize;
    }
    pub fn minimize(&mut self) {
        self.direction = OptimizationDirection::Minimize;
    }
    pub fn add_command(&mut self, command: &str) {
        self.commands.push(command.to_string())
    }
    pub fn add_var(&mut self) -> VariableBuilder<'_, N> {
        VariableBuilder::new(self)
    }

    pub fn add_const(&mut self, c: Constraint<N>) {
        self.constraints.push(c)
    }

    pub fn set_objective(&mut self, obj: Expression<N>) {
        self.objective = obj;
    }
}

impl<N> Default for Model<N>
where
    N: Num + Clone,
{
    fn default() -> Self {
        Self {
            commands: Default::default(),
            objective: Default::default(),
            direction: Default::default(),
            variables: Default::default(),
            constraints: Default::default(),
        }
    }
}

impl<N> Model<N>
where
    N: Num + Clone + Display + Signed,
{
    fn export(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        match self.direction {
            OptimizationDirection::Maximize => w.write(b"Maximize\n")?,
            OptimizationDirection::Minimize => w.write(b"Minimize\n")?,
        };
        let obj = Expression(self.objective.0.iter().filter(|(_, b)| b.is_some()).map(Clone::clone).collect());
        w.write_fmt(format_args_nl!(" obj: {}", obj))?;
        w.write(b"Subject To\n")?;
        for (i, c) in self.constraints.iter().enumerate() {
            w.write_fmt(format_args_nl!(" c{}: {}", i, c.clone().to_normalized()))?;
        }
        w.write(b"Bounds\n")?;
        for (i, v) in self.variables.iter().enumerate() {
            match (&v.lb, &v.ub) {
                (Some(lb), Some(ub)) => {
                    w.write_fmt(format_args_nl!(
                        " {} <= {} <= {}",
                        lb,
                        v.name.clone().unwrap_or_else(|| format!("v{i}")),
                        ub
                    ))?;
                }
                (Some(lb), None) => {
                    w.write_fmt(format_args_nl!(
                        " {} <= {} <= +inf",
                        lb,
                        v.name.clone().unwrap_or_else(|| format!("v{i}"))
                    ))?;
                }
                (None, Some(ub)) => {
                    w.write_fmt(format_args_nl!(
                        " -inf <= {} <= {}",
                        v.name.clone().unwrap_or_else(|| format!("v{i}")),
                        ub
                    ))?;
                }
                (None, None) => {
                    w.write_fmt(format_args_nl!(
                        " {} free",
                        v.name.clone().unwrap_or_else(|| format!("v{i}"))
                    ))?;
                }
            }
        }
        w.write(b"General\n")?;
        for (i, v) in self.variables.iter().enumerate() {
            if let VariableType::Integer = v.v_type {
                if let Some(name) = &v.name {
                    w.write_fmt(format_args_nl!("{name}"))?;
                } else {
                    w.write_fmt(format_args_nl!(" v{i}"))?;
                }
            }
        }
        w.write(b"Binary\n")?;
        for (i, v) in self.variables.iter().enumerate() {
            if let VariableType::Binary = v.v_type {
                if let Some(name) = &v.name {
                    w.write_fmt(format_args_nl!("{name}"))?;
                } else {
                    w.write_fmt(format_args_nl!(" v{i}"))?;
                }
            }
        }
        w.write(b"End\n")?;

        Ok(())
    }
}

impl Model<BigRational> {
    fn import(&self, v: &mut impl std::io::Read) -> std::io::Result<Solution<BigRational>> {
        let re = std::cell::LazyCell::new(|| {
            use regex::Regex;
            Regex::new(r"^(?<id>\w+)\s+(?<fraction>\d+(?:\/\d+)?)").unwrap()
        });

        let mut result = Solution {
            values: Default::default(),
        };

        let lines = std::io::BufReader::new(v).lines().try_collect::<Vec<_>>()?;

        let re = &*re;
        for capture in lines.iter().map(|l| re.captures(l)) {
            let Some(caps) = capture else {
                continue;
            };
            let id = caps["id"].to_string();
            let fraction = BigRational::from_str(&caps["fraction"]).unwrap();
            result.values.insert(id, fraction);
        }

        Ok(result)
    }

    pub fn solve(&self, leave_debug_info: bool) -> std::io::Result<Solution<BigRational>> {
        use std::fs;
        use std::process::{Command, Stdio};
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();

        let formulation_path = dir.path().join("formulation.lp");
        let solution_path = dir.path().join("solution.sol");
        if leave_debug_info {
            std::mem::forget(dir);
        }
        let mut f = fs::File::create(&formulation_path).unwrap();
        self.export(&mut f).unwrap();
        drop(f);

        let mut command = Command::new("scip");
        command.arg("-c").arg("set exact enabled TRUE");

        for c in self.commands.iter() {
            command.arg("-c").arg(c);
        }
        command
            .arg("-c")
            .arg(format!("read {}", formulation_path.to_string_lossy()))
            .arg("-c")
            .arg(&format!("optimize"))
            .arg("-c")
            .arg(&format!(
                "write solution {}",
                solution_path.to_string_lossy()
            ))
            .stdout(Stdio::inherit())
            .output()
            .unwrap();

        let mut f = fs::File::open(&solution_path).unwrap();

        let solution = self.import(&mut f).unwrap();
        Ok(solution)
    }
}

impl Model<f64> {
    fn import(&self, v: &mut impl std::io::Read) -> std::io::Result<Solution<f64>> {
        let re = std::cell::LazyCell::new(|| {
            use regex::Regex;
            Regex::new(r"^(?<id>\w+)\s+(?<number>.+)\(obj:").unwrap()
        });

        let mut result = Solution {
            values: Default::default(),
        };

        let lines = std::io::BufReader::new(v).lines().try_collect::<Vec<_>>()?;

        let re = &*re;
        for capture in lines.iter().map(|l| re.captures(l)) {
            let Some(caps) = capture else {
                continue;
            };
            let id = caps["id"].to_string();
            let fraction = f64::from_str(&caps["number"].trim()).unwrap();
            result.values.insert(id, fraction);
        }

        Ok(result)
    }

    pub fn solve(&self, leave_debug_info: bool) -> std::io::Result<Solution<f64>> {
        use std::fs;
        use std::process::{Command, Stdio};
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();

        let formulation_path = dir.path().join("formulation.lp");
        let solution_path = dir.path().join("solution.sol");
        if leave_debug_info {
            std::mem::forget(dir);
        }
        let mut f = fs::File::create(&formulation_path).unwrap();
        self.export(&mut f).unwrap();
        drop(f);

        let mut command = Command::new("scip");

        for c in self.commands.iter() {
            command.arg("-c").arg(c);
        }
        let out = command
            .arg("-c")
            .arg(format!("read {}", formulation_path.to_string_lossy()))
            .arg("-c")
            .arg(&format!("optimize"))
            .arg("-c")
            .arg(&format!(
                "write solution {}",
                solution_path.to_string_lossy()
            ))
            .arg("-c")
            .arg("quit")
            .stdout(Stdio::inherit())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        
        let mut attempt = 0;
        while !fs::exists(&solution_path).unwrap() && attempt < 10 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            attempt += 1;
        }
        if !fs::exists(&solution_path).unwrap() {
            panic!("No file {:?}. Exit status {:?}. Command: {:?}", solution_path, out, command);
        }
        let mut f = fs::File::open(&solution_path).unwrap();

        let solution = self.import(&mut f).unwrap();
        Ok(solution)
    }
}

pub struct Constant<N>(pub N)
where
    N: Num + Clone;

impl<IntoExpression, N> std::ops::Mul<IntoExpression> for Constant<N>
where
    N: Num + Clone,
    IntoExpression: Into<Expression<N>>,
{
    type Output = Expression<N>;
    fn mul(self, rhs: IntoExpression) -> Self::Output {
        rhs.into() * self.0
    }
}

pub fn c<N>(e: N) -> Constant<BigRational>
where
    N: Into<BigInt>,
{
    Constant(e.into().into())
}

#[cfg(test)]
mod tests {
    use num::{BigInt, BigRational};

    use crate::{c, Expression, Model};

    #[test]
    fn test_expression() {
        let mut model = Model::<BigRational>::default();
        let x = c(2) * model.add_var().name("a").build();
        let y = c(3) * model.add_var().name("b").build();
        let z = c(2) * y.clone();
        model.add_var();
        let k = c(6) * model.add_var().name("e").build();
        let expr = (x + y) + (z + k);
        println!("{expr}")
    }

    #[test]
    fn test_expression2() {
        /*let mut model = Model::<BigRational>::default();
        let x = model.add_var().build() * 2.into();
        let b = model.add_var().build() * 3.into();
        let c = model.add_var().build() * 4.into();
        let e = model.add_var().build() * 6.into();
        let expr = (a + b) + (c + e);
        println!("{expr}")*/
    }

    #[test]
    fn test_expression3() {
        /*let mut model = Model::<Rational32>::default();
        let a = model.add_var().build() * 2.into();
        let b = model.add_var().build() * 3.into();
        let c = model.add_var().build() * (4.into()) / 3.into();
        let e = model.add_var().build() * 6.into();
        let expr: Expression<Rational32> = (a + b) - (c + e) + Rational32::from(9);
        println!("{expr}")*/
    }

    #[test]
    fn test_constraint() {
        /*let mut model = Model::<Rational32>::default();
        let a = model.add_var().build() * 2.into();
        let b = model.add_var().build() * 3.into();
        let c = model.add_var().build() * (4.into()) / 3.into();
        let e = model.add_var().build() * 6.into();
        let expr: Expression<Rational32> = (a + b) - (c + e) + Rational32::from(9);
        println!("{}", expr.le(Rational32::from(-10)))*/
    }

    #[test]
    fn test_export_and_import() {
        let mut model = Model::<BigRational>::new();

        let x = model
            .add_var()
            .name("x")
            .lb(BigRational::new(0.into(), 1.into()))
            .build();
        let y = model
            .add_var()
            .name("y")
            .lb(BigRational::new(0.into(), 1.into()))
            .build();

        model.maximize();
        model.set_objective(
            x.clone() * BigRational::new(2.into(), 1.into())
                + y.clone() * BigRational::new(5.into(), 1.into()),
        );

        model.add_const((x.clone() + c(4) * y.clone()).le(BigRational::new(24.into(), 1.into())));
        model.add_const((c(3) * x.clone() + y.clone()).le(BigRational::new(21.into(), 1.into())));
        model.add_const((x.clone() + y.clone()).le(BigRational::new(9.into(), 1.into())));

        let solution = model.solve(false).unwrap();

        assert_eq!(
            solution.get_value(x.clone()),
            BigRational::new(4.into(), 1.into())
        );
        assert_eq!(
            solution.get_value(y.clone()),
            BigRational::new(5.into(), 1.into())
        );
    }
}
