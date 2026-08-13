use std::{
    fmt::Display,
    fs::{create_dir_all, read_to_string, remove_dir_all, write},
    path::Path,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

const HEADER: &str = "/// This an auto-generated file. Do not edit directly.";
const DERIVE: &str = "#[derive(Clone, Debug, PartialEq, broto::Encode, broto::Decode)]";

pub trait Parse {
    type Error: std::error::Error;

    fn parse(&self, content: &impl AsRef<str>) -> Result<Specification, Self::Error>;
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct StructDefinition {
    pub name: TypeName,
    pub fields: Vec<Field>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(tag = "kind")]
#[serde(tag = "kind")]
pub enum EnumVariant {
    Struct {
        name: TypeName,
        tag: Option<u8>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        fields: Vec<Field>,
    },
    Tuple {
        name: TypeName,
        tag: Option<u8>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        types: Vec<TypeName>,
    },
    Unit {
        name: TypeName,
        tag: Option<u8>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct EnumDefinition {
    pub name: TypeName,
    pub variants: Vec<EnumVariant>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[schemars(tag = "kind")]
#[serde(tag = "kind")]
pub enum TypeDefinition {
    Struct(StructDefinition),
    Enum(EnumDefinition),
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct TypeName(String);

impl Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Field {
    pub name: String,
    pub r#type: TypeName,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Specification {
    pub types: Vec<TypeDefinition>,
    pub requests: EnumDefinition,
    pub responses: EnumDefinition,
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

    let mut versions = vec![];
    let mut specifications = vec![];

    for entry in WalkDir::new(&input_dir).max_depth(1) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str())
                        && filename.contains(ext_filter)
                    {
                        versions.push(filename.to_string());
                        if let Ok(content) = read_to_string(&path) {
                            match parse.parse(&content) {
                                Ok(spec) => specifications.push(spec),
                                Err(err) => log::warn!("failed to parse file {filename}: {err}"),
                            }
                        } else {
                            log::warn!("failed to read file {filename}");
                        }
                    } else {
                        log::warn!("filename is not valid");
                    }
                }
            }
            Err(err) => {
                log::warn!("failed to handle entry due to {err}");
            }
        }
    }

    _ = remove_dir_all(&output_dir)
        .inspect_err(|e| log::warn!("failed to remove dir {output_dir:?}: {e}"));
    create_dir_all(&output_dir)
        .inspect_err(|e| log::warn!("failed to create dir {output_dir:?}: {e}"))?;

    write(output_dir.join("version.rs"), make_version_rs(&versions))
        .inspect_err(|e| log::warn!("failed to write mod.rs: {e}"))?;

    for (version, spec) in versions.iter().zip(specifications.into_iter()) {
        let mut path = output_dir.join(version);
        path.set_extension("rs");

        write(&path, spec_to_rs(&spec))
            .inspect_err(|e| log::warn!("failed to write {path:?}: {e}"))?;
    }

    write(output_dir.join("mod.rs"), make_mod_rs(&versions))
        .inspect_err(|e| log::warn!("failed to write mod.rs: {e}"))?;

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

fn spec_to_rs(
    Specification {
        types,
        requests,
        responses,
    }: &Specification,
) -> String {
    let mut content = String::default();

    content.push_str(HEADER);
    content.push_str("\n");

    content.push_str(&make_enum(&requests));
    content.push_str("\n");
    content.push_str(&make_enum(&responses));
    content.push_str("\n");

    for type_def in types {
        content.push_str(&make_type(&type_def));
        content.push_str("\n");
    }

    content
}

fn make_type(def: &TypeDefinition) -> String {
    let mut content = String::default();

    match def {
        TypeDefinition::Struct(struct_def) => content.push_str(&make_struct(struct_def)),
        TypeDefinition::Enum(en) => content.push_str(&make_enum(en)),
    }
    content
}

fn make_enum(EnumDefinition { name, variants, .. }: &EnumDefinition) -> String {
    let mut content = String::default();
    content.push_str(DERIVE);
    content.push_str("\n");
    content.push_str(&format!("pub enum {} {{\n", name));
    for variant in variants {
        match variant {
            EnumVariant::Struct { name, tag, fields } => {
                if let Some(tag) = tag {
                    content.push_str(&format!("#[tag({tag})]\n"));
                }

                content.push_str(&format!("\t{} {{\n", name));
                for field in fields {
                    content.push_str(&format!("\t\t{}: {},\n", field.name, field.r#type));
                }
                content.push_str("\t},\n");
            }
            EnumVariant::Tuple { name, tag, types } => {
                if let Some(tag) = tag {
                    content.push_str(&format!("#[tag({tag})]\n"));
                }
                content.push_str(&format!("\t{}(", name));
                for (i, r#type) in types.iter().enumerate() {
                    if i > 0 {
                        content.push_str(", ");
                    }
                    content.push_str(&format!("{}", r#type));
                }
                content.push_str(")\n");
            }
            EnumVariant::Unit { name, tag } => {
                if let Some(tag) = tag {
                    content.push_str(&format!("#[tag({tag})]\n"));
                }
                content.push_str(&format!("\t{},\n", name));
            }
        }
    }
    content.push_str("}\n");
    content
}

fn make_struct(StructDefinition { name, fields }: &StructDefinition) -> String {
    let mut content = String::default();

    content.push_str(DERIVE);
    content.push_str("\n");
    content.push_str(&format!("pub struct {} {{\n", name));
    for field in fields {
        content.push_str(&format!("\tpub {}: {},\n", field.name, field.r#type));
    }
    content.push_str("}\n");

    content
}

#[cfg(test)]
mod tests {

    use schemars::schema_for;

    use super::*;

    #[ignore = "for making examples"]
    #[test]
    fn yaml_example() {
        let spec = Specification {
            types: vec![
                TypeDefinition::Struct(StructDefinition {
                    name: TypeName("Point".to_string()),
                    fields: vec![
                        Field {
                            name: "x".to_string(),
                            r#type: TypeName("f64".to_string()),
                        },
                        Field {
                            name: "y".to_string(),
                            r#type: TypeName("f64".to_string()),
                        },
                    ],
                }),
                TypeDefinition::Enum(EnumDefinition {
                    name: TypeName("Color".to_string()),
                    variants: vec![EnumVariant::Unit {
                        name: TypeName("Red".to_string()),
                        tag: None,
                    }],
                }),
            ],
            requests: EnumDefinition {
                name: TypeName("Request".to_string()),
                variants: vec![],
            },
            responses: EnumDefinition {
                name: TypeName("Response".to_string()),
                variants: vec![],
            },
        };

        println!("ex: {:#?}", yaml_serde::to_string(&spec).unwrap());
    }

    #[ignore = "for making examples"]
    #[test]
    fn toml_example() {
        let spec = Specification {
            types: vec![TypeDefinition::Struct(StructDefinition {
                name: TypeName("Point".to_string()),
                fields: vec![
                    Field {
                        name: "x".to_string(),
                        r#type: TypeName("f64".to_string()),
                    },
                    Field {
                        name: "y".to_string(),
                        r#type: TypeName("f64".to_string()),
                    },
                ],
            })],
            requests: EnumDefinition {
                name: TypeName("Request".to_string()),
                variants: vec![],
            },
            responses: EnumDefinition {
                name: TypeName("Response".to_string()),
                variants: vec![],
            },
        };

        println!("ex: {:#?}", toml::to_string(&spec).unwrap());
    }

    #[test]
    fn schema() {
        let schema = schema_for!(Specification);
        let json = serde_json::to_string_pretty(&schema).unwrap();
        write("schemas/spec-schema.json", json).unwrap();
    }
}
