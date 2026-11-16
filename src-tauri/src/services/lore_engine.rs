// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::models::{LoreAnomalyItem, LoreAnomalyReport};
use rusqlite::params;
use std::collections::HashMap;

pub const VECTOR_DIM: usize = 256;

fn floor_char_boundary(s: &str, mut index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    while index > 0 && !s.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn ceil_char_boundary(s: &str, mut index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    while index < s.len() && !s.is_char_boundary(index) {
        index += 1;
    }
    index
}

fn has_leading_negation(context_window: &str, needle_pos: usize) -> bool {
    let prefix_start = floor_char_boundary(context_window, needle_pos.saturating_sub(24));
    let prefix = &context_window[prefix_start..needle_pos];
    let negations = [
        "not ",
        "never ",
        "no longer ",
        "не ",
        "никогда ",
        "не является ",
        "nicht ",
        "niemals ",
        "kein ",
        "keine ",
        "no ",
        "nunca ",
        "jamás ",
    ];
    negations
        .iter()
        .any(|neg| prefix.ends_with(neg) || prefix.contains(neg))
}

pub fn is_codex_category(category: &str) -> bool {
    let lower = category.to_lowercase();
    let markers = [
        "character",
        "artifact",
        "relic",
        "demiurge",
        "entity",
        "sanctuary",
        "location",
        "персонаж",
        "артефакт",
        "реликвия",
        "демиург",
        "сущность",
        "святилище",
        "локация",
        "charakter",
        "artefakt",
        "relikt",
        "entität",
        "heiligtum",
        "ort",
        "personaje",
        "artefacto",
        "reliquia",
        "entidad",
        "santuario",
        "ubicación",
    ];
    markers.iter().any(|m| lower.contains(m))
}

pub fn is_divine_category(category: &str) -> bool {
    let lower = category.to_lowercase();
    let markers = [
        "artifact",
        "relic",
        "demiurge",
        "артефакт",
        "реликвия",
        "демиург",
        "artefakt",
        "relikt",
        "artefacto",
        "reliquia",
    ];
    markers.iter().any(|m| lower.contains(m))
}

fn is_enemy_relation(relation_type: &str) -> bool {
    let lower = relation_type.to_lowercase();
    lower.contains("вражд")
        || lower.contains("enemy")
        || lower.contains("rivalry")
        || lower.contains("rival")
        || lower.contains("conflict")
        || lower.contains("конфликт")
        || lower.contains("konflikt")
}

pub fn compute_embedding(text: &str) -> [f32; VECTOR_DIM] {
    let mut vec = [0.0f32; VECTOR_DIM];
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return vec;
    }

    for word in &words {
        let clean: String = word
            .chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect();

        if clean.is_empty() {
            continue;
        }

        let mut h = 2166136261u32;
        for b in clean.bytes() {
            h ^= b as u32;
            h = h.wrapping_mul(16777619);
        }
        let idx = (h as usize) % VECTOR_DIM;
        vec[idx] += 1.0;

        let char_vec: Vec<char> = clean.chars().collect();
        if char_vec.len() >= 3 {
            for window in char_vec.windows(3) {
                let mut tri_h = 2166136261u32;
                for ch in window {
                    tri_h ^= *ch as u32;
                    tri_h = tri_h.wrapping_mul(16777619);
                }
                let tri_idx = (tri_h as usize) % VECTOR_DIM;
                vec[tri_idx] += 0.4;
            }
        }
    }

    let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in vec.iter_mut() {
            *v /= norm;
        }
    }
    vec
}

pub fn cosine_similarity(a: &[f32; VECTOR_DIM], b: &[f32; VECTOR_DIM]) -> f32 {
    let mut dot = 0.0f32;
    for i in 0..VECTOR_DIM {
        dot += a[i] * b[i];
    }
    dot
}

pub fn vector_to_bytes(v: &[f32; VECTOR_DIM]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(VECTOR_DIM * 4);
    for num in v {
        bytes.extend_from_slice(&num.to_le_bytes());
    }
    bytes
}

