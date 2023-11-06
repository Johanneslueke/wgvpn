use std::env;
use std::path::PathBuf;

fn main() {

    //println!("cargo:rustc-link-lib=clib");
    println!("cargo:rerun-if-changed=wg/wireguard.h");

    cc::Build::new()
        .file("wg/wireguard.c")
        .compile("wireguard.a");

    let bindings = bindgen::Builder::default()
        .header("wg/wireguard.h")
        .allowlist_function("^wg_set_device$")
        .allowlist_function("^wg_get_device$")
        .allowlist_function("^wg_add_device$")
        .allowlist_function("^wg_del_device$")
        .allowlist_function("^wg_free_device$")
        .allowlist_function("^wg_list_device_names$")
        .allowlist_function("^wg_key_to_base64$")
        .allowlist_function("^wg_key_from_base64$")
        .allowlist_function("^wg_key_is_zero$")
        .allowlist_function("^wg_generate_public_key$")
        .allowlist_function("^wg_generate_private_key$")
        .allowlist_function("^wg_generate_preshared_key$")
        .allowlist_type("^wg_key$")
        .allowlist_type("^wg_device$")
        .allowlist_type("^wg_key_b64_string$")
        .bitfield_enum("^wg_device_flags$")
        .bitfield_enum("^wg_peer_flags$")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Couldn't generate bindings");

    let out_path = PathBuf::from("./src");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect(" Could write bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Could not write to file");
}