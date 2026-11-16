// Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)
// SPDX-License-Identifier: GPL-3.0-only

export type LocaleCode = "en" | "ru" | "de" | "es";

export interface LocaleMeta {
  code: LocaleCode;
  name: string;
  nativeName: string;
}

export interface TranslationDictionary {
  meta: LocaleMeta;
  common: {
    appName: string;
    save: string;
    saving: string;
    saved: string;
    cancel: string;
    confirm: string;
    close: string;
    delete: string;
    create: string;
    edit: string;
    add: string;
    search: string;
    reset: string;
    back: string;
    next: string;
    prev: string;
    loading: string;
    untitled: string;
    empty: string;
    yes: string;
    no: string;
    accept: string;
    dismiss: string;
    importLabel: string;
    exportLabel: string;
  };
  splash: {
    subtitle: string;
    logs: readonly [string, string, string, string, string];
  };
  nav: {
    toc: string;
    projects: string;
    searchPlaceholder: string;
    reader: string;
    studio: string;
    board: string;
    codex: string;
    timeline: string;
    graph: string;
    settings: string;
    sprint: string;
    checkUpdates: string;
    checkingUpdates: string;
    twist: string;
    menu: string;
  };
  projects: {
    title: string;
    empty: string;
    newProject: string;
    createTitle: string;
    editTitle: string;
    createSubtitle: string;
    titlePlaceholder: string;
    descPlaceholder: string;
    createAction: string;
    editAction: string;
    editTooltip: string;
    deleteConfirmTitle: string;
    deleteConfirmMessage: string;
    cannotDeleteLast: string;
    createdToast: string;
    updatedToast: string;
    deletedToast: string;
    importedToast: string;
    defaultTitle: string;
    defaultDescription: string;
  };
  toc: {
    title: string;
    manuscriptProgress: string;
    wordsProgress: string;
    newScene: string;
    empty: string;
    moveUp: string;
    moveDown: string;
    importTxt: string;
    exportTxt: string;
    importWstory: string;
    exportWstory: string;
    buildHtml: string;
    exportMarkdown: string;
  };
  subtoolbar: {
    fontSizeDecrease: string;
    fontSizeIncrease: string;
    themes: {
      obsidian: string;
      celestial: string;
      sepia: string;
      paper: string;
      oled: string;
    };
    modes: {
      split: string;
      edit: string;
      preview: string;
      zen: string;
    };
    quickTxt: string;
    quickMd: string;
    quickHtml: string;
    quickWstory: string;
  };
  statusbar: {
    sqliteWal: string;
    words: string;
    total: string;
    revision: string;
    syncing: string;
    verified: string;
    versionLabel: string;
  };
  notifications: {
    updateTitle: string;
    updateDefaultBody: string;
    installAndRestart: string;
    twistBadge: string;
    insertTwist: string;
    twistPrefix: string;
    sceneSaved: string;
    sprintFinished: string;
    vaultSealedToast: string;
    vaultUnlockedToast: string;
    vaultUnlockError: string;
    snapshotSavedToast: string;
    snapshotRestoredToast: string;
    relationDeletedToast: string;
    exportError: string;
    importError: string;
    importedChapters: string;
    twistError: string;
    exportShared: string;
    exportCanceled: string;
    exportSavedPath: string;
    exportDownloaded: string;
  };
  palette: {
    placeholder: string;
    resultsCount: string;
    hint: string;
    empty: string;
  };
  studio: {
    editorTab: string;
    previewTab: string;
    titlePlaceholder: string;
    categoryPlaceholder: string;
    revBadge: string;
    vaultSealed: string;
    vaultLocked: string;
    vaultUnsealed: string;
    vaultTooltip: string;
    vaultLockedBanner: string;
    loreBadgeTooltip: string;
    loreAlert: string;
    loreInspect: string;
    analyticsTooltip: string;
    loreTooltip: string;
    vaultTooltipBtn: string;
    deleteSceneTooltip: string;
    clearDraftTooltip: string;
    deleteSceneConfirmTitle: string;
    deleteSceneConfirmMessage: string;
    editorPlaceholder: string;
    wordsStat: string;
    charsStat: string;
    canonStat: string;
    ropeLinesStat: string;
    ropeHashStat: string;
    vaultActive: string;
    vaultProtected: string;
    operatorLabel: string;
    defaultSceneTitle: string;
    defaultCategory: string;
    toolbar: {
      h1: string;
      h2: string;
      h3: string;
      bold: string;
      italic: string;
      strike: string;
      highlight: string;
      spoiler: string;
      wiki: string;
      mention: string;
      quote: string;
      checklist: string;
      list: string;
      code: string;
      asterism: string;
      h1Sample: string;
      h2Sample: string;
      h3Sample: string;
      boldSample: string;
      italicSample: string;
      strikeSample: string;
      highlightSample: string;
      spoilerSample: string;
      wikiSample: string;
      mentionSample: string;
      quoteSample: string;
      checklistSample: string;
      listSample: string;
      codeSample: string;
    };
  };
  analytics: {
    title: string;
    wordsAndChars: string;
    paragraphs: string;
    dialogues: string;
    vocabulary: string;
    cadence: string;
    readingTime: string;
    minutesUnit: string;
    fleschTitle: string;
    fleschEasy: string;
    fleschOptimal: string;
    fleschLiterary: string;
    fleschDense: string;
    pacingTitle: string;
    clichesTitle: string;
    repetitionsTitle: string;
    repetitionsEmpty: string;
    tones: {
      dialogue: string;
      action: string;
      prose: string;
      epic: string;
      neutral: string;
    };
  };
  lore: {
    title: string;
    scoreLabel: string;
    checkedPredicates: string;
    noParadoxes: string;
    needsAttention: string;
    filters: {
      all: string;
      critical: string;
      major: string;
      minor: string;
    };
    textPrefix: string;
    canonPrefix: string;
    suggestionLabel: string;
    analyzing: string;
    cleanState: string;
    semanticTitle: string;
    semanticPlaceholder: string;
    searchBtn: string;
    jumpToScene: string;
  };
  vault: {
    title: string;
    tabSecurity: string;
    tabSnapshots: string;
    statusLabel: string;
    sealed: string;
    locked: string;
    open: string;
    entitiesCount: string;
    revisionsSum: string;
    merkleRootLabel: string;
    missing: string;
    passphrasePlaceholder: string;
    lockEditingLabel: string;
    sealing: string;
    sealBtn: string;
    unlockPassphrasePlaceholder: string;
    unlocking: string;
    unlockBtn: string;
    wrongPassphrase: string;
    verifying: string;
    verifyBtn: string;
    validResult: string;
    invalidResult: string;
    createSnapshotBtn: string;
    restoreSnapshotBtn: string;
    deleteSnapshotTooltip: string;
    emptySnapshots: string;
    snapshotWords: string;
  };
  lsp: {
    popupHeader: string;
    relationsLabel: string;
  };
  reader: {
    ivory: string;
    parchment: string;
    sepia: string;
    midnight: string;
    spreadMode: string;
    scrollMode: string;
    widthNarrow: string;
    widthStandard: string;
    widthWide: string;
    editCta: string;
    bookmarkTooltip: string;
    bookmarkSaved: string;
    endOfScene: string;
    readingMinutes: string;
    prevChapter: string;
    nextChapter: string;
    editInStudio: string;
    defaultSceneTitle: string;
  };
  board: {
    defaultActs: readonly [string, string, string, string];
    addSceneTooltip: string;
    moveUpTooltip: string;
    moveDownTooltip: string;
    emptyColumn: string;
    addInline: string;
    moveToPrefix: string;
    wordsShort: string;
  };
  codex: {
    title: string;
    subtitle: string;
    createEntityBtn: string;
    filters: {
      all: string;
      character: string;
      artifact: string;
      pantheon: string;
      sanctuary: string;
    };
    sort: {
      impact: string;
      title: string;
      revision: string;
    };
    viewGrid: string;
    viewTable: string;
    searchPlaceholder: string;
    inspectHint: string;
    mentionsCount: string;
    linksCount: string;
    emptyTitle: string;
    emptySubtitle: string;
    emptyAction: string;
    table: {
      entity: string;
      category: string;
      impact: string;
      revision: string;
      actions: string;
      openDossier: string;
    };
    creator: {
      title: string;
      nameLabel: string;
      namePlaceholder: string;
      categoryLabel: string;
      archetypeLabel: string;
      archetypeDefault: string;
      statusLabel: string;
      statusDefault: string;
      descLabel: string;
      descPlaceholder: string;
      submitBtn: string;
      categories: {
        character: string;
        artifact: string;
        pantheon: string;
        sanctuary: string;
      };
      templates: {
        dossierHeading: string;
        archetypeField: string;
        statusField: string;
        fallbackBody: string;
        characterDefaultTitle: string;
        characterTemplate: string;
        artifactDefaultTitle: string;
        artifactTemplate: string;
        pantheonDefaultTitle: string;
        pantheonTemplate: string;
        sanctuaryDefaultTitle: string;
        sanctuaryTemplate: string;
      };
    };
    dossier: {
      openInStudio: string;
      tabDossier: string;
      tabRelations: string;
      tabQuickEdit: string;
      addRelationTitle: string;
      targetPlaceholder: string;
      relationTypes: {
        ally: string;
        enemy: string;
        ownership: string;
        hierarchy: string;
      };
      relationDescPlaceholder: string;
      defaultRelationDesc: string;
      linkBtn: string;
      deleteRelationTooltip: string;
      emptyRelations: string;
      saveCanonBtn: string;
      canonSavedToast: string;
    };
  };
  timeline: {
    title: string;
    subtitle: string;
    newMilestoneBtn: string;
    editMilestoneTooltip: string;
    tensorTitle: string;
    tensorScale: string;
    intensityLabel: string;
    jumpToScene: string;
    jumpToSceneShort: string;
    emptyState: string;
    createFirstBtn: string;
    modal: {
      title: string;
      editTitle: string;
      namePlaceholder: string;
      descPlaceholder: string;
      actPlaceholder: string;
      defaultAct: string;
      characterPlaceholder: string;
      defaultCharacter: string;
      intensityLabel: string;
      linkSceneOption: string;
    };
  };
  graph: {
    searchPlaceholder: string;
    teleportBtn: string;
    filterAll: string;
    filterEntities: string;
    resetCamera: string;
    zoomIn: string;
    zoomOut: string;
    minimapCaption: string;
    connectionsCount: string;
    nodeMass: string;
    openSceneBtn: string;
    linkNodeBtn: string;
    relationsListTitle: string;
    unlinkBtn: string;
    modal: {
      title: string;
      nodePrefix: string;
      targetPlaceholder: string;
      descPlaceholder: string;
      submitBtn: string;
      types: {
        ally: string;
        enemy: string;
        ownership: string;
        hierarchy: string;
      };
    };
  };
  settings: {
    title: string;
    subtitle: string;
    resetBtn: string;
    localization: {
      title: string;
      subtitle: string;
      languageLabel: string;
    };
    typography: {
      title: string;
      subtitle: string;
      fontLabel: string;
      fonts: {
        sans: string;
        mono: string;
        serif: string;
      };
      lineHeightLabel: string;
      lineHeights: {
        compact: string;
        standard: string;
        loose: string;
      };
      canvasWidthLabel: string;
      canvasWidths: {
        focus: string;
        wide: string;
        full: string;
      };
      smoothCaretLabel: string;
    };
    database: {
      title: string;
      subtitle: string;
      scenes: string;
      projects: string;
      relations: string;
      events: string;
      size: string;
      walBtn: string;
      vacuumBtn: string;
      autoSaveLabel: string;
      autoSaveOptions: {
        instant: string;
        optimal: string;
        relaxed: string;
        manual: string;
      };
      walDone: string;
      walError: string;
      vacuumDone: string;
      vacuumError: string;
      bytesUnits: readonly [string, string, string, string];
    };
    loreSection: {
      title: string;
      subtitle: string;
      sensitivityLabel: string;
      sensitivities: {
        strict: string;
        balanced: string;
        permissive: string;
      };
      auditOnInputLabel: string;
    };
    productivity: {
      title: string;
      subtitle: string;
      dailyGoalLabel: string;
      bookGoalLabel: string;
      sprintDurationLabel: string;
      hapticsLabel: string;
    };
    about: {
      title: string;
      subtitle: string;
      versionLabel: string;
      licenseLabel: string;
      studioLabel: string;
    };
  };
  mobile: {
    toolsTitle: string;
    chaptersTitle: string;
    bookProgress: string;
    wordsOfGoal: string;
    createSceneBtn: string;
    modulesTitle: string;
    themesTitle: string;
    buildAndUpdatesTitle: string;
    checkUpdateBtn: string;
    checkingBtn: string;
    saveShort: string;
    readerTab: string;
    studioTab: string;
    boardTab: string;
    codexTab: string;
    menuTab: string;
    sprintTab: string;
    importExportTitle: string;
  };
  errors: {
    generic: string;
    projectDeleteLast: string;
    conflictRevision: string;
  };
}