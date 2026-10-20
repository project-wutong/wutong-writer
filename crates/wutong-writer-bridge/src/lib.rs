// SPDX-License-Identifier: AGPL-3.0-only
//! C++ ⇄ Rust bridge (cxx). The GUI owns no protocol or file-format logic; it calls in here.

#[cxx::bridge(namespace = "wutong::writer")]
mod ffi {
    extern "Rust" {
        /// `wutong-core-rs/<version>`.
        fn core_version() -> String;
        /// Human-readable summary of a file (or socket for Network) for the status bar.
        fn describe_path(path: &str) -> Result<String>;
    }
}

fn core_version() -> String {
    wutong_core::IMPLEMENTATION.to_owned()
}

fn describe_path(path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    Ok(wutong_writer_core::describe(bytes))
}

#[cfg(test)]
mod tests {
    #[test]
    fn version() {
        assert!(super::core_version().starts_with("wutong-core-rs/"));
    }
}
