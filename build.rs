use dotenv;

fn main() {
    let dotenv_path = dotenv::dotenv().expect("failed to find .env file");
    println!("cargo:rerun-if-changed={}", dotenv_path.display());

    // Warning: `dotenv_iter()` is deprecated! Roll your own or use a maintained fork such as `dotenvy`.
    for key in dotenv::vars().map(|(k, _)| k) {
        let value = dotenv::var(&key).unwrap();
        println!("cargo:rustc-env={key}={value}");
    }
}
