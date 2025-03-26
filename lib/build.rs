fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_USAGE_TRACKING");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_NO_USAGE_TRACKING");
    println!("cargo::rustc-check-cfg=cfg(usage_tracking_enabled)");

    let is_release = std::env::var("PROFILE").map(|v| v == "release").unwrap_or(false);
    let usage_tracking = std::env::var("CARGO_FEATURE_USAGE_TRACKING").is_ok();
    let no_usage_tracking = std::env::var("CARGO_FEATURE_NO_USAGE_TRACKING").is_ok();

    if (!is_release || usage_tracking) && !no_usage_tracking {
        println!("cargo:rustc-cfg=usage_tracking_enabled");
    }
}
