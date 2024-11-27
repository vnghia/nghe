fn main() {
    built::write_built_file().expect("Could not acquire build-time information");
}
