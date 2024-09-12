fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");
    pyo3_build_config::use_pyo3_cfgs();
}
