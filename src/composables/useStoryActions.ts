import { invoke } from "@tauri-apps/api/core";
import type { EnterpriseStoryItem } from "../types";
import { useHaptics } from "./useHaptics";
import { useDevice } from "./useDevice";

export function useStoryActions(
  activeProjectIdGetter: () => string,
  projectNameGetter: () => string,
  exportProjectWstory: (id: string) => Promise<string>,
  importProjectWstory: (raw: string) => Promise<any>
) {
  const { vibrateSuccess, vibrateWarning } = useHaptics();
  const { shareOrExportFile } = useDevice();

  async function exportWstory(): Promise<string> {
    try {
      const pid = activeProjectIdGetter();
      const raw = await exportProjectWstory(pid);
      const cleanTitle = (projectNameGetter() || "Project").replace(/[^a-zA-Zа-яА-ЯёЁ0-9_-]/g, "_");
      const filename = `${cleanTitle}.wstory`;
      const status = await shareOrExportFile(filename, raw, "application/octet-stream");
      vibrateSuccess();
      return status;
    } catch (err) {
      vibrateWarning();
      throw err;
    }
  }

  async function exportTxtBook(): Promise<string> {
    try {
      const pid = activeProjectIdGetter();
      const txt = await invoke<string>("export_manuscript_txt", { projectId: pid });
      const status = await shareOrExportFile("Wingnite_Book.txt", txt, "text/plain");
      vibrateSuccess();
      return status;
    } catch (err) {
      vibrateWarning();
      throw err;
    }
  }

  async function exportMarkdownBook(): Promise<string> {
    try {
      const pid = activeProjectIdGetter();
      const md = await invoke<string>("export_manuscript", { projectId: pid });
      const status = await shareOrExportFile("Wingnite_Manuscript.md", md, "text/markdown");
      vibrateSuccess();
      return status;
    } catch (err) {
      vibrateWarning();
      throw err;
    }
  }

  async function exportHtmlBook(): Promise<string> {
    try {
      const pid = activeProjectIdGetter();
      const html = await invoke<string>("compile_book_edition", { projectId: pid });
      const status = await shareOrExportFile("Wingnite_Book_Edition.html", html, "text/html");
      vibrateSuccess();
      return status;
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
      });
      vibrateSuccess();
      return imported;
    } catch (err) {
      vibrateWarning();
      throw err;
    }
  }

  async function importWstoryFile(file: File): Promise<any> {
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