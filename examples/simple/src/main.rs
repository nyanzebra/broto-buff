use std::collections::VecDeque;

use broto::{Decode, DecodeExt as _, Encode};

mod api;
use api::{spec::*, *};

fn main() {
    let mut buffer = VecDeque::with_capacity(1024);

    let version = Version::Spec;
    let request = Request::Shape(Shape::LineShape {
        start: Point { x: 0.1, y: 0.2 },
        end: Point { x: 0.3, y: 0.4 },
    });
    let response = Response::Shape(Shape::Circle { radius: 4.2 });

    version.encode(&mut buffer).unwrap();
    request.encode(&mut buffer).unwrap();
    response.encode(&mut buffer).unwrap();
    println!("encoded: {:?}", buffer);

    assert_eq!(version, Version::decode(&mut buffer).unwrap());
    assert_eq!(request, Request::decode(&mut buffer).unwrap());
    assert_eq!(response, Response::decode(&mut buffer).unwrap());
    println!("decoded: {:?}", buffer);
}
