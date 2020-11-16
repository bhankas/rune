#[macro_export]
macro_rules! vec_into {
    ($($x:expr),+ $(,)?) => (vec![$($x.into()),+]);
}

#[macro_export]
macro_rules! cons {
    ($car:expr, $cdr:expr) => (Cons::new($car.into(), $cdr.into()));
    ($car:expr) => (Cons::new($car.into(), false.into()));
}

#[macro_export]
macro_rules! list {
    ($x:expr) => (cons!($x));
    ($x:expr, $($y:expr),+ $(,)?) => (cons!($x, list!($($y),+)));
}
