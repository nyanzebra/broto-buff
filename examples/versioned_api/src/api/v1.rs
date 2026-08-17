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

#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
pub enum Request {
    Shape(Shape),
}

#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
pub enum Response {
    Shape(Shape),
}
