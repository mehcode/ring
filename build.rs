// Copyright 2015-2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

// TODO: Deny `unused_qualifications` after
// https://github.com/rust-lang/rust/issues/37345 is fixed.
#![deny(
    const_err,
    dead_code,
    deprecated,
    exceeding_bitshifts,
    fat_ptr_transmutes,
    improper_ctypes,
    missing_copy_implementations,
    missing_debug_implementations,
    mutable_transmutes,
    no_mangle_const_items,
    non_camel_case_types,
    non_shorthand_field_patterns,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    path_statements,
    plugin_as_library,
    private_no_mangle_fns,
    private_no_mangle_statics,
    stable_features,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unknown_crate_types,
    unknown_lints,
    unreachable_code,
    unsafe_code,
    unstable_features,
    unused_allocation,
    unused_assignments,
    unused_attributes,
    unused_comparisons,
    unused_extern_crates,
    unused_features,
    unused_import_braces,
    unused_imports,
    unused_must_use,
    unused_mut,
    unused_parens,
    unused_results,
    unused_unsafe,
    unused_variables,
    variant_size_differences,
    warnings,
    while_true,
)]

extern crate gcc;
extern crate rayon;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs::{self, DirEntry};
use rayon::par_iter::{ParallelIterator, IntoParallelIterator,
                      IntoParallelRefIterator};

#[cfg_attr(rustfmt, rustfmt_skip)]
const RING_SRC: &'static [&'static str] =
    &["crypto/aes/aes.c",
      "crypto/bn/add.c",
      "crypto/bn/bn.c",
      "crypto/bn/cmp.c",
      "crypto/bn/convert.c",
      "crypto/bn/div.c",
      "crypto/bn/exponentiation.c",
      "crypto/bn/gcd.c",
      "crypto/bn/generic.c",
      "crypto/bn/montgomery.c",
      "crypto/bn/montgomery_inv.c",
      "crypto/bn/mul.c",
      "crypto/bn/random.c",
      "crypto/bn/shift.c",
      "crypto/cipher/e_aes.c",
      "crypto/crypto.c",
      "crypto/curve25519/curve25519.c",
      "crypto/ec/ecp_nistz.c",
      "crypto/ec/ecp_nistz256.c",
      "crypto/ec/gfp_p256.c",
      "crypto/ec/gfp_p384.c",
      "crypto/mem.c",
      "crypto/modes/gcm.c",
      "crypto/rand/sysrand.c",
      "crypto/limbs/limbs.c"];

const RING_INTEL_SHARED_SRCS: &'static [&'static str] = &["crypto/cpu-intel.c"];

const RING_X86_SRCS: &'static [&'static str] =
    &["crypto/aes/asm/aes-586.pl",
      "crypto/aes/asm/aesni-x86.pl",
      "crypto/aes/asm/vpaes-x86.pl",
      "crypto/bn/asm/x86-mont.pl",
      "crypto/chacha/asm/chacha-x86.pl",
      "crypto/ec/asm/ecp_nistz256-x86.pl",
      "crypto/modes/asm/ghash-x86.pl",
      "crypto/poly1305/asm/poly1305-x86.pl",
      "crypto/sha/asm/sha256-586.pl",
      "crypto/sha/asm/sha512-586.pl"];

const RING_X86_64_SRC: &'static [&'static str] =
    &["crypto/aes/asm/aes-x86_64.pl",
      "crypto/aes/asm/aesni-x86_64.pl",
      "crypto/aes/asm/bsaes-x86_64.pl",
      "crypto/aes/asm/vpaes-x86_64.pl",
      "crypto/bn/asm/x86_64-mont.pl",
      "crypto/bn/asm/x86_64-mont5.pl",
      "crypto/chacha/asm/chacha-x86_64.pl",
      "crypto/curve25519/asm/x25519-asm-x86_64.S",
      "crypto/curve25519/x25519-x86_64.c",
      "crypto/ec/asm/ecp_nistz256-x86_64.pl",
      "crypto/ec/asm/p256-x86_64-asm.pl",
      "crypto/modes/asm/aesni-gcm-x86_64.pl",
      "crypto/modes/asm/ghash-x86_64.pl",
      "crypto/poly1305/asm/poly1305-x86_64.pl",
      "crypto/sha/asm/sha256-x86_64.pl",
      "crypto/sha/asm/sha512-x86_64.pl"];