pub fn bytes_to_vector(b: &[u8]) -> Option<[f32; VECTOR_DIM]> {
    if b.len() != VECTOR_DIM * 4 {
        return None;
    }
    let mut v = [0.0f32; VECTOR_DIM];
    for (i, chunk) in b.chunks_exact(4).enumerate() {
        let arr = [chunk[0], chunk[1], chunk[2], chunk[3]];
        v[i] = f32::from_le_bytes(arr);
    }
    Some(v)
}

pub fn sync_story_embedding(
    conn: &rusqlite::Connection,
    story_id: &str,
    content: &str,
) -> Result<(), rusqlite::Error> {
    let embedding = compute_embedding(content);
    let bytes = vector_to_bytes(&embedding);
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT INTO lore_embeddings (story_id, vector, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(story_id) DO UPDATE SET
            vector = excluded.vector,
            updated_at = excluded.updated_at",
        params![story_id, bytes, now],
    )?;
    Ok(())
}

struct OntologicalRule {
    text_needles: &'static [&'static str],
    lore_needles: &'static [&'static str],
    severity: &'static str,
    conflict_en: &'static str,
    conflict_ru: &'static str,
    conflict_de: &'static str,
    conflict_es: &'static str,
    fix_en: &'static str,
    fix_ru: &'static str,
    fix_de: &'static str,
    fix_es: &'static str,
}

const ONTOLOGICAL_RULES: &[OntologicalRule] = &[
    OntologicalRule {
        text_needles: &["mortal", "смертен", "sterblich"],
        lore_needles: &["immortal", "бессмерт", "unsterblich", "inmortal"],
        severity: "CRITICAL",
        conflict_en: "Ontological immortality paradox",
        conflict_ru: "Онтологический парадокс бессмертия",
        conflict_de: "Ontologisches Unsterblichkeits-Paradoxon",
        conflict_es: "Paradoja ontológica de inmortalidad",
        fix_en: "Remove mortality claim or establish loss of divine status",
        fix_ru: "Устранить упоминание смертности или зафиксировать утрату божественного статуса",
        fix_de: "Sterblichkeitshinweis entfernen oder Verlust des göttlichen Status erklären",
        fix_es: "Eliminar mención mortal o justificar la pérdida de inmortalidad",
    },
    OntologicalRule {
        text_needles: &["died", "perished", "killed", "погиб", "умер", "starb", "getötet", "murió"],
        lore_needles: &["alive", "жив", "жива", "lebendig", "vivo", "viva"],
        severity: "CRITICAL",
        conflict_en: "Chronological mortality contradiction",
        conflict_ru: "Хронологическое нарушение гибели",
        conflict_de: "Chronologischer Todeswiderspruch",
        conflict_es: "Contradicción cronológica de fallecimiento",
        fix_en: "Entity is recorded as alive in canon. Verify timeline order",
        fix_ru: "Персонаж зафиксирован как живой в каноне. Проверьте хронологию сцены",
        fix_de: "Entität gilt im Kanon als lebendig. Chronologie prüfen",
        fix_es: "La entidad figura como viva en el canon. Verifica el orden temporal",
    },
    OntologicalRule {
        text_needles: &["destroyed", "shattered", "уничтожен", "разрушен", "zerstört", "destruido"],
        lore_needles: &["indestructible", "invulnerable", "неуязвим", "неразрушим", "unzerstörbar", "indestructible"],
        severity: "CRITICAL",
        conflict_en: "Relic integrity violation",
        conflict_ru: "Нарушение целостности реликвии",
        conflict_de: "Verletzung der Relikt-Integrität",
        conflict_es: "Violación de integridad de la reliquia",
        fix_en: "Artifact is recorded as indestructible in canon",
        fix_ru: "Артефакт зафиксирован в каноне как неуязвимый",
        fix_de: "Artefakt ist im Kanon als unzerstörbar definiert",
        fix_es: "El artefacto consta como indestructible en el canon",
    },
    OntologicalRule {
        text_needles: &["traitor", "betrayed", "предатель", "предал", "verräter", "traidor"],
        lore_needles: &["loyal", "faithful", "верный", "предан", "loyal", "treu", "leal"],
        severity: "MAJOR",
        conflict_en: "Allegiance motivation mismatch",
        conflict_ru: "Искажение мотивов альянса",
        conflict_de: "Widerspruch zur Loyalitätsmotivation",
        conflict_es: "Discordancia de lealtad y alianza",
        fix_en: "Canon records strong allegiance to this faction",
        fix_ru: "В каноне зафиксирована верность фракции",
        fix_de: "Im Kanon ist Treue zu dieser Fraktion vermerkt",
        fix_es: "El canon registra lealtad firme hacia la facción",
    },
    OntologicalRule {
        text_needles: &["ordinary human", "человек", "gewöhnlicher mensch", "humano común"],
        lore_needles: &["demiurge", "демиург", "architekt", "arquitecto"],
        severity: "MAJOR",
        conflict_en: "Divine rank distortion",
        conflict_ru: "Искажение божественного ранга",
        conflict_de: "Verzerrung des göttlichen Ranges",
        conflict_es: "Distorsión de rango divino",
        fix_en: "Entity holds Demiurge rank in canon",
        fix_ru: "Сущность обладает рангом Демиурга в каноне",
        fix_de: "Entität besitzt im Kanon den Rang eines Demiurgen",
        fix_es: "La entidad posee rango de Demiurgo en el canon",
    },
    OntologicalRule {
        text_needles: &["powerless", "helpless", "бессилен", "machtlos", "impotente"],
        lore_needles: &["omnipotent", "all-powerful", "всемогущ", "allmächtig", "omnipotente"],
        severity: "MINOR",
        conflict_en: "Power ceiling contradiction",
        conflict_ru: "Конфликт пределов могущества",
        conflict_de: "Widerspruch der Machtgrenzen",
        conflict_es: "Conflicto de límites de poder",
        fix_en: "Clarify the source suppressing the entity's abilities",
        fix_ru: "Уточните источник подавления сил сущности",
        fix_de: "Quelle der Machtunterdrückung im Text erläutern",
        fix_es: "Aclara el origen que suprime las habilidades",
    },
];

