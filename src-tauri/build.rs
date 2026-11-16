// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

fn main() {
    println!("cargo::rustc-check-cfg=cfg(mobile)");
    tauri_build::build();
}