#[cfg_attr(rustfmt, rustfmt_skip)]
const RING_ARM_SHARED_SRCS: &'static [&'static str] =
    &["crypto/cpu-arm.c",
      "crypto/cpu-arm-linux.c",
      "crypto/aes/asm/aesv8-armx.pl",
      "crypto/modes/asm/ghashv8-armx.pl"];

const RING_ARM_SRCS: &'static [&'static str] =
    &["crypto/aes/asm/aes-armv4.pl",
      "crypto/aes/asm/bsaes-armv7.pl",
      "crypto/bn/asm/armv4-mont.pl",
      "crypto/chacha/asm/chacha-armv4.pl",
      "crypto/curve25519/asm/x25519-asm-arm.S",
      "crypto/ec/asm/ecp_nistz256-armv4.pl",
      "crypto/modes/asm/ghash-armv4.pl",
      "crypto/poly1305/asm/poly1305-armv4.pl",
      "crypto/sha/asm/sha256-armv4.pl",
      "crypto/sha/asm/sha512-armv4.pl"];

const RING_AARCH64_SRCS: &'static [&'static str] =
    &["crypto/cpu-aarch64-linux.c",
      "crypto/bn/asm/armv8-mont.pl",
      "crypto/chacha/asm/chacha-armv8.pl",
      "crypto/ec/asm/ecp_nistz256-armv8.pl",
      "crypto/poly1305/asm/poly1305-armv8.pl",
      "crypto/sha/asm/sha256-armv8.pl",
      "crypto/sha/asm/sha512-armv8.pl"];

#[cfg_attr(rustfmt, rustfmt_skip)]
const RING_PPC_SRCS: &'static [&'static str] =
    &["crypto/aes/asm/aesp8-ppc.pl",
      "crypto/cpu-ppc64le.c"];

const RING_TEST_SRCS: &'static [&'static str] =
    &["crypto/bn/bn_test.cc",
      "crypto/constant_time_test.c",
      "crypto/test/bn_test_convert.c",
      "crypto/test/bn_test_lib.c",
      "crypto/test/bn_test_new.c",
      "crypto/test/file_test.cc"];

#[cfg_attr(rustfmt, rustfmt_skip)]
const RING_HEADERS: &'static [&'static str] =
    &["crypto/poly1305/internal.h",
      "crypto/test/scoped_types.h",
      "crypto/test/rand.h",
      "crypto/curve25519/internal.h",
      "crypto/cipher/internal.h",
      "crypto/bn/rsaz_exp.h",
      "crypto/bn/internal.h",
      "crypto/internal.h",
      "crypto/rsa/internal.h",
      "crypto/modes/internal.h",
      "crypto/ec/ecp_nistz.h",
      "crypto/ec/ecp_nistz384.h",
      "crypto/ec/ecp_nistz256.h",
      "crypto/ec/gfp_internal.h",
      "crypto/limbs/limbs.h",
      "include/openssl/type_check.h",
      "include/openssl/mem.h",
      "include/openssl/bn.h",
      "include/openssl/opensslconf.h",
      "include/openssl/arm_arch.h",
      "include/openssl/cpu.h",
      "include/openssl/rsa.h",
      "include/openssl/aes.h",
      "include/openssl/base.h",
      "include/openssl/err.h"];

const RING_TEST_HEADERS: &'static [&'static str] =
    &["crypto/test/bn_test_lib.h",
      "crypto/test/file_test.h",
      "crypto/test/bn_test_util.h"];

#[cfg_attr(rustfmt, rustfmt_skip)]
const RING_INLINE_FILES: &'static [&'static str] =
    &["crypto/ec/ecp_nistz256_table.inl",
      "crypto/ec/ecp_nistz384.inl",
      "crypto/ec/gfp_limbs.inl",
      "crypto/ec/ecp_nistz384_mul.inl",
      "crypto/limbs/limbs.inl"];

#[cfg_attr(rustfmt, rustfmt_skip)]
const RING_PERL_INCLUDES: &'static [&'static str] =
    &["crypto/sha/asm/sha-x86_64.pl",
      "crypto/sha/asm/sha-armv8.pl",
      "crypto/perlasm/x86masm.pl",
      "crypto/perlasm/x86gas.pl",
      "crypto/perlasm/x86nasm.pl",
      "crypto/perlasm/x86asm.pl",
      "crypto/perlasm/x86_64-xlate.pl",
      "crypto/perlasm/arm-xlate.pl",
      "crypto/perlasm/ppc-xlate.pl"];

