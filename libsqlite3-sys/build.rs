fn main() {
    build::main();
}

#[cfg(feature = "bundled")]
mod build {
    extern crate cc;
    use std::{env, fs};
    use std::path::Path;

    pub fn main() {
        let out_dir = env::var("OUT_DIR").unwrap();
        let out_path = Path::new(&out_dir).join("bindgen.rs");

        let bundled_file = if cfg!(feature = "sqlcipher") {
            "sqlcipher/bundled_sqlcipher.rs"
        } else {
            "sqlite3/bindgen_bundled_version.rs"
        };

        fs::copy(bundled_file, out_path)
            .expect("Could not copy bindings to output directory");

        let mut cfg = cc::Build::new();
//        cfg.flag("-DSQLITE_CORE")
//            .flag("-DSQLITE_DEFAULT_FOREIGN_KEYS=1")
//            .flag("-DSQLITE_ENABLE_API_ARMOR")
//            .flag("-DSQLITE_ENABLE_COLUMN_METADATA")
//            .flag("-DSQLITE_ENABLE_DBSTAT_VTAB")
//            .flag("-DSQLITE_ENABLE_FTS3")
//            .flag("-DSQLITE_ENABLE_FTS3_PARENTHESIS")
//            .flag("-DSQLITE_ENABLE_FTS5")
//            .flag("-DSQLITE_ENABLE_JSON1")
//            .flag("-DSQLITE_ENABLE_LOAD_EXTENSION=1")
//            .flag("-DSQLITE_ENABLE_MEMORY_MANAGEMENT")
//            .flag("-DSQLITE_ENABLE_RTREE")
//            .flag("-DSQLITE_ENABLE_STAT2")
//            .flag("-DSQLITE_ENABLE_STAT4")
//            .flag("-DSQLITE_HAVE_ISNAN")
//            .flag("-DSQLITE_SOUNDEX")
//            .flag("-DSQLITE_THREADSAFE=2")
//            .flag("-DSQLITE_USE_URI")
//            .flag("-DHAVE_USLEEP=1");
        if cfg!(feature = "optimize") {
//            cfg.flag("-DSQLITE_OMIT_SHARED_CACHE")
//                .flag("-DSQLITE_DEFAULT_MEMSTATUS=0")
//                .flag("-DSQLITE_OMIT_PROGRESS_CALLBACK")
//                .flag("-DSQLITE_DEFAULT_WAL_SYNCHRONOUS=1")
//                .flag("-DSQLITE_ENABLE_UPDATE_DELETE_LIMIT=1");
        }

        if cfg!(feature = "secure_delete") {
//            cfg.flag("-DSQLITE_SECURE_DELETE");
        }

        if cfg!(feature = "unlock_notify") {
//            cfg.flag("-DSQLITE_ENABLE_UNLOCK_NOTIFY");
        }

        if cfg!(feature = "sqlcipher") {
            cfg.file("sqlcipher/sqlite3.c");
//                .flag("-DSQLITE_HAS_CODEC")
//                .flag("-DSQLITE_TEMP_STORE=2");

            let target = env::var("TARGET").unwrap();
            let host = env::var("HOST").unwrap();
            let mut openssl_dir_env_name = "OPENSSL_DIR";
            if target == host {
                const HOST_OPENSSL_DIR: &'static str = "HOST_OPENSSL_DIR";
                if env::var_os(HOST_OPENSSL_DIR).is_some() {
                    openssl_dir_env_name = HOST_OPENSSL_DIR;
                }
            };

            // Default to CommonCrypto on Apple systems unless the "openssl" feature is explicitly
            // given.

            if target.contains("apple") && !cfg!(feature = "openssl") {
                cfg.flag("-DSQLCIPHER_CRYPTO_CC");
                println!("cargo:rustc-link-lib=framework=CoreFoundation");
                println!("cargo:rustc-link-lib=framework=Security");
            } else if cfg!(feature = "tomcrypt") {
                cfg.flag("-DSQLCIPHER_CRYPTO_LIBTOMCRYPT");
                println!("cargo:rustc-link-lib=tomcrypt");
            } else {
                cfg.flag("-DSQLCIPHER_CRYPTO_OPENSSL");

                if let Some(dir) = env::var_os(openssl_dir_env_name) {
                    let ssl_root = Path::new(&dir);
                    cfg.flag(&format!("{}{}", "-I", ssl_root.join("include").to_str().unwrap()));
                    println!("cargo:rustc-link-search={}", ssl_root.join("lib").to_str().unwrap());
                    if target.contains("windows") {
                        println!("cargo:rustc-link-lib={}={}", "static", "crypto");
                        println!("cargo:rustc-link-lib=gdi32");
                        if target.contains("msvc") {
                            println!("cargo:rustc-link-lib=user32");
                            println!("cargo:rustc-link-lib=crypt32");
                        }
                    } else {
                        println!("cargo:rustc-link-lib=crypto");
                    }
                }
            }
        } else {
            cfg.file("sqlite3/sqlite3.c");
        }

        cfg.compile("libsqlite3.a");
    }
}