fn localized_claim_in_text(locale: &str, needle: &str) -> String {
    match locale {
        "ru" => format!("В сцене: «{}»", needle),
        "de" => format!("Im Text: „{}“", needle),
        "es" => format!("En escena: «{}»", needle),
        _ => format!("In scene: \"{}\"", needle),
    }
}

fn localized_lore_fact(locale: &str, category: &str, needle: &str) -> String {
    match locale {
        "ru" => format!("В каноне [{}] «{}»", category, needle),
        "de" => format!("Im Kanon [{}] „{}“", category, needle),
        "es" => format!("En canon [{}] «{}»", category, needle),
        _ => format!("In canon [{}] \"{}\"", category, needle),
    }
}

fn localized_rule_text(rule: &OntologicalRule, locale: &str) -> (&'static str, &'static str) {
    match locale {
        "ru" => (rule.conflict_ru, rule.fix_ru),
        "de" => (rule.conflict_de, rule.fix_de),
        "es" => (rule.conflict_es, rule.fix_es),
        _ => (rule.conflict_en, rule.fix_en),
    }
}

fn localized_rivalry_anomaly(locale: &str, title: &str, target: &str) -> LoreAnomalyItem {
    let (conflict_type, claim_in_text, original_lore_fact, suggested_fix) = match locale {
        "ru" => (
            "Нарушение статуса вражды",
            "В тексте персонажи выступают в мирном альянсе",
            "В графе зафиксирована «Вражда»",
            "Обоснуйте перемирие или измените связь в Кодексе",
        ),
        "de" => (
            "Verletzung des Rivalitätsstatus",
            "Charaktere handeln im Text als friedliche Verbündete",
            "Im Beziehungsgraph ist Rivalität gespeichert",
            "Waffenstillstand begründen oder Relation im Kodex aktualisieren",
        ),
        "es" => (
            "Violación de estado de rivalidad",
            "Los personajes actúan como aliados pacíficos en la escena",
            "El grafo registra una relación de rivalidad",
            "Justifica la tregua o actualiza el vínculo en el Códice",
        ),
        _ => (
            "Rivalry status violation",
            "Characters act in peaceful alliance in the scene",
            "Relation graph records an active rivalry",
            "Justify the truce or update the link in the Codex",
        ),
    };

    LoreAnomalyItem {
        id: format!("rivalry::{}::{}", title, target),
        entity_title: format!("{} ↔ {}", title, target),
        conflict_type: conflict_type.to_string(),
        claim_in_text: claim_in_text.to_string(),
        original_lore_fact: original_lore_fact.to_string(),
        severity: "MAJOR".to_string(),
        suggested_fix: suggested_fix.to_string(),
    }
}