const RING_BUILD_FILE: &'static [&'static str] = &["build.rs"];

fn c_flags(target: &Target) -> &'static [&'static str] {
    if target.env != "msvc" {
        static NON_MSVC_FLAGS: &'static [&'static str] = &[
            "-std=c1x", // GCC 4.6 requires "c1x" instead of "c11"
            "-Wbad-function-cast",
            "-Wmissing-prototypes",
            "-Wnested-externs",
            "-Wstrict-prototypes"
        ];
        NON_MSVC_FLAGS
    } else {
        &[]
    }
}

fn cxx_flags(target: &Target) -> &'static [&'static str] {
    if target.env != "msvc" {
        static NON_MSVC_FLAGS: &'static [&'static str] = &[
            "-std=c++0x"  // GCC 4.6 requires "c++0x" instead of "c++11"
        ];
        NON_MSVC_FLAGS
    } else {
        &[]
    }
}

fn cpp_flags(target: &Target) -> &'static [&'static str] {
    if target.env != "msvc" {
        static NON_MSVC_FLAGS: &'static [&'static str] = &[
            "-fdata-sections",
            "-ffunction-sections",
            "-pedantic",
            "-pedantic-errors",
            "-Wall",
            "-Werror",
            "-Wextra",
            "-Wcast-align",
            "-Wcast-qual",
            "-Wenum-compare",
            "-Wfloat-equal",
            "-Wformat=2",
            "-Winline",
            "-Winvalid-pch",
            "-Wmissing-declarations",
            "-Wmissing-field-initializers",
            "-Wmissing-include-dirs",
            "-Wredundant-decls",
            "-Wshadow",
            "-Wsign-compare",
            "-Wundef",
            "-Wuninitialized",
            "-Wwrite-strings",
            "-fno-strict-aliasing",
            "-fvisibility=hidden",
            "-Wno-cast-align"
        ];
        NON_MSVC_FLAGS
    } else {
        static MSVC_FLAGS: &'static [&'static str] = &[
            "-EHsc",

            "/GS", // Buffer security checks.

            "/Zc:wchar_t",
            "/Zc:forScope",
            "/Zc:inline",
            "/Zc:rvalueCast",
            "/utf-8", // Input files are Unicode.

            // Warnings.
            "/sdl",
            "/Wall",
            "/WX",
            "/wd4127", // C4127: conditional expression is constant
            "/wd4464", // C4464: relative include path contains '..'
            "/wd4514", // C4514: <name>: unreferenced inline function has be
            "/wd4710", // C4710: function not inlined
            "/wd4711", // C4711: function 'function' selected for inline expansion
            "/wd4820", // C4820: <struct>: <n> bytes padding added after <name>
        ];
        MSVC_FLAGS
    }
}

const LD_FLAGS: &'static [&'static str] = &[];

fn main() {
    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = PathBuf::from(out_dir);

    // copied from gcc
    let mut cfg = rayon::Configuration::new();
    if let Ok(amt) = env::var("NUM_JOBS") {
        if let Ok(amt) = amt.parse() {
            cfg = cfg.set_num_threads(amt);
        }
    }
    rayon::initialize(cfg).unwrap();

    let _ = rayon::join(check_all_files_tracked, || build_c_code(out_dir));
}

struct Target {
    arch: String,
    os: String,
    env: String,
    obj_ext: &'static str,
    obj_opt: &'static str,
}

impl Target {
    pub fn new() -> Target {
        let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
        let env = env::var("CARGO_CFG_TARGET_ENV").unwrap();
        let (obj_ext, obj_opt) = if env == "msvc" {
            ("obj", "/Fo")
        } else {
            ("o", "-o")
        };
        Target {
            arch: arch,
            os: os,
            env: env,
            obj_ext: obj_ext,
            obj_opt: obj_opt,
        }
    }

    pub fn arch(&self) -> &str { &self.arch }
    pub fn os(&self) -> &str { &self.os }
    pub fn env(&self) -> &str { &self.env }
}

