use std::{
    fmt::Display,
    fs::{create_dir_all, read_to_string, remove_dir_all, write},
    path::Path,
    process::Command,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

const HEADER: &str = "/// This an auto-generated file. Do not edit directly.";
const DERIVE: &str = "#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]";

/// Trait for parsing a specification from a string.
/// Users should implement this trait for their own parsers as this
/// crate provides no default implementations.
pub trait Parse {
    type Error: std::error::Error;

    /// Parses the given content into a [`Specification`].
    fn parse(&self, content: &impl AsRef<str>) -> Result<Specification, Self::Error>;
}

/// Represents a type name, optionally with a module prefix.
/// The module prefix should be a fully qualified module path.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TypeName {
    /// The name of the type.
    pub name: String,

    /// The module prefix, if any.
    pub module: Option<String>,
}

impl Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(module) = &self.module {
            write!(f, "{}::{}", module, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

/// Represents a field in a struct or enum.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Field {
    /// The name of the field.
    pub name: String,
    /// The type of the field.
    pub r#type: TypeName,
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pub {}: {}", self.name, self.r#type)
    }
}

/// Represents a struct definition.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct StructDefinition {
    /// The name of the struct.
    pub name: TypeName,
    /// The fields of the struct.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<Field>,
}

impl Display for StructDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", DERIVE)?;
        writeln!(f, "pub struct {} {{", self.name)?;
        for field in &self.fields {
            writeln!(f, "\t{},", field)?;
        }
        writeln!(f, "}}")
    }
}

/// Represents a struct tuple definition.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TupleDefinition {
    /// The name of the tuple.
    pub name: TypeName,
    /// The types in the tuple.
    pub types: Vec<TypeName>,
}

impl Display for TupleDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", DERIVE)?;
        writeln!(f, "pub struct {}(", self.name)?;
        for (i, t) in self.types.iter().enumerate() {
            if i < self.types.len() - 1 {
                writeln!(f, "{},", t)?;
            } else {
                writeln!(f, "{}", t)?;
            }
        }
        writeln!(f, ");")
    }
}

/// Represents a single variant in an enum (struct, tuple, or unit).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(tag = "kind")]
#[serde(tag = "kind")]
pub enum EnumVariant {
    /// Represents a struct variant.
    Struct {
        /// The name of the struct.
        name: TypeName,
        /// The tag value, if any.
        tag: Option<u8>,
        /// The fields of the struct.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        fields: Vec<Field>,
    },
    /// Represents a tuple variant.
    Tuple {
        /// The name of the tuple.
        name: TypeName,
        /// The tag value, if any.
        tag: Option<u8>,
        /// The types in the tuple.
        types: Vec<TypeName>,
    },
    /// Represents a unit variant.
    Unit {
        /// The name of the unit variant.
        name: TypeName,
        /// The tag value, if any.
        tag: Option<u8>,
    },
}

impl Display for EnumVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnumVariant::Struct { name, tag, fields } => {
                if let Some(tag) = tag {
                    writeln!(f, "#[tag({tag})]")?;
                }

                write!(f, "\t{} {{\n", name)?;
                for field in fields {
                    write!(f, "\t\t{}: {},\n", field.name, field.r#type)?;
                }
                writeln!(f, "\t}},")
            }
            EnumVariant::Tuple { name, tag, types } => {
                if let Some(tag) = tag {
                    writeln!(f, "#[tag({tag})]")?;
                }
                write!(f, "\t{}(", name)?;
                for (i, r#type) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", r#type)?;
                }
                writeln!(f, "),")
            }
            EnumVariant::Unit { name, tag } => {
                if let Some(tag) = tag {
                    writeln!(f, "#[tag({tag})]")?;
                }
                writeln!(f, "\t{},", name)
            }
        }
    }
}

/// Represents an enum definition.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnumDefinition {
    /// The name of the enum.
    pub name: TypeName,
    /// The variants in the enum.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<EnumVariant>,
}

impl Display for EnumDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", DERIVE)?;
        writeln!(f, "pub enum {} {{", self.name)?;
        for variant in &self.variants {
            write!(f, "{}", variant)?;
        }
        writeln!(f, "}}")
    }
}

