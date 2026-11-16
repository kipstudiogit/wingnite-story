// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

use crate::models::TextMetricsResult;
use std::collections::{HashMap, HashSet};

fn locale_cliches(locale: &str) -> &'static [&'static str] {
    match locale {
        "ru" => &[
            "в конечном итоге",
            "имеет место быть",
            "ни для кого не секрет",
            "на сегодняшний день",
            "ледяной ужас",
            "мурашки по спине",
            "холодный пот",
            "волосы встали дыбом",
            "кровь застыла в жилах",
            "в глубине души",
            "луч света в темном",
            "на вес золота",
        ],
        "de" => &[
            "am ende des tages",
            "im großen und ganzen",
            "eiskalter schauer",
            "blut gefror in den adern",
            "im tiefsten innern",
            "wie aus dem nichts",
            "es ist kein geheimnis",
            "auf gold aufwiegen",
        ],
        "es" => &[
            "al fin y al cabo",
            "a fin de cuentas",
            "heló la sangre en las venas",
            "sudor frío",
            "en el fondo de su alma",
            "de la nada",
            "un rayo de esperanza",
            "no es ningún secreto",
        ],
        _ => &[
            "at the end of the day",
            "needless to say",
            "blood ran cold",
            "cold sweat",
            "hair stood on end",
            "deep down inside",
            "out of nowhere",
            "ray of light",
            "worth its weight in gold",
            "in the blink of an eye",
        ],
    }
}

fn format_repetition_label(locale: &str, word: &str, count: usize) -> String {
    match locale {
        "ru" => format!("{}: {} раз", word, count),
        "de" => format!("{}: {}×", word, count),
        "es" => format!("{}: {} veces", word, count),
        _ => format!("{}: {} times", word, count),
    }
}

fn format_cliche_label(locale: &str, cliche: &str) -> String {
    match locale {
        "ru" => format!("Штамп: «{}»", cliche),
        "de" => format!("Klischee: „{}“", cliche),
        "es" => format!("Cliché: «{}»", cliche),
        _ => format!("Cliché: \"{}\"", cliche),
    }
}

fn compute_flesch_by_locale(locale: &str, asl: f64, asw: f64) -> f64 {
    let raw = match locale {
        "ru" => 206.835 - (1.3 * asl) - (60.1 * asw),
        "de" => 180.0 - asl - (58.5 * asw),
        "es" => 206.84 - (1.02 * asl) - (0.60 * (asw * 100.0)),
        _ => 206.835 - (1.015 * asl) - (84.6 * asw),
    };
    raw.clamp(0.0, 100.0)
}

