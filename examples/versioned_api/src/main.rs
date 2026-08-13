use broto::{Decode, DecodeExt, Encode};
use futures::io::Cursor;
use futures::{StreamExt as _, executor::block_on};
use futures_io::{AsyncBufRead, AsyncRead};

mod api;
use api::*;

fn main() -> broto::Result<()> {
    block_on(run())
}

async fn run() -> broto::Result<()> {
    let mut writer = Cursor::new(Vec::new());

    let version = Version::V1;
    let request = v1::Request::Shape(v1::Shape::LineShape {
        start: v1::Point { x: 0.1, y: 0.2 },
        end: v1::Point { x: 0.3, y: 0.4 },
    });

    version.encode(&mut writer).await.unwrap();
    request.encode(&mut writer).await.unwrap();
    let bytes = writer.into_inner();
    println!("encoded: {:?}", bytes);

    let mut reader = Cursor::new(bytes);
    handle(&mut reader).await?;

    let mut writer = Cursor::new(Vec::new());

    let version = Version::V2;
    let request = v2::Request::Shape(v2::Shape::Triangle {
        one: v2::Point { x: 0.1, y: 0.2 },
        two: v2::Point { x: 0.3, y: 0.4 },
        three: v2::Point { x: 0.5, y: 0.6 },
    });

    version.encode(&mut writer).await.unwrap();
    request.encode(&mut writer).await.unwrap();
    let bytes = writer.into_inner();
    println!("encoded: {:?}", bytes);

    let mut reader = Cursor::new(bytes);
    handle(&mut reader).await?;

    Ok(())
}

async fn handle<R>(reader: &mut R) -> broto::Result<()>
where
    R: DecodeExt,
{
    let version = Version::decode(reader).await?;

    match version {
        Version::V1 => handle_v1(reader).await,
        Version::V2 => handle_v2(reader).await,
    }
}

async fn handle_v1<R>(reader: &mut R) -> broto::Result<()>
where
    R: DecodeExt,
{
    let mut messages = reader.messages::<v1::Request>();
    while let Some(message) = messages.next().await {
        let response = match message? {
            v1::Request::Shape(shape) => v1::Response::Shape(shape),
        };

        println!("sending response: {:?}", response);
    }
    Ok(())
}

async fn handle_v2<R>(reader: &mut R) -> broto::Result<()>
where
    R: DecodeExt,
{
    let mut messages = reader.messages::<v2::Request>();
    while let Some(message) = messages.next().await {
        let response = match message? {
            v2::Request::Shape(shape) => v2::Response::Shape(shape),
        };

        println!("sending response: {:?}", response);
    }
    Ok(())
}
