fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=./spec");

    struct Parser;

    impl broto_buff::Parse for Parser {
        type Error = yaml_serde::Error;
        fn parse(
            &self,
            content: &impl AsRef<str>,
        ) -> Result<broto_buff::Specification, Self::Error> {
            let content = content.as_ref();
            let spec: broto_buff::Specification = yaml_serde::from_str(content)?;
            Ok(spec)
        }
    }

    broto_buff::generate(Parser, "./spec", "./src/api", "yaml")?;

    Ok(())
}
