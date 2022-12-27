fn main() {
    if Ok("release".to_owned()) == std::env::var("PROFILE") {
        static_vcruntime::metabuild();
    }
}
