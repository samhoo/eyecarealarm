export type Lang =
  | "zh-CN"
  | "zh-TW"
  | "en"
  | "pt"
  | "es"
  | "ru"
  | "fr"
  | "ko"
  | "de"
  | "ja";

/** All supported languages, in dropdown display order. */
export const LANGS: Lang[] = [
  "zh-CN",
  "zh-TW",
  "en",
  "pt",
  "es",
  "ru",
  "fr",
  "ko",
  "de",
  "ja",
];

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
    restText: (n) => `请看向至少6米(20英尺)外远处物体${n}秒`,
    exitBtn: (n) => `退出 (Esc) ${n}秒`,
    noSound: "无音效",
    importSound: "+ 我的音效",
    knowledgeNA: "护眼知识页面尚未配置",
    langName: "简体中文",
  },
  "zh-TW": {
    headerLine: (m, s, n) =>
      `${m}分${String(s).padStart(2, "0")}秒後眼睛休息一下，今日眼睛已放鬆${n}次`,
    remindToggle: "提醒休息眼睛",
    remindNow: "立即提醒",
    interval: "提醒間隔",
    intervalUnit: "分鐘",
    rest: "休息時長",
    restUnit: "秒",
    overlayOpacity: "遮罩透明度",
    sound: "音效",
    volume: "音量",
    language: "語言",
    autostart: "開機自動啟動",
    moreKnowledge: "更多護眼知識",
    about: "關於 EyeCareAlarm",
    version: "version 0.1.23",
    quit: "退出 EyeCareAlarm",
    launched: "護眼提醒已啟動",
    warmupText: "眼睛需要休息啦~",
    restText: (n) => `請看向至少6公尺(20英尺)外遠處物體${n}秒`,
    exitBtn: (n) => `退出 (Esc) ${n}秒`,
    noSound: "無音效",
    importSound: "+ 我的音效",
    knowledgeNA: "護眼知識頁面尚未設定",
    langName: "繁體中文",
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
    autostart: "Launch at startup",
    moreKnowledge: "More eye-care tips",
    about: "About EyeCareAlarm",
    version: "version 0.1.23",
    quit: "Quit EyeCareAlarm",
    launched: "EyeCareAlarm started",
    warmupText: "Time to rest your eyes~",
    restText: (n) => `Look at something at least 6m (20ft) away for ${n}s`,
    exitBtn: (n) => `Exit (Esc) ${n}s`,
    noSound: "No sound",
    importSound: "+ My sound",
    knowledgeNA: "Eye-care page not configured yet",
    langName: "English",
  },
  pt: {
    headerLine: (m, s, n) =>
      `Pausa para os olhos em ${m}m ${String(s).padStart(2, "0")}s · ${n} pausa${n === 1 ? "" : "s"} hoje`,
    remindToggle: "Lembrete de pausa visual",
    remindNow: "Lembrar agora",
    interval: "Intervalo",
    intervalUnit: "min",
    rest: "Duração da pausa",
    restUnit: "s",
    overlayOpacity: "Opacidade da máscara",
    sound: "Som",
    volume: "Volume",
    language: "Idioma",
    autostart: "Iniciar com o sistema",
    moreKnowledge: "Mais dicas de cuidado visual",
    about: "Sobre o EyeCareAlarm",
    version: "version 0.1.23",
    quit: "Sair do EyeCareAlarm",
    launched: "EyeCareAlarm iniciado",
    warmupText: "Hora de descansar os olhos~",
    restText: (n) => `Olhe para algo a pelo menos 6 m (20 pés) por ${n} s`,
    exitBtn: (n) => `Sair (Esc) ${n}s`,
    noSound: "Sem som",
    importSound: "+ Meu som",
    knowledgeNA: "Página de cuidado visual ainda não configurada",
    langName: "Português",
  },
  es: {
    headerLine: (m, s, n) =>
      `Descanso visual en ${m}m ${String(s).padStart(2, "0")}s · ${n} descanso${n === 1 ? "" : "s"} hoy`,
    remindToggle: "Recordatorio de descanso visual",
    remindNow: "Recordar ahora",
    interval: "Intervalo",
    intervalUnit: "min",
    rest: "Duración del descanso",
    restUnit: "s",
    overlayOpacity: "Opacidad de la máscara",
    sound: "Sonido",
    volume: "Volumen",
    language: "Idioma",
    autostart: "Iniciar con el sistema",
    moreKnowledge: "Más consejos para el cuidado visual",
    about: "Acerca de EyeCareAlarm",
    version: "version 0.1.23",
    quit: "Salir de EyeCareAlarm",
    launched: "EyeCareAlarm iniciado",
    warmupText: "Tus ojos necesitan un descanso~",
    restText: (n) => `Mira un objeto a al menos 6 m (20 pies) durante ${n} s`,
    exitBtn: (n) => `Salir (Esc) ${n}s`,
    noSound: "Sin sonido",
    importSound: "+ Mi sonido",
    knowledgeNA: "Página de cuidado visual aún no configurada",
    langName: "Español",
  },
  ru: {
    headerLine: (m, s, n) =>
      `Отдых для глаз через ${m}м ${String(s).padStart(2, "0")}с · Сегодня: ${n}`,
    remindToggle: "Напоминание об отдыхе",
    remindNow: "Напомнить сейчас",
    interval: "Интервал",
    intervalUnit: "мин",
    rest: "Длительность отдыха",
    restUnit: "с",
    overlayOpacity: "Прозрачность маски",
    sound: "Звук",
    volume: "Громкость",
    language: "Язык",
    autostart: "Автозапуск при входе",
    moreKnowledge: "Больше о заботе о глазах",
    about: "О программе EyeCareAlarm",
    version: "version 0.1.23",
    quit: "Выйти из EyeCareAlarm",
    launched: "EyeCareAlarm запущен",
    warmupText: "Глазам нужен отдых~",
    restText: (n) => `Смотрите на объект не ближе 6 м (20 футов) в течение ${n} с`,
    exitBtn: (n) => `Выход (Esc) ${n}с`,
    noSound: "Без звука",
    importSound: "+ Мой звук",
    knowledgeNA: "Страница о заботе о глазах пока не настроена",
    langName: "Русский",
  },
  fr: {
    headerLine: (m, s, n) =>
      `Pause visuelle dans ${m}m ${String(s).padStart(2, "0")}s · ${n} pause${n === 1 ? "" : "s"} aujourd'hui`,
    remindToggle: "Rappel de pause visuelle",
    remindNow: "Rappeler maintenant",
    interval: "Intervalle",
    intervalUnit: "min",
    rest: "Durée de la pause",
    restUnit: "s",
    overlayOpacity: "Opacité du masque",
    sound: "Son",
    volume: "Volume",
    language: "Langue",
    autostart: "Démarrage automatique",
    moreKnowledge: "Plus de conseils pour les yeux",
    about: "À propos d'EyeCareAlarm",
    version: "version 0.1.23",
    quit: "Quitter EyeCareAlarm",
    launched: "EyeCareAlarm démarré",
    warmupText: "Vos yeux ont besoin de repos~",
    restText: (n) => `Regardez un objet à au moins 6 m (20 pieds) pendant ${n} s`,
    exitBtn: (n) => `Quitter (Esc) ${n}s`,
    noSound: "Aucun son",
    importSound: "+ Mon son",
    knowledgeNA: "Page de conseils non configurée",
    langName: "Français",
  },
  ko: {
    headerLine: (m, s, n) =>
      `${m}분 ${String(s).padStart(2, "0")}초 후 눈 휴식 · 오늘 ${n}회`,
    remindToggle: "눈 휴식 알림",
    remindNow: "지금 알림",
    interval: "알림 간격",
    intervalUnit: "분",
    rest: "휴식 시간",
    restUnit: "초",
    overlayOpacity: "마스크 투명도",
    sound: "효과음",
    volume: "볼륨",
    language: "언어",
    autostart: "시작 시 자동 실행",
    moreKnowledge: "눈 건강 정보 더 보기",
    about: "EyeCareAlarm 정보",
    version: "version 0.1.23",
    quit: "EyeCareAlarm 종료",
    launched: "눈 보호 알림이 시작되었습니다",
    warmupText: "눈을 쉬게 할 시간이에요~",
    restText: (n) => `최소 6m(20피트) 이상 떨어진 물체를 ${n}초간 바라보세요`,
    exitBtn: (n) => `종료 (Esc) ${n}초`,
    noSound: "무음",
    importSound: "+ 내 효과음",
    knowledgeNA: "눈 건강 페이지가 아직 설정되지 않았습니다",
    langName: "한국어",
  },
  de: {
    headerLine: (m, s, n) =>
      `Augenpause in ${m}m ${String(s).padStart(2, "0")}s · heute ${n} Pause${n === 1 ? "" : "n"}`,
    remindToggle: "Augenpausen-Erinnerung",
    remindNow: "Jetzt erinnern",
    interval: "Intervall",
    intervalUnit: "Min.",
    rest: "Pausendauer",
    restUnit: "Sek.",
    overlayOpacity: "Masken-Deckkraft",
    sound: "Klang",
    volume: "Lautstärke",
    language: "Sprache",
    autostart: "Beim Anmelden starten",
    moreKnowledge: "Mehr Augenpflege-Tipps",
    about: "Über EyeCareAlarm",
    version: "version 0.1.23",
    quit: "EyeCareAlarm beenden",
    launched: "EyeCareAlarm gestartet",
    warmupText: "Zeit für eine Augenpause~",
    restText: (n) => `Schaue ${n} Sek. auf ein mindestens 6 m (20 Fuß) entferntes Objekt`,
    exitBtn: (n) => `Beenden (Esc) ${n}s`,
    noSound: "Kein Klang",
    importSound: "+ Mein Klang",
    knowledgeNA: "Augenpflege-Seite noch nicht konfiguriert",
    langName: "Deutsch",
  },
  ja: {
    headerLine: (m, s, n) =>
      `${m}分${String(s).padStart(2, "0")}秒後に目を休めましょう · 今日${n}回`,
    remindToggle: "目の休息リマインダー",
    remindNow: "今すぐ通知",
    interval: "通知間隔",
    intervalUnit: "分",
    rest: "休息時間",
    restUnit: "秒",
    overlayOpacity: "マスクの透明度",
    sound: "サウンド",
    volume: "音量",
    language: "言語",
    autostart: "ログイン時に起動",
    moreKnowledge: "目のケア情報をもっと見る",
    about: "EyeCareAlarm について",
    version: "version 0.1.23",
    quit: "EyeCareAlarm を終了",
    launched: "目の休息リマインダーが起動しました",
    warmupText: "目を休めましょう~",
    restText: (n) => `少なくとも6メートル(20フィート)先の物を${n}秒間見てください`,
    exitBtn: (n) => `終了 (Esc) ${n}秒`,
    noSound: "音なし",
    importSound: "+ マイサウンド",
    knowledgeNA: "目のケアページはまだ設定されていません",
    langName: "日本語",
  },
};

export function t(lang: Lang): Messages {
  return dict[lang] ?? dict["zh-CN"];
}

/** Best-effort OS language detection for first run. */
export function detectLang(): Lang {
  const l = navigator.language.toLowerCase();
  if (l.startsWith("zh")) {
    return l.includes("tw") || l.includes("hk") || l.includes("hant") ? "zh-TW" : "zh-CN";
  }
  for (const lang of LANGS) {
    if (lang !== "zh-CN" && lang !== "zh-TW" && l.startsWith(lang)) return lang;
  }
  return "en";
}