fn build_c_code(out_dir: PathBuf) {
    let target = Target::new();
    let mut lib_target = out_dir.clone();
    lib_target.push("libring-core.a");
    let lib_target = lib_target.as_path();

    let mut test_target = out_dir.clone();
    test_target.push("libring-test.a");
    let test_target = test_target.as_path();

    let lib_header_change = RING_HEADERS.par_iter()
        .chain(RING_INLINE_FILES.par_iter())
        .chain(RING_BUILD_FILE.par_iter())
        .map(Path::new)
        .any(|p| need_run(&p, lib_target));
    let test_header_change = RING_TEST_HEADERS.par_iter()
        .map(Path::new)
        .any(|p| need_run(&p, test_target)) ||
                             lib_header_change;

    let srcs = match target.arch() {
        "x86_64" => vec![RING_X86_64_SRC, RING_INTEL_SHARED_SRCS],
        "x86" => vec![RING_X86_SRCS, RING_INTEL_SHARED_SRCS],
        "arm" => vec![RING_ARM_SHARED_SRCS, RING_ARM_SRCS],
        "aarch64" => vec![RING_ARM_SHARED_SRCS, RING_AARCH64_SRCS],
        _ => Vec::new(),
    };

    let additional = srcs.into_par_iter()
        .weight_max()
        .flat_map(|additional_src| {
            additional_src.par_iter()
                .map(|src| make_asm(src, out_dir.clone(), &target))
        });
    build_library(lib_target, additional, RING_SRC, &target, out_dir.clone(),
                  lib_header_change);

    // XXX: Ideally, this would only happen for `cargo test`,
    // but we don't know how to do that yet.
    build_library(test_target, Vec::new().into_par_iter(), RING_TEST_SRCS,
                  &target, out_dir.clone(), test_header_change);
    if target.env() != "msvc" {
        let libcxx = if use_libcxx(&target) {
            "c++"
        } else {
            "stdc++"
        };
        println!("cargo:rustc-flags=-l dylib={}", libcxx);
    }

     print_rerun();
}


fn build_library<P>(out_path: &Path, additional: P,
                    lib_src: &'static [&'static str], target: &Target,
                    out_dir: PathBuf, header_changed: bool)
    where P: ParallelIterator<Item = String>
{
    // Compile all the (dirty) source files into object files.
    let objs = additional.chain(lib_src.par_iter().map(|a| String::from(*a)))
        .weight_max()
        .filter(|f| target.env() != "msvc" || !f.ends_with(".S"))
        .map(|f| compile(&f, target, out_dir.clone(), header_changed))
        .map(|v| vec![v])
        .reduce(Vec::new,
                &|mut a: Vec<String>, b: Vec<String>| -> Vec<String> {
                    a.extend(b.into_iter());
                    a
                });

    //Rebuild the library if necessary.
    if objs.par_iter()
        .map(|f| Path::new(f))
        .any(|p| need_run(&p, out_path)) {
        let mut c = gcc::Config::new();

        for f in LD_FLAGS {
            let _ = c.flag(&f);
        }
        match target.os() {
            "macos" => {
                let _ = c.flag("-fPIC");
                let _ = c.flag("-Wl,-dead_strip");
            },
            _ => {
                let _ = c.flag("-Wl,--gc-sections".into());
            },
        }
        for o in objs {
            let _ = c.object(o);
        }
        c.compile(out_path.file_name()
            .and_then(|f| f.to_str())
            .expect("No filename"));
    }
}

fn compile(file: &str, target: &Target, mut out_dir: PathBuf,
           header_change: bool)
           -> String {
    let p = Path::new(file);
    out_dir.push(p.file_name().expect("There is a filename"));
    out_dir.set_extension(target.obj_ext);
    if header_change || need_run(&p, out_dir.as_path()) {
        let ext = p.extension().unwrap().to_str().unwrap();
        let mut c = if target.env() != "msvc" || ext != "asm" {
            cc(file, ext, target, &out_dir)
        } else {
            yasm(file, target, &out_dir)
        };

        println!("{:?}", c);
        if !c.status()
            .expect(&format!("Failed to compile {}", file))
            .success() {
            panic!("Failed to compile {}", file)
        }
    }
    out_dir.to_str().expect("Invalid path").into()
}

