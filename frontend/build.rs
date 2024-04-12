use std::path::Path;

fn main() {
    let themes: Vec<String> = serde_json::from_str(
        &std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("daisy-themes.json"))
            .unwrap(),
    )
    .unwrap();

    std::fs::write(
        Path::new(&std::env::var("OUT_DIR").unwrap()).join("daisy-themes.rs"),
        "use strum::{AsRefStr, EnumIter};\n".to_string()
            + "use serde::{Serialize, Deserialize};\n"
            + "#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumIter, \
               PartialEq, Eq)]\n"
            + "#[strum(serialize_all = \"snake_case\")]\n"
            + "pub enum DaisyTheme {\n"
            + &themes
                .into_iter()
                .map(|mut t| t.remove(0).to_ascii_uppercase().to_string() + &t + ",\n")
                .collect::<Vec<_>>()
                .join("")
            + "}",
    )
    .unwrap();
}