/// Represents a type definition (enum, struct, or tuple).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(tag = "kind")]
#[serde(tag = "kind")]
pub enum TypeDefinition {
    /// Represents an enum definition.
    Enum(EnumDefinition),
    /// Represents a struct definition.
    Struct(StructDefinition),
    /// Represents a tuple definition.
    Tuple(TupleDefinition),
}

impl Display for TypeDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeDefinition::Enum(e) => writeln!(f, "{}", e)?,
            TypeDefinition::Struct(s) => writeln!(f, "{}", s)?,
            TypeDefinition::Tuple(t) => writeln!(f, "{}", t)?,
        }
        Ok(())
    }
}

/// Represents a module containing type definitions.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Module {
    /// The name of the module.
    pub name: String,
    /// The type definitions in the module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<TypeDefinition>,
    /// The submodules in the module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mods: Vec<Module>,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "pub mod {} {{", self.name)?;
        for t in &self.types {
            writeln!(f, "\t{}", t)?;
        }
        for m in &self.mods {
            writeln!(f, "\t{}", m)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

/// Represents a collection of type definitions and modules.
/// This is the global module that contains all type definitions and submodules.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Types {
    /// The type definitions in the global module.
    pub definitions: Vec<TypeDefinition>,
    /// The submodules in the global module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<Module>,
}

impl Display for Types {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in &self.definitions {
            writeln!(f, "{}", t)?;
        }

        for m in &self.modules {
            writeln!(f, "{}", m)?;
        }

        Ok(())
    }
}

/// Represents the entire specification, including type definitions and requests/responses.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Specification {
    /// The collection of type definitions and modules.
    pub types: Types,
    /// The enum definition for requests.
    pub requests: EnumDefinition,
    /// The enum definition for responses.
    pub responses: EnumDefinition,
}

impl Display for Specification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", HEADER)?;
        writeln!(f, "{}", self.types)?;
        writeln!(f, "{}", self.requests)?;
        writeln!(f, "{}", self.responses)?;
        Ok(())
    }
}

pub fn generate<P>(
    parse: P,
    input_dir: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
    ext_filter: &str,
) -> std::io::Result<()>
where
    P: Parse,
    P::Error: std::error::Error,
{
    let input_dir = input_dir.as_ref();
    let output_dir = output_dir.as_ref();

    if !input_dir.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "folder not found",
        ));
    }

    println!("cargo:info=generating broto-buff bindings for files in {input_dir:?}");

    let mut versions = vec![];
    let mut specifications = vec![];

    for entry in WalkDir::new(&input_dir).max_depth(1) {
        match entry {
            Ok(entry) => {
                println!("cargo:info=checking file {entry:?}");

                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str())
                        && filename.contains(ext_filter)
                    {
                        println!("cargo:info=found file {filename}");
                        versions.push(filename.to_string());
                        if let Ok(content) = read_to_string(&path) {
                            match parse.parse(&content) {
                                Ok(spec) => specifications.push(spec),
                                Err(err) => {
                                    println!("cargo:warning=failed to parse file {filename}: {err}")
                                }
                            }
                        } else {
                            println!("cargo:warning=failed to read file {filename}");
                        }
                    } else {
                        println!("cargo:warning=path {path:?} is not valid");
                    }
                }
            }
            Err(err) => {
                println!("cargo:warning=failed to handle entry due to {err}");
            }
        }
    }

    _ = remove_dir_all(&output_dir)
        .inspect_err(|e| println!("cargo:warning=failed to remove dir {output_dir:?}: {e}"));
    create_dir_all(&output_dir)
        .inspect_err(|e| println!("cargo:warning=failed to create dir {output_dir:?}: {e}"))?;

    write(output_dir.join("version.rs"), make_version_rs(&versions))
        .inspect_err(|e| println!("cargo:warning=failed to write mod.rs: {e}"))?;
    rustfmt(&output_dir.join("version.rs"))?;

    for (version, spec) in versions.iter().zip(specifications.into_iter()) {
        let mut path = output_dir.join(version);
        path.set_extension("rs");

        write(&path, &format!("{}", spec))
            .inspect_err(|e| println!("cargo:warning=failed to write {path:?}: {e}"))?;
        rustfmt(&path)?;
    }

    write(output_dir.join("mod.rs"), make_mod_rs(&versions))
        .inspect_err(|e| println!("cargo:warning=failed to write mod.rs: {e}"))?;
    rustfmt(&output_dir.join("mod.rs"))?;

    Ok(())
}

