use std::path::Path;

pub fn yaml_string(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    )
}

pub fn display_path(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    if let Some(path) = path.strip_prefix("//?/UNC/") {
        format!("//{path}")
    } else {
        path.strip_prefix("//?/").unwrap_or(&path).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_yaml_strings() {
        assert_eq!(yaml_string("a \"quote\""), "\"a \\\"quote\\\"\"");
    }

    #[test]
    fn hides_windows_verbatim_prefixes() {
        assert_eq!(
            display_path(Path::new(r"\\?\C:\Users\Sabrina\project")),
            "C:/Users/Sabrina/project"
        );
        assert_eq!(
            display_path(Path::new(r"\\?\UNC\server\share\project")),
            "//server/share/project"
        );
    }
}
