/*
 * Copyright 2016-2018 Nathan Fiedler
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
extern crate bindgen;
extern crate pkg_config;

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

static HEADER: &'static str = "#include <MagickWand/MagickWand.h>\n";

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_path_str = out_dir.join("bindings.rs");

    #[derive(Debug)]
    struct IgnoreMacros(HashSet<String>);

    impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
        fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
            if self.0.contains(name) {
                bindgen::callbacks::MacroParsingBehavior::Ignore
            } else {
                bindgen::callbacks::MacroParsingBehavior::Default
            }
        }
    }

    let ignored_macros = IgnoreMacros(
        vec![
            "FP_INFINITE".into(),
            "FP_NAN".into(),
            "FP_NORMAL".into(),
            "FP_SUBNORMAL".into(),
            "FP_ZERO".into(),
            "IPPORT_RESERVED".into(),
            "FP_INT_UPWARD".into(),
            "FP_INT_DOWNWARD".into(),
            "FP_INT_TOWARDZERO".into(),
            "FP_INT_TONEARESTFROMZERO".into(),
            "FP_INT_TONEAREST".into(),
        ]
        .into_iter()
        .collect(),
    );

    if !Path::new(&bindings_path_str).exists() {
        // Create the header file that rust-bindgen needs as input.
        let gen_h_path = out_dir.join("gen.h");
        let mut gen_h = File::create(&gen_h_path).expect("could not create file");
        gen_h
            .write_all(HEADER.as_bytes())
            .expect("could not write header file");

        // Geneate the bindings.
        let mut builder = bindgen::Builder::default()
            .emit_builtins()
            .ctypes_prefix("libc")
            .raw_line("extern crate libc;")
            .header(gen_h_path.to_str().unwrap())
            .size_t_is_usize(true)
            .parse_callbacks(Box::new(ignored_macros))
            .blacklist_type("timex")
            .blacklist_function("clock_adjtime");

        let include_dirs = env::var("DEP_IMAGEMAGICK_INCLUDE")
            .expect("DEP_IMAGEMAGICK_INCLUDE should have been set by imagemagick-sys crate");
        builder = builder.clang_arg(format!("-I{}", include_dirs));

        let bindings = builder.generate().unwrap();
        let mut file = File::create(&bindings_path_str).expect("could not create bindings file");
        // Work around the include! issue in rustc (as described in the
        // rust-bindgen README file) by wrapping the generated code in a
        // `pub mod` declaration; see issue #359 in (old) rust-bindgen.
        file.write(b"pub mod bindings {\n").unwrap();
        file.write(bindings.to_string().as_bytes()).unwrap();
        file.write(b"\n}").unwrap();

        std::fs::remove_file(&gen_h_path).expect("could not remove header file");
    }
}
