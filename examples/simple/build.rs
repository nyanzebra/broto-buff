fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=./spec");

    struct Parser;

    impl broto_buff::Parse for Parser {
        type Error = toml::de::Error;
        fn parse(
            &self,
            content: &impl AsRef<str>,
        ) -> Result<broto_buff::Specification, Self::Error> {
            let content = content.as_ref();
            let spec: broto_buff::Specification = toml::from_str(content)
                .map_err(|e| e.into())
                .inspect_err(|e| println!("cargo:warning=failed to parse: {e:?}"))?;
            println!("cargo:info=parsed spec: {spec:?}");
            Ok(spec)
        }
    }

    broto_buff::generate(Parser, "./spec", "./src/api", "toml")?;

    Ok(())
}
