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
  quit: string;
  launched: string;
  warmupText: string;
  restText: (n: number) => string;
  exitBtn: (n: number) => string;
  noSound: string;
  importSound: string;
  checkUpdate: string;
  checking: string;
  updateAvailable: (v: string) => string;
  upToDate: string;
  checkFailed: string;
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
    moreKnowledge: "查看20-20-20护眼法则",
    about: "关于 EyeCareAlarm",
    quit: "退出 EyeCareAlarm",
    launched: "护眼提醒已启动",
    warmupText: "眼睛需要休息啦~",
    restText: (n) => `目光离开屏幕，向20英尺(6米)以外物体眺望${n}秒`,
    exitBtn: (n) => `退出 (Esc) ${n}秒`,
    noSound: "无音效",
    importSound: "+ 我的音效",
    langName: "简体中文",
    checkUpdate: "检查更新…",
    checking: "检查中…",
    updateAvailable: (v) => `发现新版本 ${v}，点击下载`,
    upToDate: "已是最新版本",
    checkFailed: "检查失败，请稍后重试",
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
    moreKnowledge: "查看20-20-20護眼法則",
    about: "關於 EyeCareAlarm",
    quit: "退出 EyeCareAlarm",
    launched: "護眼提醒已啟動",
    warmupText: "眼睛需要休息啦~",
    restText: (n) => `目光離開螢幕，向20英尺(6公尺)以外物體眺望${n}秒`,
    exitBtn: (n) => `退出 (Esc) ${n}秒`,
    noSound: "無音效",
    importSound: "+ 我的音效",
    langName: "繁體中文",
    checkUpdate: "檢查更新…",
    checking: "檢查中…",
    updateAvailable: (v) => `發現新版本 ${v}，點擊下載`,
    upToDate: "已是最新版本",
    checkFailed: "檢查失敗，請稍後重試",
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
    moreKnowledge: "Learn about the 20-20-20 rule",
    about: "About EyeCareAlarm",
    quit: "Quit EyeCareAlarm",
    launched: "EyeCareAlarm started",
    warmupText: "Time to rest your eyes~",
    restText: (n) => `Look away from the screen and gaze at something 20 feet (6m) away for ${n}s`,
    exitBtn: (n) => `Exit (Esc) ${n}s`,
    noSound: "No sound",
    importSound: "+ My sound",
    langName: "English",
    checkUpdate: "Check for updates…",
    checking: "Checking…",
    updateAvailable: (v) => `New version ${v} — click to download`,
    upToDate: "You are up to date",
    checkFailed: "Check failed, try again later",
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
    moreKnowledge: "Saiba mais sobre a regra 20-20-20",
    about: "Sobre o EyeCareAlarm",
    quit: "Sair do EyeCareAlarm",
    launched: "EyeCareAlarm iniciado",
    warmupText: "Hora de descansar os olhos~",
    restText: (n) => `Desvie o olhar da tela e observe um objeto a 20 pés (6 m) por ${n} s`,
    exitBtn: (n) => `Sair (Esc) ${n}s`,
    noSound: "Sem som",
    importSound: "+ Meu som",
    checkUpdate: "Verificar atualizações…",
    checking: "Verificando…",
    updateAvailable: (v) => `Nova versão ${v} — clique para baixar`,
    upToDate: "Você está atualizado",
    checkFailed: "Falha ao verificar, tente mais tarde",
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
    moreKnowledge: "Consulta la regla 20-20-20",
    about: "Acerca de EyeCareAlarm",
    quit: "Salir de EyeCareAlarm",
    launched: "EyeCareAlarm iniciado",
    warmupText: "Tus ojos necesitan un descanso~",
    restText: (n) => `Aparta la vista de la pantalla y mira un objeto a 20 pies (6 m) durante ${n} s`,
    exitBtn: (n) => `Salir (Esc) ${n}s`,
    noSound: "Sin sonido",
    importSound: "+ Mi sonido",
    checkUpdate: "Buscar actualizaciones…",
    checking: "Comprobando…",
    updateAvailable: (v) => `Nueva versión ${v} — haz clic para descargar`,
    upToDate: "Ya tienes la última versión",
    checkFailed: "Error al comprobar, inténtalo más tarde",
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
    moreKnowledge: "О правиле 20-20-20 для глаз",
    about: "О программе EyeCareAlarm",
    quit: "Выйти из EyeCareAlarm",
    launched: "EyeCareAlarm запущен",
    warmupText: "Глазам нужен отдых~",
    restText: (n) => `Отведите взгляд от экрана и смотрите на объект в 20 футах (6 м) в течение ${n} с`,
    exitBtn: (n) => `Выход (Esc) ${n}с`,
    noSound: "Без звука",
    importSound: "+ Мой звук",
    checkUpdate: "Проверить обновления…",
    checking: "Проверка…",
    updateAvailable: (v) => `Новая версия ${v} — нажмите для загрузки`,
    upToDate: "У вас последняя версия",
    checkFailed: "Ошибка проверки, попробуйте позже",
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
    moreKnowledge: "Découvrir la règle 20-20-20",
    about: "À propos d'EyeCareAlarm",
    quit: "Quitter EyeCareAlarm",
    launched: "EyeCareAlarm démarré",
    warmupText: "Vos yeux ont besoin de repos~",
    restText: (n) => `Détournez le regard de l'écran et fixez un objet à 20 pieds (6 m) pendant ${n} s`,
    exitBtn: (n) => `Quitter (Esc) ${n}s`,
    noSound: "Aucun son",
    importSound: "+ Mon son",
    checkUpdate: "Rechercher des mises à jour…",
    checking: "Recherche…",
    updateAvailable: (v) => `Nouvelle version ${v} — cliquez pour télécharger`,
    upToDate: "Vous êtes à jour",
    checkFailed: "Échec de la vérification, réessayez plus tard",
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
    moreKnowledge: "20-20-20 눈 건강 규칙 보기",
    about: "EyeCareAlarm 정보",
    quit: "EyeCareAlarm 종료",
    launched: "눈 보호 알림이 시작되었습니다",
    warmupText: "눈을 쉬게 할 시간이에요~",
    restText: (n) => `화면에서 눈을 떼고 20피트(6m) 밖의 물체를 ${n}초간 바라보세요`,
    exitBtn: (n) => `종료 (Esc) ${n}초`,
    noSound: "무음",
    importSound: "+ 내 효과음",
    checkUpdate: "업데이트 확인…",
    checking: "확인 중…",
    updateAvailable: (v) => `새 버전 ${v} — 클릭하여 다운로드`,
    upToDate: "최신 버전입니다",
    checkFailed: "확인 실패, 나중에 다시 시도하세요",
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
    moreKnowledge: "Zur 20-20-20-Regel für die Augen",
    about: "Über EyeCareAlarm",
    quit: "EyeCareAlarm beenden",
    launched: "EyeCareAlarm gestartet",
    warmupText: "Zeit für eine Augenpause~",
    restText: (n) => `Blicke vom Bildschirm weg und schaue ${n} Sek. auf ein 20 Fuß (6 m) entferntes Objekt`,
    exitBtn: (n) => `Beenden (Esc) ${n}s`,
    noSound: "Kein Klang",
    importSound: "+ Mein Klang",
    checkUpdate: "Nach Updates suchen…",
    checking: "Prüfe…",
    updateAvailable: (v) => `Neue Version ${v} — klicken zum Herunterladen`,
    upToDate: "Sie sind auf dem neuesten Stand",
    checkFailed: "Prüfung fehlgeschlagen, später erneut versuchen",
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
    moreKnowledge: "20-20-20 ルールを見る",
    about: "EyeCareAlarm について",
    quit: "EyeCareAlarm を終了",
    launched: "目の休息リマインダーが起動しました",
    warmupText: "目を休めましょう~",
    restText: (n) => `画面から目を離し、20フィート(6メートル)先の物を${n}秒間眺めてください`,
    exitBtn: (n) => `終了 (Esc) ${n}秒`,
    noSound: "音なし",
    importSound: "+ マイサウンド",
    checkUpdate: "更新を確認…",
    checking: "確認中…",
    updateAvailable: (v) => `新しいバージョン ${v} — クリックしてダウンロード`,
    upToDate: "最新バージョンです",
    checkFailed: "確認に失敗しました。後で再試行してください",
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
