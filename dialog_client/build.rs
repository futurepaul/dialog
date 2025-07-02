fn main() {
    // This tells cargo to rerun this script if the UDL file changes
    println!("cargo:rerun-if-changed=src/dialog_client.udl");
    
    // Generate the Rust scaffolding from the UDL file
    uniffi::generate_scaffolding("src/dialog_client.udl").unwrap();
}