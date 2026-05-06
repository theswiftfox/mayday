fn main() {
    tauri_build::build();

    // Ensure AuthenticationServices framework is linked (needed for ASWebAuthenticationSession)
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=AuthenticationServices");
}