fn ally_context_present(lower_text: &str) -> bool {
    let markers = [
        "ally",
        "friend",
        "together",
        "truce",
        "союзник",
        "друг",
        "вместе",
        "verbündet",
        "freund",
        "aliado",
        "amigo",
    ];
    markers.iter().any(|m| lower_text.contains(m))
}

pub fn evaluate_lore_consistency(
    lore_entities: &[(String, String, String, String)],
    relations_map: &HashMap<String, Vec<(String, String)>>,
    active_content: &str,
    mode: &str,
    locale: &str,
) -> LoreAnomalyReport {
    let mut anomalies = Vec::new();
    let mut verified_count = 0usize;
    let lower_text = active_content.to_lowercase();

    for (id, title, category, lore_text) in lore_entities {
        let title_lower = title.to_lowercase();
        if title_lower.is_empty() || !lower_text.contains(&title_lower) {
            continue;
        }

        verified_count += 1;
        let lore_lower = lore_text.to_lowercase();

        if let Some(p) = lower_text.find(&title_lower) {
            let window_start = floor_char_boundary(&lower_text, p.saturating_sub(180));
            let window_end =
                ceil_char_boundary(&lower_text, (p + title_lower.len() + 180).min(lower_text.len()));
            let context_window = &lower_text[window_start..window_end];

            for (rule_idx, rule) in ONTOLOGICAL_RULES.iter().enumerate() {
                if mode == "PERMISSIVE" && rule.severity != "CRITICAL" {
                    continue;
                }
                if mode == "BALANCED" && rule.severity == "MINOR" {
                    continue;
                }

                let matched_text = rule
                    .text_needles
                    .iter()
                    .find_map(|n| context_window.find(n).map(|pos| (*n, pos)));
                let matched_lore = rule.lore_needles.iter().find(|n| lore_lower.contains(**n));

                if let (Some((text_needle, needle_pos)), Some(lore_needle)) =
                    (matched_text, matched_lore)
                {
                    if has_leading_negation(context_window, needle_pos) {
                        continue;
                    }
                    let (conflict_desc, fix) = localized_rule_text(rule, locale);
                    anomalies.push(LoreAnomalyItem {
                        id: format!("rule::{}::{}::{}", id, rule_idx, text_needle),
                        entity_title: title.clone(),
                        conflict_type: conflict_desc.to_string(),
                        claim_in_text: localized_claim_in_text(locale, text_needle),
                        original_lore_fact: localized_lore_fact(locale, category, lore_needle),
                        severity: rule.severity.to_string(),
                        suggested_fix: fix.to_string(),
                    });
                }
            }
        }

        if mode != "PERMISSIVE" {
            if let Some(rels) = relations_map.get(id) {
                for (target_title, rel_type) in rels {
                    let tgt_lower = target_title.to_lowercase();
                    if !tgt_lower.is_empty() && lower_text.contains(&tgt_lower) {
                        let pos_a = lower_text.find(&title_lower).unwrap_or(0);
                        let pos_b = lower_text.find(&tgt_lower).unwrap_or(0);
                        let dist = (pos_a as isize - pos_b as isize).abs();

                        if dist < 400
                            && is_enemy_relation(rel_type)
                            && ally_context_present(&lower_text)
                        {
                            anomalies.push(localized_rivalry_anomaly(locale, title, target_title));
                        }
                    }
                }
            }
        }
    }

    let penalty = (anomalies.len() as f32 * 14.0).min(100.0);
    let consistency_score = (100.0 - penalty).max(0.0);

    LoreAnomalyReport {
        anomalies,
        verified_claims_count: verified_count,
        consistency_score,
    }
}