fn cc(file: &str, ext: &str, target: &Target, out_dir: &Path) -> Command {
    let mut c = gcc::Config::new();
    let _ = c.include("include");
    match ext {
        "c" => {
            for f in c_flags(target) {
                let _ = c.flag(f);
            }
        },
        "cc" => {
            for f in cxx_flags(target) {
                let _ = c.flag(f);
            }
            let _ = c.cpp(true);
            if use_libcxx(target) {
                let _ = c.cpp_set_stdlib(Some("c++"));
            }
        },
        "S" => {},
        e => panic!("Unsupported file extension: {:?}", e),
    };
    for f in cpp_flags(target) {
        let _ = c.flag(&f);
    }
    if target.os() != "none" &&
        target.os() != "redox" &&
        target.env() != "msvc" {
        let _ = c.flag("-fstack-protector");
    }
    match (target.os(), target.env()) {
        // ``-gfull`` is required for Darwin's |-dead_strip|.
        ("macos", _) => { let _ = c.flag("-gfull"); },
        (_, "msvc") => {},
        _ => { let _ = c.flag("-g3"); },
    };
    if env::var("PROFILE").unwrap() != "debug" {
        let _ = c.define("NDEBUG", None);
        if target.env() == "msvc" {
            let _ = c.flag("/Oi"); // Generate intrinsic functions.
        }
    } else {
        if target.env() == "msvc" {
            let _ = c.flag("/Oy-"); // Don't omit frame pointers.
            // run-time checking: (s)tack frame, (u)ninitialized variables
            let _ = c.flag("/RTCsu");
            let _ = c.flag("/Od"); // Disable optimization for debug builds.
        }
    }
    if target.env() != "msvc" {
        let _ = c.define("_XOPEN_SOURCE", Some("700"));
    }
    if target.env() == "musl" {
        // Some platforms enable _FORTIFY_SOURCE by default, but musl
        // libc doesn't support it yet. See
        // http://wiki.musl-libc.org/wiki/Future_Ideas#Fortify
        // http://www.openwall.com/lists/musl/2015/02/04/3
        // http://www.openwall.com/lists/musl/2015/06/17/1
        let _ = c.flag("-U_FORTIFY_SOURCE");
    }
    if target.os() == "android" && target.arch() == "arm" {
        //let _ = c.define("BORINGSSL_CLANG_SUPPORTS_DOT_ARCH");
        let _ = c.flag("-march=armv7-a");
        let _ = c.flag("-mthumb");
        let _ = c.flag("-fno-integrated-as");
    }

    let mut c = c.get_compiler().to_command();
    let _ = c.arg("-c")
             .arg(format!("{}{}", target.obj_opt,
                          out_dir.to_str().expect("Invalid path")))
             .arg(file);
    c
}

fn yasm(file: &str, target: &Target, out_file: &Path) -> Command {
    let (oformat, machine) = if target.arch() == "x86_64" {
        ("--oformat=win64", "--machine=amd64")
    } else {
        ("--oformat=win32", "--machine=x86")
    };
    let mut c = Command::new("yasm.exe");
    let _ = c.arg("-X").arg("vc")
             .arg("--dformat=cv8")
             .arg(oformat)
             .arg(machine)
             .arg("-o").arg(out_file.to_str().expect("Invalid path"))
             .arg(file);
    c
}

fn use_libcxx(target: &Target) -> bool {
    target.os() == "macos" ||
        target.os() == "ios" ||
        target.os() == "freebsd"
}

fn run_command_with_args<S>(command_name: S, args: &[String])
    where S: AsRef<std::ffi::OsStr> + Copy
{
    let mut cmd = Command::new(command_name);
    let _ = cmd.args(args);

    println!("running: {:?}", cmd);

    let status = cmd.status().unwrap_or_else(|e| {
        panic!("failed to execute {}: {}",
               command_name.as_ref().to_str().unwrap(), e);
    });

    if !status.success() {
        panic!("execution failed");
    }
}

fn make_asm(source: &str, mut dst: PathBuf, target: &Target)
            -> String {
    let p = Path::new(source);
    if p.extension().expect("File without extension").to_str() == Some("pl") {
        dst.push(p.file_name().expect("File without filename??"));
        dst.set_extension(if target.env() == "msvc" { "asm" } else { "S" });
        let r: String = dst.to_str().expect("Could not convert path").into();
        let perl_include_changed = RING_PERL_INCLUDES.iter()
            .any(|i| need_run(&Path::new(i), dst.as_path()));
        if need_run(&p, dst.as_path()) || perl_include_changed {
            let format = match (target.os(), target.arch()) {
                ("macos", _) => "macosx",
                ("ios", "arm") => "ios32",
                ("ios", "aarch64") => "ios64",
                ("windows", "x86_64") => "nasm",
                ("windows", "x86") => "win32n",
                (_, "aarch64") => "linux64",
                (_, "arm") => "linux32",
                _ => "elf",
            };
            let mut args = vec![source.to_owned()];
            args.push(format.into());
            if target.arch() == "x86" {
                args.push("-fPIC".into());
                args.push("-DOPENSSL_IA32_SSE2".into());
            }
            args.push(r.clone());
            run_command_with_args(&get_command("PERL_EXECUTABLE", "perl"),
                                  &args);
        }
        r
    } else {
        p.to_str().expect("Could not convert path").into()
    }
}

