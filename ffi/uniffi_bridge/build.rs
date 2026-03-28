fn main() {
    uniffi::generate_scaffolding("src/dcg_uniffi.udl").expect("generate uniffi scaffolding");
}
