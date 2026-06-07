#!/usr/bin/env -S cargo +nightly -Zscript
---
[package]
edition = "2021"

[dependencies]
invoke = { path = "../../../../crates/invoke" }
specta = { version = "2.0.0-rc.25", features = ["serde_json"] }
specta-serde = "0.0.12"
specta-typescript = "0.0.12"
---

//! Generates the pack wire-protocol TypeScript from libinvoke's `proto.rs`.
//!
//! libinvoke owns the contract; every runtime (this Bun runtime today, a
//! hypothetical Zig/Python/Rust/... one tomorrow) derives its bindings from here.
//! Run via `bun run gen-proto`:
//!
//!   cargo +nightly -Zscript scripts/proto-types/generate.rs <out.d.ts>

use std::path::PathBuf;

use invoke::pack::proto::{HostHandlers, PackHandlers};

fn main() {
	let outpath = std::env::args()
		.nth(1)
		.map(PathBuf::from)
		.unwrap_or_else(|| PathBuf::from("src/proto.d.ts"));

	let types = specta::Types::default().register::<HostHandlers>().register::<PackHandlers>();

	specta_typescript::Typescript::default()
		.export_to(&outpath, &types, specta_serde::PhasesFormat)
		.expect("export proto types");

	println!("[gen-proto] wrote {}", outpath.display());
}
