// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

import { invoke } from "@tauri-apps/api/core";
import type { EnterpriseStoryItem, ProjectItem } from "../types";
import { useHaptics } from "./useHaptics";
import { useDevice } from "./useDevice";
import { useI18n } from "../i18n";

export function useStoryActions(
  activeProjectIdGetter: () => string,
  projectNameGetter: () => string,
  exportProjectWstory: (id: string) => Promise<string>,
  importProjectWstory: (raw: string) => Promise<ProjectItem>
) {
  const { vibrateSuccess, vibrateWarning, vibrateLight } = useHaptics();
  const { shareOrExportFile } = useDevice();
  const { locale } = useI18n();

  function buildSafeSlug(): string {
    const base = (projectNameGetter() || "Project").replace(/[^a-zA-Zа-яА-ЯёЁäöüÄÖÜßñÑ0-9_-]/g, "_");
    return base || "Project";
  }

  async function runSaveFlow(
    filename: string,
    content: string,
    mimeType: string
  ): Promise<string> {
    const result = await shareOrExportFile(filename, content, mimeType);
    if (result.saved) {
      vibrateSuccess();
    } else {
      vibrateLight();
    }
    return result.message;
  }

  async function exportWstory(): Promise<string> {
    try {
      const pid = activeProjectIdGetter();
      const raw = await exportProjectWstory(pid);
      return await runSaveFlow(`${buildSafeSlug()}.wstory`, raw, "application/octet-stream");
    } catch (err) {
      vibrateWarning();
      throw err;
    }
  }

  async function exportTxtBook(): Promise<string> {
    try {
      const pid = activeProjectIdGetter();
      const txt = await invoke<string>("export_manuscript_txt", { projectId: pid });
      return await runSaveFlow(`${buildSafeSlug()}.txt`, txt, "text/plain");
    } catch (err) {
      vibrateWarning();
      throw err;
    }
  }

  async function exportMarkdownBook(): Promise<string> {
    try {
      const pid = activeProjectIdGetter();
      const md = await invoke<string>("export_manuscript", {
        projectId: pid,
        locale: locale.value,
      });
      return await runSaveFlow(`${buildSafeSlug()}_Manuscript.md`, md, "text/markdown");
    } catch (err) {
      vibrateWarning();
      throw err;
    }
  }

  async function exportHtmlBook(): Promise<string> {
    try {
      const pid = activeProjectIdGetter();
      const html = await invoke<string>("compile_book_edition", {
        projectId: pid,
        locale: locale.value,
      });
      return await runSaveFlow(`${buildSafeSlug()}_Edition.html`, html, "text/html");
    } catch (err) {
      vibrateWarning();
      throw err;
    }
  }

  async function importTxtFile(file: File): Promise<EnterpriseStoryItem[]> {
    try {
      const text = await file.text();
      const cleanTitle = file.name.replace(/\.[^/.]+$/, "");
      const pid = activeProjectIdGetter();
      const imported = await invoke<EnterpriseStoryItem[]>("import_plain_text_chapters", {
        projectId: pid,
        rawText: text,
        defaultTitle: cleanTitle,
        locale: locale.value,
      });
      vibrateSuccess();
      return imported;
    } catch (err) {
      vibrateWarning();
      throw err;
    }
  }

  async function importWstoryFile(file: File): Promise<ProjectItem> {
    try {
      const text = await file.text();
      const imported = await importProjectWstory(text);
      vibrateSuccess();
      return imported;
    } catch (err) {
      vibrateWarning();
      throw err;
    }
  }

  return {
    exportWstory,
    exportTxtBook,
    exportMarkdownBook,
    exportHtmlBook,
    importTxtFile,
    importWstoryFile,
  };
}