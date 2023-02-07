macro_rules! default_trait {
    ($trait:ident, $type:ident, $method:ident) => {
        impl $trait for $type {
            type Output = $type;
            fn $method(self) -> Self::Output {
                $type::$method(self)
            }
        }
    };
}

pub trait Sqrt {
    type Output;
    fn sqrt(self) -> Self::Output;
}
default_trait!(Sqrt, f32, sqrt);
default_trait!(Sqrt, f64, sqrt);

pub trait Sin {
    type Output;
    fn sin(self) -> Self::Output;
}
default_trait!(Sin, f32, sin);
default_trait!(Sin, f64, sin);

pub trait Cos {
    type Output;
    fn cos(self) -> Self::Output;
}
default_trait!(Cos, f32, cos);
default_trait!(Cos, f64, cos);
