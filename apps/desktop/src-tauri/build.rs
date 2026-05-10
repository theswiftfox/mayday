// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
fn main() {
    tauri_build::build();

    // Rerun when capabilities change — ensures permission updates aren't missed by cargo caching
    println!("cargo:rerun-if-changed=capabilities/");

    // Ensure AuthenticationServices framework is linked (needed for ASWebAuthenticationSession)
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=AuthenticationServices");
}
