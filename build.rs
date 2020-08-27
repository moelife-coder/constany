fn main() {
    if let Ok(i) = std::env::var("NO_BUILD") {
        if &i == "true" {
            return;
        }
    }
    std::process::Command::new(std::env::var("CARGO").unwrap())
        .args(&["run", "--release", "--features", "stage_one"])
        .env("NO_BUILD", "true")
        .status()
        .unwrap();
    println!("cargo:rustc-cfg=feature=\"stage_two\"");
}
