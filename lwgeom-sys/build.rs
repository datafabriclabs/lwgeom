use std::path::{Path, PathBuf};
use std::process::Command;

fn symlink_file<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) {
    if !link.as_ref().exists() {
        std::os::unix::fs::symlink(original, link).expect("failed to create symlink");
    }
}

fn run(cmd: &mut Command) {
    let status = cmd.status().expect("failed to execute process");
    if !status.success() {
        panic!("command did not execute successfully: {:?}", cmd)
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=configure.ac");
    println!("cargo:rerun-if-changed=GNUmakefile");
    println!("cargo:rerun-if-changed=postgis_revision.h");
    println!("cargo:rerun-if-changed=postgis");
    #[cfg(feature = "mvt")]
    {
        println!("cargo:rerun-if-changed=mvt.h");
        println!("cargo:rerun-if-changed=mvt.c");
    }

    let src = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let dst = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    let postgis_dst = dst.join("postgis");
    if !postgis_dst.exists() {
        std::fs::create_dir(&postgis_dst).expect("failed to create directory");
    }
    let macros_dst = postgis_dst.join("macros");
    if !macros_dst.exists() {
        std::fs::create_dir(&macros_dst).expect("failed to create directory");
    }
    let liblwgeom_dst = postgis_dst.join("liblwgeom");
    if !liblwgeom_dst.exists() {
        std::fs::create_dir(&liblwgeom_dst).expect("failed to create directory");
    }
    let ryu_dst = postgis_dst.join("deps/ryu");
    if !ryu_dst.exists() {
        std::fs::create_dir_all(&ryu_dst).expect("failed to create directory");
    }

    for path in ["configure.ac", "GNUmakefile", "postgis_revision.h"] {
        symlink_file(src.join(path), postgis_dst.join(path));
    }
    for path in [
        "postgis/autogen.sh",
        "postgis/postgis_config.h.in",
        "postgis/Version.config",
        "postgis/deps/ryu/Makefile.in",
        "postgis/liblwgeom/liblwgeom.h.in",
        "postgis/liblwgeom/lwin_wkt_lex.l",
        "postgis/liblwgeom/lwin_wkt_parse.y",
        "postgis/liblwgeom/Makefile.in",
    ] {
        symlink_file(src.join(path), dst.join(path));
    }
    for pattern in [
        "postgis/macros/*.m4",
        "postgis/deps/ryu/*.h",
        "postgis/deps/ryu/*.c",
        "postgis/liblwgeom/*.h",
        "postgis/liblwgeom/*.c",
    ] {
        for entry in glob::glob(pattern).unwrap() {
            let path = entry.unwrap();
            symlink_file(src.join(&path), dst.join(&path));
        }
    }

    run(Command::new("sh")
        .current_dir(&postgis_dst)
        .arg("autogen.sh"));
    run(Command::new("sh")
        .current_dir(&postgis_dst)
        .arg("configure")
        .arg("--disable-shared")
        .args([
            "--without-pgconfig",
            "--without-libiconv-prefix",
            "--without-libintl-prefix",
            "--without-json",
            "--without-protobuf",
            "--without-phony-revision",
            "--without-address-standardizer",
            "--without-topology",
            "--without-raster",
        ]));
    run(Command::new("make")
        .current_dir(&postgis_dst)
        .args(["-C", "liblwgeom", "all"])
        .args(["-j", &std::env::var("NUM_JOBS").unwrap()]));

    println!(
        "cargo:rustc-link-search=native={}",
        liblwgeom_dst.join(".libs").display()
    );
    println!("cargo:rustc-link-lib=static=lwgeom");

    let proj_lib = pkg_config::Config::new()
        .probe("proj")
        .expect("No package 'proj' found");
    println!(
        "cargo:rustc-link-search=native={}",
        proj_lib.link_paths[0].display()
    );
    println!("cargo:rustc-link-lib=proj");

    let geos_lib = pkg_config::Config::new()
        .probe("geos")
        .expect("No package 'geos' found");
    println!(
        "cargo:rustc-link-search=native={}",
        geos_lib.link_paths[0].display()
    );
    println!("cargo:rustc-link-lib=geos_c");

    #[cfg(feature = "mvt")]
    {
        let wagyu_dst = postgis_dst.join("deps/wagyu/include/mapbox/geometry/wagyu");
        if !wagyu_dst.exists() {
            std::fs::create_dir_all(&wagyu_dst).expect("failed to create directory");
        }

        for pattern in [
            "postgis/deps/wagyu/*.h",
            "postgis/deps/wagyu/*.cpp",
            "postgis/deps/wagyu/include/mapbox/*.hpp",
            "postgis/deps/wagyu/include/mapbox/geometry/*.hpp",
            "postgis/deps/wagyu/include/mapbox/geometry/wagyu/*.hpp",
        ] {
            for entry in glob::glob(pattern).unwrap() {
                let path = entry.unwrap();
                symlink_file(src.join(&path), dst.join(&path));
            }
        }

        for path in ["mvt.h", "mvt.c"] {
            symlink_file(src.join(path), liblwgeom_dst.join(path));
        }

        cc::Build::new()
            .cpp(true)
            .std("c++14")
            .file(postgis_dst.join("deps/wagyu/lwgeom_wagyu.cpp"))
            .include(postgis_dst.join("deps/wagyu/include"))
            .include(&liblwgeom_dst)
            .include(&proj_lib.include_paths[0])
            .compile("lwgeom_wagyu");
        cc::Build::new()
            .file(liblwgeom_dst.join("mvt.c"))
            .include(postgis_dst.join("deps/wagyu"))
            .include(&liblwgeom_dst)
            .include(&proj_lib.include_paths[0])
            .warnings(false)
            .compile("lwgeom_mvt");
    }

    let builder = bindgen::Builder::default()
        .header(
            liblwgeom_dst
                .join("liblwgeom.h")
                .to_string_lossy()
                .into_owned(),
        )
        .header(
            liblwgeom_dst
                .join("liblwgeom_topo.h")
                .to_string_lossy()
                .into_owned(),
        )
        .header(
            liblwgeom_dst
                .join("lwtree.h")
                .to_string_lossy()
                .into_owned(),
        );

    #[cfg(feature = "mvt")]
    let builder = builder.header(liblwgeom_dst.join("mvt.h").to_string_lossy().into_owned());

    let bindings = builder
        .clang_arg(format!("-I{}", proj_lib.include_paths[0].display()))
        .ctypes_prefix("libc")
        .use_core()
        .wrap_static_fns(true)
        .wrap_static_fns_path(liblwgeom_dst.join("wrap_static_fns"))
        .generate()
        .expect("Unable to generate bindings");

    cc::Build::new()
        .file(liblwgeom_dst.join("wrap_static_fns.c"))
        .compile("wrap_static_fns");

    bindings
        .write_to_file(dst.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
