/**
 * 情绪展示资源：与后端 `Emotion` 小写标签及 `roles/{roleId}/assets/images/` 下文件名对齐。
 * 新增后端情绪枚举时，须同步更新本文件并补充角色包图片（可用占位图）。
 * 已支持：happy / sad / angry / shy / confused / disgust* / neutral / excited 等。
 */

/** 小写 key；未知键由 UI 回退为原文或默认图标 */
export const emotionToEmoji: Record<string, string> = {
  happy: "😊",
  sad: "😢",
  angry: "😠",
  shy: "☺️",
  confused: "😕",
  disgust: "🙄",
  neutral: "😐",
  /** 后端若未来扩展 `Emotion::Excited` 等，可在此与图片一并补齐 */
  excited: "🤩",
};

/** 文件名（不含路径）；缺省回退为 `{emotion}.png` */
export const emotionToImage: Record<string, string> = {
  happy: "happy.png",
  sad: "sad.png",
  angry: "angry.png",
  shy: "shy.png",
  confused: "confused.png",
  disgust: "disgust_light.png",
  neutral: "normal.png",
  excited: "excited.png",
  disgust_light: "disgust_light.png",
  disgust_mid: "disgust_mid.png",
  disgust_heavy: "disgust_heavy.png",
};

/** 顶栏中文标签；未知键显示原始 emotion 字符串 */
export const emotionToLabelZh: Record<string, string> = {
  happy: "开心",
  sad: "难过",
  angry: "生气",
  shy: "害羞",
  confused: "困惑",
  disgust: "嫌弃",
  neutral: "平静",
  excited: "兴奋",
};

export function emotionToAssetFilename(emotion: string): string {
  const e = emotion.trim().toLowerCase();
  return emotionToImage[e] ?? `${e}.png`;
}
