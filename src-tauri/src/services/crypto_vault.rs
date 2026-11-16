// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::models::{EnterpriseStoryItem, ProjectItem};
use sha2::{Digest, Sha256};

pub struct VaultService;

impl VaultService {
    pub fn calculate_hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn hash_passphrase(salt: &str, passphrase: &str) -> String {
        let mut current = Self::calculate_hash(&format!("{}:{}", salt, passphrase));
        for _ in 0..4096 {
            current = Self::calculate_hash(&format!("{}:{}", current, salt));
        }
        current
    }

    pub fn verify_passphrase(salt: &str, passphrase: &str, expected_hash: &str) -> bool {
        if expected_hash.is_empty() {
            return passphrase.is_empty();
        }
        let candidate = Self::hash_passphrase(salt, passphrase);
        let a = candidate.as_bytes();
        let b = expected_hash.as_bytes();
        if a.len() != b.len() {
            return false;
        }
        let mut diff = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            diff |= x ^ y;
        }
        diff == 0
    }

    pub fn compute_merkle_root(mut hashes: Vec<String>) -> String {
        if hashes.is_empty() {
            return Self::calculate_hash("genesis_empty_vault");
        }
        hashes.sort();
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    format!("{}{}", chunk[0], chunk[1])
                } else {
                    format!("{}{}", chunk[0], chunk[0])
                };
                next_level.push(Self::calculate_hash(&combined));
            }
            hashes = next_level;
        }
        hashes.remove(0)
    }

    pub fn compute_wstory_checksum(project: &ProjectItem, stories: &[EnterpriseStoryItem]) -> String {
        let mut hash_acc = String::new();
        hash_acc.push_str(&project.id);
        hash_acc.push_str(&project.title);
        for s in stories {
            hash_acc.push_str(&s.content_hash);
        }
        Self::calculate_hash(&hash_acc)
    }
}