fn make_mod_rs(versions: &[String]) -> String {
    let mut content = String::default();

    content.push_str(HEADER);
    content.push_str("\n");
    content.push_str(&format!("mod version;"));
    content.push_str("\n");
    content.push_str(&format!("pub use version::*;"));
    content.push_str("\n");

    for variant in versions.iter().map(|s| s.split('.').next().unwrap_or(s)) {
        content.push_str(&format!("pub mod {variant};"));
        content.push_str("\n");
    }

    content
}

fn make_version_rs(versions: &[String]) -> String {
    let mut content = String::default();

    content.push_str(HEADER);
    content.push_str("\n");
    content.push_str(DERIVE);
    content.push_str("\n");
    content.push_str("pub enum Version {\n");
    for variant in versions.iter().map(|s| s.split('.').next().unwrap_or(s)) {
        content.push_str("\t");
        content.push_str(&variant[0..1].to_uppercase());
        content.push_str(&variant[1..]);
        content.push_str(",\n");
    }
    content.push_str("}\n");
    content
}

fn rustfmt(path: &Path) -> std::io::Result<()> {
    let status = Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .arg(path)
        .status()?;

    if !status.success() {
        return Err(std::io::Error::other(format!(
            "rustfmt failed for {}",
            path.display()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use schemars::schema_for;

    use super::*;

    #[ignore = "for making examples"]
    #[test]
    fn yaml_example() {
        println!("ex: {:#?}", yaml_serde::to_string(&test_spec()).unwrap());
    }

    #[ignore = "for making examples"]
    #[test]
    fn toml_example() {
        println!("ex: {:#?}", toml::to_string(&test_spec()).unwrap());
    }

    #[test]
    fn schema() {
        let schema = schema_for!(Specification);
        let json = serde_json::to_string_pretty(&schema).unwrap();
        write("schemas/spec-schema.json", json).unwrap();
    }

    fn test_spec() -> Specification {
        Specification {
            types: Types {
                definitions: vec![
                    TypeDefinition::Struct(StructDefinition {
                        name: TypeName {
                            name: "Point".to_string(),
                            module: None,
                        },
                        fields: vec![Field {
                            name: "y".to_string(),
                            r#type: TypeName {
                                name: "f64".to_string(),
                                module: None,
                            },
                        }],
                    }),
                    TypeDefinition::Enum(EnumDefinition {
                        name: TypeName {
                            name: "Color".to_string(),
                            module: None,
                        },
                        variants: vec![EnumVariant::Unit {
                            name: TypeName {
                                name: "Red".to_string(),
                                module: None,
                            },
                            tag: None,
                        }],
                    }),
                    TypeDefinition::Tuple(TupleDefinition {
                        name: TypeName {
                            name: "Degrees".to_string(),
                            module: Some("stuff".to_string()),
                        },
                        types: vec![],
                    }),
                ],
                modules: vec![Module {
                    name: "stuff".to_string(),
                    types: vec![TypeDefinition::Tuple(TupleDefinition {
                        name: TypeName {
                            name: "Degrees".to_string(),
                            module: None,
                        },
                        types: vec![TypeName {
                            name: "f64".to_string(),
                            module: None,
                        }],
                    })],
                    mods: vec![],
                }],
            },
            requests: EnumDefinition {
                name: TypeName {
                    name: "Request".to_string(),
                    module: None,
                },
                variants: vec![],
            },
            responses: EnumDefinition {
                name: TypeName {
                    name: "Response".to_string(),
                    module: None,
                },
                variants: vec![],
            },
        }
    }
}
