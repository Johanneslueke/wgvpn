#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use std::env;
use std::path::PathBuf;

const libctypes: &[&str] = &[
    "in_addr",
    "in6_addr",
    "sockaddr",
    "sockaddr_in",
    "sockaddr_in6",
    "timespec",
];

const ignoretypes: &[&str] = &[
    "in6_addr__bindgen_ty_1",
    "sa_family_t",
    "in_addr_t",
    "in_port_t",
];

fn main() {
    //println!("cargo:rustc-link-lib=clib");
    println!("cargo:rerun-if-changed=wg/wireguard.h");

    let mut bindings = bindgen::Builder::default()
        .header("wg/wireguard.h")
        .impl_debug(true)
        .clang_arg("-fretain-comments-from-system-headers")
        .sort_semantically(true)
        .generate_comments(true)
        .generate_block(true)
        .generate_cstr(true)
        //.explicit_padding(true)
        //.translate_enum_integer_types(true)
        .enable_function_attribute_detection()
        .wrap_unsafe_ops(true)
        .use_core()
        .array_pointers_in_arguments(true)
        // .must_use_type("int")
        // .must_use_type("bool")
        // .must_use_type("char")
        // .must_use_type(r"char \*")
        .allowlist_function("wg_.*")
        .bitfield_enum("wg_peer_flags")
        .bitfield_enum("wg_device_flags")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .raw_line("extern crate libc;");

    for libc_type in libctypes {
        bindings = bindings
            .blocklist_type(libc_type)
            .raw_line(format!("use libc::{};", libc_type));
    }

    for useless_type in ignoretypes {
        bindings = bindings.blocklist_type(useless_type);
    }


    let bindings = bindings.generate().expect("Couldn't generate bindings");
    if std::env::var("DOCS_RS").is_err() {
        let out_path = PathBuf::from("./src");
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect(" Could write bindings");
    }

        

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write to file");

    cc::Build::new()
        .file("wg/wireguard.c")
        .warnings(true)
        .extra_warnings(true)
        .warnings_into_errors(true)
        .flag_if_supported("-Wno-unused-parameter")
        .compile("wireguard.a");
}
