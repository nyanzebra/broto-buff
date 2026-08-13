fn main() -> std::io::Result<()> {
    struct Parser;

    impl broto_buff::Parse for Parser {
        type Error = toml::de::Error;
        fn parse(
            &self,
            content: &impl AsRef<str>,
        ) -> Result<broto_buff::Specification, Self::Error> {
            let content = content.as_ref();
            let spec: broto_buff::Specification = toml::from_str(content).map_err(|e| e.into())?;
            Ok(spec)
        }
    }

    broto_buff::generate(Parser, "./spec", "./src/api", "toml")?;

    Ok(())
}
