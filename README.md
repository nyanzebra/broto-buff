# broto-buff

`broto-buff` generates Rust types and wire-protocol bindings from a declarative specification.

It is built on top of [`broto`](https://github.com/nyanzebra/broto) and generates types with `broto`'s `Encode` and `Decode` derives, allowing protocol definitions to live separately from the generated Rust code.

The specification format is represented by `broto_buff::Specification`. The repository also provides a JSON Schema for editor validation and autocomplete.

## Features

* Generate Rust structs, tuple types, and enums from a specification.
* Generate request and response enums.
* Support nested modules and module-qualified type references.
* Support explicit enum tags.
* Generate multiple protocol versions from multiple specification files.
* Support TOML, YAML, or any other input format through the `Parse` trait.
* Generate `broto::Encode` and `broto::Decode` implementations.
* Format generated Rust with `rustfmt`.
* Provide a JSON Schema for editor validation and autocomplete.

## How it works

`broto-buff` separates parsing, code generation, and wire encoding:

```text
 specification file
        │
        │  Parse
        ▼
  Specification
        │
        │  generate
        ▼
 generated Rust
        │
        │
        ▼
 broto::Encode / Decode
```

The input format is deliberately not part of the generator. An application provides a parser that converts its chosen format into a `Specification`.

This means the same generator can be used with TOML, YAML, JSON, or another format without making the generator depend on that format.

## Quick start

Add `broto-buff` as a build dependency:

```toml
[build-dependencies]
broto-buff = { git = "https://github.com/nyanzebra/broto-buff", branch = "main" }
toml = "1"
```

Create a specification file, for example:

```text
spec/
└── api.toml
```

Then use `broto-buff` from `build.rs`:

```rust
fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=spec");

    struct Parser;

    impl broto_buff::Parse for Parser {
        type Error = toml::de::Error;

        fn parse(
            &self,
            content: &impl AsRef<str>,
        ) -> Result<broto_buff::Specification, Self::Error> {
            toml::from_str(content.as_ref())
        }
    }

    broto_buff::generate(Parser, "spec", "src/api", "toml")?;

    Ok(())
}
```

The generated Rust can then be included in your crate:

```rust
mod api;
use api::*;
```

Running:

```bash
cargo build
```

will parse the specification, generate the Rust bindings, and format the generated files.

## Specification

The specification is represented by `broto_buff::Specification` and describes:

* global type definitions
* nested modules
* module-qualified type references
* request types
* response types
* structs
* tuple types
* enums
* unit, tuple, and struct enum variants
* optional explicit enum tags

The authoritative schema is [`schemas/spec-schema.json`](schemas/spec-schema.json).

The schema is generated from the Rust specification types and can be used by editors and language servers to provide validation and autocomplete.

### Editor support

TOML specifications can reference the schema with:

```toml
#:schema ../../../schemas/spec-schema.json
```

YAML specifications can reference the same schema with:

```yaml
# yaml-language-server: $schema=../../../schemas/spec-schema.json
```

The schema should be treated as the source of truth for the exact specification structure. The README intentionally does not duplicate the complete schema.

## Input formats

`broto-buff` does not require a specific serialization format.

Instead, implement the `Parse` trait for the format you want to use:

```rust
pub trait Parse {
    type Error;

    fn parse(
        &self,
        content: &impl AsRef<str>,
    ) -> Result<Specification, Self::Error>;
}
```

For example, a TOML parser can be implemented with `serde`:

```rust
struct Parser;

impl broto_buff::Parse for Parser {
    type Error = toml::de::Error;

    fn parse(
        &self,
        content: &impl AsRef<str>,
    ) -> Result<broto_buff::Specification, Self::Error> {
        toml::from_str(content.as_ref())
    }
}
```

The generator only operates on the resulting `Specification`, so the parser and generator remain independent.

## Modules

Specifications can organize types into modules.

Modules can contain:

* type definitions
* nested modules
* references to types in other modules

Type references can be qualified with a module path. This allows different modules to define types with the same name without creating a collision.

For example, a specification can contain both:

```text
foo::bar
poo::bar
```

and another type can explicitly refer to either definition.

Modules may also contain nested modules, allowing larger protocols to be organized into a hierarchy.

See the module examples in the repository for a complete specification.

## Requests and responses

Requests and responses are represented as enum definitions.

This makes protocol operations explicit in the generated Rust code and allows each operation to carry its associated request or response data.

For example, a request enum might conceptually look like:

```rust
pub enum Request {
    GetPoint(Point),
    GetShape(Shape),
}
```

and the corresponding response enum:

```rust
pub enum Response {
    Point(Point),
    Shape(Shape),
}
```

The actual wire representation is handled by `broto`.

## Enum tags

Enum variants can optionally specify an explicit `u8` tag.

Tags are useful when the wire representation needs stable, explicitly assigned discriminants rather than relying on the variant's position.

Tags are constrained to the range `0..=255`.

When a tag is not specified, the generator leaves the variant untagged and `broto` determines the appropriate encoding behavior.

See the [`broto`](https://github.com/nyanzebra/broto) documentation for details about the resulting wire representation.

## Versioned specifications

`broto-buff` can generate bindings from multiple specification files.

For example:

```text
spec/
├── v1.toml
└── v2.toml
```

Each specification becomes a generated Rust module, allowing protocol versions to coexist in the same crate.

This is useful when a protocol needs to maintain compatibility with older clients or servers.

The generated API exposes the available versions so the application can determine which set of generated types should be used.

See [`examples/versioned_api`](examples/versioned_api) for a complete example.

## Generated code

Generated Rust should be treated as build output.

Do not edit generated files manually. They are regenerated from the specification and any manual changes will be overwritten.

The generation process is:

```text
parse specification
       │
       ▼
generate Rust source
       │
       ▼
write generated files
       │
       ▼
run rustfmt
       │
       ▼
formatted generated Rust
```

`broto-buff` uses `rustfmt` to format generated Rust rather than implementing its own formatting rules.

This keeps generated code consistent with normal Rust code and means consumers do not need to maintain a separate formatting step just for generated files.

## Examples

The repository contains examples demonstrating common usage patterns.

### Simple API

[`examples/simple`](examples/simple) demonstrates generating bindings from a single specification.

### Versioned API

[`examples/versioned_api`](examples/versioned_api) demonstrates generating bindings from multiple specifications and working with protocol versions.

Additional test specifications can be found throughout the repository and are useful references when adding new features to the specification format.

## Project structure

The repository is organized roughly as follows:

```text
broto-buff/
├── src/
│   └── ...
├── schemas/
│   └── spec-schema.json
├── examples/
│   ├── simple/
│   └── versioned_api/
├── tests/
└── Cargo.toml
```

The important pieces are:

* `src/` — specification types, parsing abstractions, and code generation.
* `schemas/` — JSON Schema for the specification.
* `examples/` — complete usage examples.
* `tests/` — generator and specification tests.

## Design goals

### Keep the protocol definition separate from Rust

The specification describes the protocol independently of the generated implementation.

This makes it easier to:

* review protocol changes
* generate bindings for multiple versions
* use different source formats
* validate specifications with editor tooling

### Keep parsing separate from generation

`broto-buff` consumes a `Specification`, not TOML or YAML or any other format directly.

This keeps the generator small and makes it possible for applications to choose their own configuration or serialization format.

### Let `broto` own wire encoding

`broto-buff` is responsible for turning a protocol specification into Rust types.

`broto` is responsible for encoding and decoding those types.

The separation keeps protocol modeling and wire-level I/O independent.

## License

MIT