fn need_run(source: &Path, target: &Path) -> bool {
    let s = std::fs::metadata(source);
    let t = std::fs::metadata(target);
    if s.is_err() || t.is_err() {
        true
    } else {
        match (s.unwrap().modified(), t.unwrap().modified()) {
            (Ok(s), Ok(t)) => s >= t,
            _ => true,
        }
    }
}

fn get_command(var: &str, default: &str) -> String {
    env::var(var).unwrap_or(default.into())
}

fn check_all_files_tracked() {
    let _ = rayon::join(|| walk_dir(&PathBuf::from("crypto"), &is_tracked),
                        || walk_dir(&PathBuf::from("include"), &is_tracked));
}

fn is_tracked(file: &DirEntry) {
    let p = file.path();
    let cmp = |f| p == PathBuf::from(f);
    let tracked = match p.extension().and_then(|p| p.to_str()) {
        Some("h") => {
            RING_HEADERS.iter().chain(RING_TEST_HEADERS.iter()).any(cmp)
        },
        Some("inl") => RING_INLINE_FILES.iter().any(cmp),
        Some("c") | Some("cc") => {
            RING_SRC.iter()
                .chain(RING_AARCH64_SRCS.iter())
                .chain(RING_ARM_SHARED_SRCS.iter())
                .chain(RING_ARM_SRCS.iter())
                .chain(RING_INTEL_SHARED_SRCS.iter())
                .chain(RING_TEST_SRCS.iter())
                .chain(RING_X86_64_SRC.iter())
                .chain(RING_X86_SRCS.iter())
                .chain(RING_PPC_SRCS.iter())
                .any(cmp)
        },
        Some("S") |
        Some("asm") => {
            RING_AARCH64_SRCS.iter()
                .chain(RING_ARM_SHARED_SRCS.iter())
                .chain(RING_ARM_SRCS.iter())
                .chain(RING_INTEL_SHARED_SRCS.iter())
                .chain(RING_X86_64_SRC.iter())
                .chain(RING_X86_SRCS.iter())
                .any(cmp)
        },
        Some("pl") => {
            RING_AARCH64_SRCS.iter()
                .chain(RING_ARM_SHARED_SRCS.iter())
                .chain(RING_ARM_SRCS.iter())
                .chain(RING_INTEL_SHARED_SRCS.iter())
                .chain(RING_X86_64_SRC.iter())
                .chain(RING_X86_SRCS.iter())
                .chain(RING_PPC_SRCS.iter())
                .chain(RING_PERL_INCLUDES.iter())
                .any(cmp)
        },
        _ => true,
    };
    if !tracked {
        panic!("{:?} is not tracked in build.rs", p);
    }
}

fn walk_dir<F>(dir: &Path, cb: &F)
    where F: Fn(&DirEntry)
{
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                walk_dir(&path, cb);
            } else {
                cb(&entry);
            }
        }
    }
}

fn print_rerun() {
    for s in RING_ARM_SHARED_SRCS.iter()
        .chain(RING_SRC.iter())
        .chain(RING_TEST_SRCS.iter())
        .chain(RING_AARCH64_SRCS.iter())
        .chain(RING_ARM_SRCS.iter())
        .chain(RING_X86_64_SRC.iter())
        .chain(RING_X86_SRCS.iter())
        .chain(RING_INTEL_SHARED_SRCS.iter())
        .chain(RING_PPC_SRCS.iter())
        .chain(RING_HEADERS.iter())
        .chain(RING_TEST_HEADERS.iter())
        .chain(RING_PERL_INCLUDES.iter())
        .chain(RING_INLINE_FILES.iter()) {
        println!("cargo:rerun-if-changed={}", s);
    }
    println!("cargo:rerun-if-changed=build.rs");
}
