// Copyright 2018-2022 the Deno authors. All rights reserved. MIT license.

use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;
use std::env;
use std::path::Path;
use std::path::PathBuf;

// TODO(bartlomieju): this module contains a lot of duplicated
// logic with `cli/build.rs`, factor out to `deno_core`.
fn create_snapshot(mut js_runtime: JsRuntime, snapshot_path: &Path, files: Vec<PathBuf>) {
    // TODO(nayeemrmn): https://github.com/rust-lang/cargo/issues/3946 to get the
    // workspace root.
    let display_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    for file in files {
        println!("cargo:rerun-if-changed={}", file.display());
        let display_path = file.strip_prefix(display_root).unwrap();
        let display_path_str = display_path.display().to_string();
        js_runtime
            .execute_script(
                &("deno:".to_string() + &display_path_str.replace('\\', "/")),
                &std::fs::read_to_string(&file).unwrap(),
            )
            .unwrap();
    }

    let snapshot = js_runtime.snapshot();
    let snapshot_slice: &[u8] = &*snapshot;
    println!("Snapshot size: {}", snapshot_slice.len());

    let compressed_snapshot_with_size = {
        let mut vec = vec![];

        vec.extend_from_slice(
            &u32::try_from(snapshot.len())
                .expect("snapshot larger than 4gb")
                .to_le_bytes(),
        );

        lzzzz::lz4_hc::compress_to_vec(snapshot_slice, &mut vec, lzzzz::lz4_hc::CLEVEL_MAX)
            .expect("snapshot compression failed");

        vec
    };

    println!(
        "Snapshot compressed size: {}",
        compressed_snapshot_with_size.len()
    );

    std::fs::write(&snapshot_path, compressed_snapshot_with_size).unwrap();
    println!("Snapshot written to: {} ", snapshot_path.display());
}

struct Permissions;

impl deno_fetch::FetchPermissions for Permissions {
    fn check_net_url(
        &mut self,
        _url: &deno_core::url::Url,
    ) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }

    fn check_read(&mut self, _p: &Path) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }
}

impl deno_websocket::WebSocketPermissions for Permissions {
    fn check_net_url(
        &mut self,
        _url: &deno_core::url::Url,
    ) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }
}

impl deno_web::TimersPermission for Permissions {
    fn allow_hrtime(&mut self) -> bool {
        unreachable!("snapshotting!")
    }

    fn check_unstable(&self, _state: &deno_core::OpState, _api_name: &'static str) {
        unreachable!("snapshotting!")
    }
}

impl deno_net::NetPermissions for Permissions {
    fn check_net<T: AsRef<str>>(
        &mut self,
        _host: &(T, Option<u16>),
    ) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }

    fn check_read(&mut self, _p: &Path) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }

    fn check_write(&mut self, _p: &Path) -> Result<(), deno_core::error::AnyError> {
        unreachable!("snapshotting!")
    }
}

fn create_runtime_snapshot(snapshot_path: &Path, files: Vec<PathBuf>) {
    let extensions: Vec<Extension> = vec![
        deno_webidl::init(),
        deno_console::init(),
        deno_url::init(),
        deno_tls::init(),
        deno_web::init::<Permissions>(deno_web::BlobStore::default(), Default::default()),
        deno_fetch::init::<Permissions>(Default::default()),
        deno_websocket::init::<Permissions>("".to_owned(), None, None),
        deno_webstorage::init(None),
        deno_crypto::init(None),
        deno_broadcast_channel::init(
            deno_broadcast_channel::InMemoryBroadcastChannel::default(),
            false, // No --unstable.
        ),
        deno_net::init::<Permissions>(
            None, false, // No --unstable.
            None,
        ),
        deno_http::init(),
    ];

    let js_runtime = JsRuntime::new(RuntimeOptions {
        will_snapshot: true,
        extensions,
        ..Default::default()
    });
    create_snapshot(js_runtime, snapshot_path, files);
}

fn get_js_files(d: &str) -> Vec<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut js_files = std::fs::read_dir(d)
        .unwrap()
        .map(|dir_entry| {
            let file = dir_entry.unwrap();
            manifest_dir.join(file.path())
        })
        .filter(|path| path.extension().unwrap_or_default() == "js")
        .collect::<Vec<PathBuf>>();
    js_files.sort();
    js_files
}

pub fn build_snapshot(runtime_snapshot_path: PathBuf) {
    let js_files = get_js_files("js");
    create_runtime_snapshot(&runtime_snapshot_path, js_files);
}

fn git_commit_hash() -> String {
    if let Ok(output) = std::process::Command::new("git")
        .arg("rev-list")
        .arg("-1")
        .arg("HEAD")
        .output()
    {
        if output.status.success() {
            std::str::from_utf8(&output.stdout[..40])
                .unwrap()
                .to_string()
        } else {
            // When not in git repository
            // (e.g. when the user install by `cargo install deno`)
            "UNKNOWN".to_string()
        }
    } else {
        // When there is no git command for some reason
        "UNKNOWN".to_string()
    }
}

fn main() {
    // To debug snapshot issues uncomment:
    // op_fetch_asset::trace_serializer();

    println!("cargo:rustc-env=TARGET={}", env::var("TARGET").unwrap());
    println!("cargo:rustc-env=PROFILE={}", env::var("PROFILE").unwrap());
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_commit_hash());
    println!("cargo:rerun-if-env-changed=GIT_COMMIT_HASH");
    println!("cargo:rustc-env=TS_VERSION=4.3.0");
    println!("cargo:rerun-if-env-changed=TS_VERSION");

    let o = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // Main snapshot
    let runtime_snapshot_path = o.join("CLI_SNAPSHOT.bin");

    build_snapshot(runtime_snapshot_path)
}
