use chrono::Utc;

fn main() {
    let now = Utc::now();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", now.format("%d %b %Y"))
}
