const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");
const APPLICATION_IS_RELEASE: bool = !cfg!(debug_assertions);

fn main() {
    println!("Hello, world!");
}