pub fn analyze_text(text: &str, locale: &str) -> TextMetricsResult {
    let char_count = text.chars().count();
    let all_words: Vec<&str> = text.split_whitespace().collect();
    let word_count = all_words.len();
    let mut syllable_count = 0usize;
    let mut word_freq: HashMap<String, usize> = HashMap::new();
    let mut unique_words = HashSet::new();

    let vowels: HashSet<char> = "aeiouyäöüáéíóúаеёиоуыэюя".chars().collect();
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let paragraph_count = lines.len().max(1);

    for word in &all_words {
        let normalized: String = word
            .chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect();

        if normalized.chars().count() > 2 {
            unique_words.insert(normalized.clone());
            *word_freq.entry(normalized.clone()).or_insert(0) += 1;
        }

        let mut word_syllables = 0usize;
        for ch in normalized.chars() {
            if vowels.contains(&ch) {
                word_syllables += 1;
            }
        }
        if !normalized.is_empty() {
            syllable_count += word_syllables.max(1);
        }
    }

    let reading_time_minutes = if word_count == 0 {
        0.0
    } else {
        (word_count as f64 / 190.0).max(0.1)
    };

    let mut in_quote = false;
    let mut dialogue_chars = 0usize;
    for line in &lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with('—') || trimmed.starts_with("- ") {
            dialogue_chars += trimmed.chars().count();
            continue;
        }
        for c in line.chars() {
            if c == '«' || c == '“' || c == '„' || (!in_quote && c == '"') {
                in_quote = true;
            } else if c == '»' || c == '”' || (in_quote && c == '"') {
                in_quote = false;
            } else if in_quote {
                dialogue_chars += 1;
            }
        }
    }

    let dialogue_ratio = if char_count > 0 {
        ((dialogue_chars as f64 / char_count as f64) * 100.0).min(100.0)
    } else {
        0.0
    };

    let mut sentence_word_counts = Vec::new();
    let mut current_sentence_words = 0usize;

    for token in &all_words {
        current_sentence_words += 1;
        if token.ends_with('.') || token.ends_with('!') || token.ends_with('?') || token.ends_with('…') {
            sentence_word_counts.push(current_sentence_words as f64);
            current_sentence_words = 0;
        }
    }
    if current_sentence_words > 0 {
        sentence_word_counts.push(current_sentence_words as f64);
    }

    let sentence_count = sentence_word_counts.len().max(1);
    let sentence_cadence = if !sentence_word_counts.is_empty() {
        let mean: f64 = sentence_word_counts.iter().sum::<f64>() / sentence_count as f64;
        let variance: f64 = sentence_word_counts
            .iter()
            .map(|l| (l - mean).powi(2))
            .sum::<f64>()
            / sentence_count as f64;
        variance.sqrt().min(40.0)
    } else {
        0.0
    };

    let asl = word_count as f64 / sentence_count as f64;
    let asw = if word_count > 0 {
        syllable_count as f64 / word_count as f64
    } else {
        1.0
    };
    let flesch_reading_ease = compute_flesch_by_locale(locale, asl, asw);

    let vocabulary_richness = if word_count > 0 {
        ((unique_words.len() as f64 / word_count as f64) * 100.0).min(100.0)
    } else {
        0.0
    };

    let mut freq_vec: Vec<(String, usize)> = word_freq.into_iter().collect();
    freq_vec.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let repetition_warnings = freq_vec
        .into_iter()
        .filter(|(_, count)| *count >= 4)
        .take(6)
        .map(|(word, count)| format_repetition_label(locale, &word, count))
        .collect();

    let lower_text = text.to_lowercase();
    let mut cliche_warnings = Vec::new();
    for cliche in locale_cliches(locale) {
        if lower_text.contains(cliche) {
            cliche_warnings.push(format_cliche_label(locale, cliche));
        }
    }

    let segments = 8;
    let mut pacing_timeline = Vec::with_capacity(segments);
    if word_count == 0 {
        pacing_timeline.resize(segments, 0.0);
    } else {
        for seg_idx in 0..segments {
            let start = (seg_idx * word_count) / segments;
            let end = ((seg_idx + 1) * word_count) / segments;
            let slice = &all_words[start..end.max(start)];
            let mut exclamations = 0usize;
            let mut quote_marks = 0usize;
            for w in slice {
                if w.contains('!') || w.contains('?') {
                    exclamations += 1;
                }
                if w.contains('«') || w.contains('"') || w.contains('“') || *w == "—" {
                    quote_marks += 1;
                }
            }
            let density = slice.len() as f64 * 1.4
                + exclamations as f64 * 12.0
                + quote_marks as f64 * 10.0;
            pacing_timeline.push(density.clamp(12.0, 100.0));
        }
    }

    let sentiment_tone_key = if word_count == 0 {
        "neutral"
    } else if dialogue_ratio > 45.0 {
        "dialogue"
    } else if sentence_cadence > 6.5 {
        "action"
    } else if flesch_reading_ease > 65.0 {
        "prose"
    } else {
        "epic"
    }
    .to_string();

    TextMetricsResult {
        word_count,
        char_count,
        paragraph_count,
        reading_time_minutes,
        dialogue_ratio,
        sentence_cadence,
        vocabulary_richness,
        flesch_reading_ease,
        sentiment_tone_key,
        repetition_warnings,
        cliche_warnings,
        pacing_timeline,
    }
}