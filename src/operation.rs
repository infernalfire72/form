use std::fmt::Display;

pub struct Operational(pub &'static str);

// TODO: implement >> as gt, << as lt
// TODO: implement !
pub enum Operation {
    Simple(String, Vec<String>),
    EQ(&'static str, String),
    GT(&'static str, String),
    LT(&'static str, String),
}

macro_rules! format_op {
  ($k:expr, $op:tt, $v:expr) => {
    (format!(concat!(stringify!({} $op), " ?"), $k), $v)
  };
}

impl Operation {
    pub fn format(self) -> (String, Vec<String>) {
        match self {
            Operation::Simple(fmt, value) => (fmt, value),
            Operation::EQ(key, value) => format_op!(key, =, vec![value]),
            Operation::GT(key, value) => format_op!(key, >, vec![value]),
            Operation::LT(key, value) => format_op!(key, <, vec![value]),
        }
    }
}

// Operations
macro_rules! gen_op {
    ($id:ident, $op:ident) => {
        pub fn $id<T>(&self, other: T) -> Operation
        where
            T: Display,
        {
            Operation::$op(self.0, other.to_string())
        }
    };
}

impl Operational {
    gen_op!(equals, EQ);
    gen_op!(greater_than, GT);
}

// TODO: reimplement these to actually do arithmetic
use std::ops::{BitAnd, BitOr, Shl, Shr};
impl BitOr for Operation {
    type Output = Operation;

    fn bitor(self, rhs: Operation) -> Self::Output {
        let (c1, mut params) = self.format();
        let (c2, mut p2) = rhs.format();
        params.append(&mut p2);
        Operation::Simple(format!("({} OR {})", c1, c2), params)
    }
}

impl BitAnd for Operation {
    type Output = Operation;

    fn bitand(self, rhs: Operation) -> Self::Output {
        let (c1, mut params) = self.format();
        let (c2, mut p2) = rhs.format();
        params.append(&mut p2);
        Operation::Simple(format!("({} AND {})", c1, c2), params)
    }
}

impl<T> Shr<T> for Operational
where
    T: Display,
{
    type Output = Operation;

    fn shr(self, rhs: T) -> Self::Output {
        Operation::GT(self.0, rhs.to_string())
    }
}

impl<T> Shl<T> for Operational
where
    T: Display,
{
    type Output = Operation;

    fn shl(self, rhs: T) -> Self::Output {
        Operation::LT(self.0, rhs.to_string())
    }
}
