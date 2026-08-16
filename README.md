# broto-buff

A build-time code generator that turns a declarative spec file into Rust
structs and enums with [`broto`](https://github.com/nyanzebra/broto)'s
`Encode`/`Decode` already derived — write the shape of your wire protocol
once, in TOML, YAML, or any other format `serde` can parse, and get typed,
encodable/decodable Rust out the other end as part of your normal build.

```toml
# spec/spec.toml
[[types]]
kind = "Struct"
name = "Point"

[[types.fields]]
name = "x"
type = "f64"

[[types.fields]]
name = "y"
type = "f64"

[requests]
name = "Request"

[[requests.variants]]
kind = "Tuple"
name = "Ping"
types = []

[responses]
name = "Response"

[[responses.variants]]
kind = "Tuple"
name = "Pong"
types = []
```

generates:

```rust
#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
pub enum Request {
    Ping,
}

#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]
pub enum Response {
    Pong,
}
```

## Usage

Add `broto-buff` as a **build-dependency** (it runs in `build.rs`, not in
your actual program) along with whatever format crate you want to parse
your spec with:

```toml
[build-dependencies]
broto-buff = { git = "https://github.com/nyanzebra/broto-buff", branch = "main", features = ["sync"] }
toml = "1"
```

Implement [`Parse`] for whatever format you're using — `broto-buff` doesn't
hardcode a format, it just needs something that can turn a file's contents
into a [`Specification`]:

```rust
// build.rs
fn main() -> std::io::Result<()> {
    struct Parser;

    impl broto_buff::Parse for Parser {
        type Error = toml::de::Error;

        fn parse(&self, content: &impl AsRef<str>) -> Result<broto_buff::Specification, Self::Error> {
            toml::from_str(content.as_ref())
        }
    }

    broto_buff::generate(Parser, "./spec", "./src/api", "toml")?;

    Ok(())
}
```

`generate` reads every file under the input directory (`./spec`) whose name
contains the given extension filter (`"toml"`), parses each with your
`Parser`, and writes the generated Rust into the output directory
(`./src/api`) — one `.rs` file per spec file, plus a `mod.rs` wiring them
together and a `version.rs` (see [Versioning](#versioning) below). The
output directory is fully regenerated on every build (existing contents are
removed first) — **never hand-edit anything under it**; treat it the same
as any other build-script output.

Wire the generated module into your crate normally:

```rust
mod api;
use api::*;
```

See [`examples/simple`](examples/simple) for a complete single-spec setup.

## Spec format

A spec has three top-level pieces:

- **`types`** — an array of struct/enum definitions, referenceable by name
  from each other and from `requests`/`responses`.
- **`requests`** / **`responses`** — each a single enum definition (your
  API's request and response types).

Every type/enum definition carries a `kind` discriminator:

| `kind` | Where | Shape |
|---|---|---|
| `Struct` | a type in `types`, or an enum variant | named fields |
| `Enum` | a type in `types` | a set of variants |
| `Tuple` | an enum variant only | positional field types |
| `Unit` | an enum variant only | no payload |

Enum variants also accept an optional `tag`: an explicit `u8` discriminant
(`0`–`255`), generating `#[tag(N)]` on that variant — see `broto`'s own
README for what that controls on the wire. Omit it and the variant just
gets its positional index, same as `broto` derives without any tag at all.

Field/variant `type`/`types` values are plain strings — either one of
`broto`'s built-in types (`f64`, `u32`, `String`, `Vec<T>`, ... — see
`broto`'s README for the full list) or the name of another type defined
in this same spec.

### Editor autocomplete

[`schemas/spec-schema.json`](schemas/spec-schema.json) is a JSON Schema for
the format above (kept in sync with [`Specification`] via `schemars` — see
the `schema` test in `src/lib.rs`). Point your editor's TOML/YAML LSP at it
for autocomplete and inline validation while writing a spec:

```toml
#:schema ../schemas/spec-schema.json
```
```yaml
# yaml-language-server: $schema=../schemas/spec-schema.json
```

## Versioning

Point `generate` at a directory with more than one spec file and each
becomes its own module (`v1.rs`, `v2.rs`, ...), alongside a generated
`Version` enum (one variant per spec file) that's meant to be decoded
first, off the front of a message, to determine which module's types the
rest of the stream should be decoded as:

```rust
let version = Version::decode(reader).await?;
match version {
    Version::V1 => { /* decode as v1::Request, etc. */ }
    Version::V2 => { /* decode as v2::Request, etc. */ }
}
```

See [`examples/versioned_api`](examples/versioned_api) for a complete
worked example, including a `broto::DecodeExt::messages()` loop handling
both versions over the same connection.

## Features

Mirrors `broto`'s own `sync`/`async` split — enable whichever one matches
how you're using `broto` in the crate consuming the generated code (they're
mutually exclusive on `broto`'s side, same as always):

```toml
broto-buff = { git = "...", features = ["sync"] }   # or "async"
```

This only affects the `broto::AsyncWrite`/`AsyncRead` vs. `std::io::Write`/`Read`
bounds `broto`'s derive puts on the generated types — the spec format and
the rest of `generate`'s behavior are identical either way.

## License

MIT
