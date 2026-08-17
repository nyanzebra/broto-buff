/// This an auto-generated file. Do not edit directly.
#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
pub struct Line {
    pub start: Point,
    pub end: Point,
}

#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
pub enum Shape {
    Circle(f64),
    Square { side_length: f64 },
    Rectangle { width: f64, height: f64 },
    LineShape { start: Point, end: Point },
}

pub mod foo {
    #[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
    pub struct Bar {
        pub value: f64,
    }

    #[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
    pub struct Baz {
        pub Bar: Bar,
    }

    #[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
    pub struct FooTest {
        pub Bar_from_poo: super::poo::Bar,
        pub Baz_from_poo: super::poo::Baz,
    }
}

pub mod poo {
    #[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
    pub struct Bar {
        pub value: i32,
    }

    #[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
    pub struct Baz {
        pub Bar: Bar,
    }

    #[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
    pub struct PooTest {
        pub Bar_from_foo: super::foo::Bar,
        pub Baz_from_foo: super::foo::Baz,
    }
}

#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
pub enum Request {
    Shape(Shape),
}

#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
pub enum Response {
    Shape(Shape),
}
