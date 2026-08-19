export type Lang = "zh-CN" | "en";

export interface Messages {
  headerLine: (m: number, s: number, n: number) => string;
  remindToggle: string;
  remindNow: string;
  interval: string;
  intervalUnit: string;
  rest: string;
  restUnit: string;
  overlayOpacity: string;
  sound: string;
  volume: string;
  language: string;
  autostart: string;
  moreKnowledge: string;
  about: string;
  version: string;
  quit: string;
  launched: string;
  warmupText: string;
  restText: (n: number) => string;
  exitBtn: (n: number) => string;
  noSound: string;
  importSound: string;
  knowledgeNA: string;
  langName: string;
}

const dict: Record<Lang, Messages> = {
  "zh-CN": {
    headerLine: (m, s, n) =>
      `${m}分${String(s).padStart(2, "0")}秒后眼睛休息一下，今日眼睛已放松${n}次`,
    remindToggle: "提醒休息眼睛",
    remindNow: "立即提醒",
    interval: "提醒间隔",
    intervalUnit: "分钟",
    rest: "休息时长",
    restUnit: "秒",
    overlayOpacity: "遮罩透明度",
    sound: "音效",
    volume: "音量",
    language: "语言",
    autostart: "开机自启动",
    moreKnowledge: "更多护眼知识",
    about: "关于 EyeCareAlarm",
    version: "version 0.1.23",
    quit: "退出 EyeCareAlarm",
    launched: "护眼提醒已启动",
    warmupText: "眼睛需要休息啦~",
    restText: (n: number) => `请看向至少6米(20英尺)外远处物体${n}秒`,
    exitBtn: (n: number) => `退出 (Esc) ${n}秒`,
    noSound: "无音效",
    importSound: "+ 我的音效",
    knowledgeNA: "护眼知识页面尚未配置",
    langName: "简体中文",
  },
  en: {
    headerLine: (m, s, n) =>
      `Eye rest in ${m}m ${String(s).padStart(2, "0")}s · ${n} eye rest${n === 1 ? "" : "s"} today`,
    remindToggle: "Eye rest reminder",
    remindNow: "Remind now",
    interval: "Interval",
    intervalUnit: "min",
    rest: "Rest duration",
    restUnit: "sec",
    overlayOpacity: "Overlay opacity",
    sound: "Sound",
    volume: "Volume",
    language: "Language",
    autostart: "Launch at login",
    moreKnowledge: "More eye-care tips",
    about: "About EyeCareAlarm",
    version: "version 0.1.23",
    quit: "Quit EyeCareAlarm",
    launched: "EyeCareAlarm started",
    warmupText: "Time to rest your eyes~",
    restText: (n: number) => `Look at something at least 6m (20ft) away for ${n}s`,
    exitBtn: (n: number) => `Exit (Esc) ${n}s`,
    noSound: "No sound",
    importSound: "+ My sound",
    knowledgeNA: "Eye-care page not configured yet",
    langName: "English",
  },
} as const;

export function t(lang: Lang): Messages {
  return dict[lang] ?? dict["zh-CN"];
}

/** Best-effort OS language detection for first run. */
export function detectLang(): Lang {
  return navigator.language.toLowerCase().startsWith("zh") ? "zh-CN" : "en";
}
