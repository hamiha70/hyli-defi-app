// Deactivate in clippy, we don't really need to recompile and it breaks our CI.
#[cfg(any(clippy, not(feature = "build")))]
fn main() {}

#[cfg(all(
    feature = "build",
    not(feature = "contract1")
))]
fn main() {
    compile_error!("When the 'build' feature is enabled, contract1 feature must also be enabled.");
}

#[cfg(all(
    not(clippy),
    feature = "build",
    feature = "contract1"
))]
fn main() {
    // First compile RISC0 contracts
    compile_risc0_contracts();
    
    // Then compile Noir contracts for UltraHonk backend
    compile_noir_contracts();
}

fn compile_risc0_contracts() {
    trait CodegenConsts {
        fn codegen_consts(&self) -> String;
    }

    impl CodegenConsts for risc0_build::GuestListEntry {
        fn codegen_consts(&self) -> String {
            use std::fmt::Write;
            // Quick check for '#' to avoid injection of arbitrary Rust code into the
            // method.rs file. This would not be a serious issue since it would only
            // affect the user that set the path, but it's good to add a check.
            if self.path.contains('#') {
                panic!("method path cannot include #: {}", self.path);
            }

            let upper = self.name.to_uppercase().replace('-', "_");

            let image_id = self.image_id.as_words();
            let elf = format!("include_bytes!({:?})", self.path);

            let mut str = String::new();

            writeln!(&mut str, "pub const {upper}_ELF: &[u8] = {elf};").unwrap();
            writeln!(&mut str, "pub const {upper}_PATH: &str = {:?};", self.path).unwrap();
            writeln!(&mut str, "pub const {upper}_ID: [u32; 8] = {image_id:?};").unwrap();

            str
        }
    }

    // clippy in workspace mode sets this, which interferes with the guest VM. Clear it temporarily.
    let env_wrapper = std::env::var("RUSTC_WORKSPACE_WRAPPER");
    std::env::set_var("RUSTC_WORKSPACE_WRAPPER", "");

    use risc0_build::{
        build_package, get_package, DockerOptionsBuilder, GuestListEntry, GuestOptionsBuilder,
    };
    use std::io::Write;

    let reproducible = cfg!(not(feature = "nonreproducible"));

    let pkg = get_package(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let manifest_dir = pkg.manifest_path.parent().unwrap();

    let methods: Vec<GuestListEntry> = [
        "contract1",
        // contract2 removed - replaced with Noir identity verification
    ]
    .iter()
    .map(|name| {
        let pkg = get_package(manifest_dir.join(name));
        let mut guest_opts = GuestOptionsBuilder::default();

        guest_opts.features(vec!["risc0".into()]);

        if reproducible {
            guest_opts.use_docker(
                DockerOptionsBuilder::default()
                    // Point to the workspace
                    .root_dir("..".to_string())
                    .build()
                    .unwrap(),
            );
        }

        build_package(
            &pkg,
            std::env::var("OUT_DIR").expect("missing OUT_DIR env var"),
            guest_opts.build().expect("failed to build guest options"),
        )
    })
    .flatten()
    .flatten()
    .collect();

    let out_dir_env = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = std::path::Path::new(&out_dir_env);

    let methods_path = out_dir.join("methods.rs");
    let mut methods_file = std::fs::File::create(&methods_path).unwrap();

    for method in methods.iter() {
        methods_file
            .write_all(method.codegen_consts().as_bytes())
            .unwrap();
    }

    // if reproducible {
    methods.iter().for_each(|data| {
        std::fs::write(format!("{}/{}.img", data.name, data.name), &data.elf)
            .expect("failed to write img");
        // Convert u32 slice to hex
        let hex_image_id = data
            .image_id
            .as_words()
            .iter()
            .map(|x| format!("{:08x}", x.to_be()))
            .collect::<Vec<_>>()
            .join("");
        std::fs::write(format!("{}/{}.txt", data.name, data.name), &hex_image_id)
            .expect("failed to write program ID");
    });
    // }
    std::env::set_var("RUSTC_WORKSPACE_WRAPPER", env_wrapper.unwrap_or_default());
}

fn compile_noir_contracts() {
    use std::process::Command;
    use std::io::Write;

    println!("cargo:rerun-if-changed=../noir-contracts/zkpassport_identity/src");
    println!("cargo:rerun-if-changed=../noir-contracts/zkpassport_identity/Nargo.toml");

    println!("🔮 Compiling Noir contracts with UltraHonk backend...");

    // Compile Noir contract to UltraHonk backend
    let noir_output = Command::new("nargo")
        .args(["compile"])
        .current_dir("../noir-contracts/zkpassport_identity")
        .output()
        .expect("Failed to execute nargo compile. Ensure Noir is installed.");

    if !noir_output.status.success() {
        let stderr = String::from_utf8_lossy(&noir_output.stderr);
        let stdout = String::from_utf8_lossy(&noir_output.stdout);
        panic!(
            "Noir compilation failed!\nSTDOUT:\n{}\nSTDERR:\n{}", 
            stdout, stderr
        );
    }

    println!("✅ Noir contract compiled successfully");

    // Generate Noir contract constants
    let out_dir_env = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = std::path::Path::new(&out_dir_env);
    
    let noir_constants_path = out_dir.join("noir_constants.rs");
    let mut constants_file = std::fs::File::create(&noir_constants_path).unwrap();

    // Add Noir contract constants
    writeln!(
        &mut constants_file,
        r#"
// Noir contract constants for UltraHonk integration
pub const ZKPASSPORT_IDENTITY_CONTRACT_PATH: &str = "../noir-contracts/zkpassport_identity/target/zkpassport_identity.json";
pub const ZKPASSPORT_IDENTITY_VERIFICATION_KEY_PATH: &str = "../noir-contracts/zkpassport_identity/target/vk";

// Contract metadata
pub const ZKPASSPORT_IDENTITY_CONTRACT_NAME: &str = "zkpassport_identity";
"#
    ).unwrap();

    println!("✅ Noir contract constants generated");
}
