// SPDX-License-Identifier: AGPL-3.0-only
fn main() {
    cxx_build::bridge("src/lib.rs")
        .std("c++17")
        .compile("wutong-writer-bridge-cxx");
    println!("cargo:rerun-if-changed=src/lib.rs");
}
