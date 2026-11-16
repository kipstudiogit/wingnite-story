// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { ref, computed, watch } from "vue";
import type { LocaleCode, LocaleMeta, TranslationDictionary } from "./types";
import { enLocale } from "./locales/en";
import { ruLocale } from "./locales/ru";
import { deLocale } from "./locales/de";
import { esLocale } from "./locales/es";

export type { LocaleCode, LocaleMeta, TranslationDictionary } from "./types";

const LOCALE_STORAGE_KEY = "wingnite_locale_v140";

const dictionaries: Record<LocaleCode, TranslationDictionary> = {
  en: enLocale,
  ru: ruLocale,
  de: deLocale,
  es: esLocale,
};

function resolveInitialLocale(): LocaleCode {
  try {
    const stored = localStorage.getItem(LOCALE_STORAGE_KEY) as LocaleCode | null;
    if (stored && stored in dictionaries) {
      return stored;
    }
  } catch {
  }

  if (typeof navigator !== "undefined" && navigator.language) {
    const base = navigator.language.slice(0, 2).toLowerCase() as LocaleCode;
    if (base in dictionaries) {
      return base;
    }
  }

  return "en";
}

const currentLocale = ref<LocaleCode>(resolveInitialLocale());

function syncDocumentLang(code: LocaleCode) {
  if (typeof document !== "undefined") {
    document.documentElement.lang = code;
  }
}

syncDocumentLang(currentLocale.value);

watch(currentLocale, (next) => {
  syncDocumentLang(next);
  try {
    localStorage.setItem(LOCALE_STORAGE_KEY, next);
  } catch {
  }
});

type Join<K, P> = K extends string
  ? P extends string
    ? `${K}.${P}`
    : never
  : never;

type Prev = [never, 0, 1, 2, 3, 4];

type LeafPaths<T, D extends number = 4> = [D] extends [never]
  ? never
  : T extends string
  ? ""
  : T extends readonly unknown[]
  ? never
  : T extends object
  ? {
      [K in keyof T]-?: K extends string
        ? T[K] extends string
          ? K
          : Join<K, LeafPaths<T[K], Prev[D]>>
        : never;
    }[keyof T]
  : never;

export type TranslationKey = LeafPaths<TranslationDictionary>;

function getNestedValue(obj: Record<string, unknown>, path: string): string {
  const parts = path.split(".");
  let cursor: unknown = obj;
  for (const part of parts) {
    if (cursor && typeof cursor === "object" && part in (cursor as Record<string, unknown>)) {
      cursor = (cursor as Record<string, unknown>)[part];
    } else {
      return path;
    }
  }
  return typeof cursor === "string" ? cursor : path;
}

export function interpolate(template: string, params?: Record<string, string | number>): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (_, key: string) => {
    const val = params[key];
    return val !== undefined ? String(val) : `{${key}}`;
  });
}

export function useI18n() {
  const locale = currentLocale;
  const messages = computed(() => dictionaries[currentLocale.value] || enLocale);
  const availableLocales: LocaleMeta[] = Object.values(dictionaries).map((d) => d.meta);

  function setLocale(code: LocaleCode) {
    if (code in dictionaries) {
      currentLocale.value = code;
    }
  }

  function t(key: TranslationKey, params?: Record<string, string | number>): string {
    const raw = getNestedValue(messages.value as unknown as Record<string, unknown>, key);
    return interpolate(raw, params);
  }

  function formatNumber(value: number): string {
    return new Intl.NumberFormat(currentLocale.value).format(value);
  }

  return {
    locale,
    messages,
    availableLocales,
    setLocale,
    t,
    formatNumber,
  };
}