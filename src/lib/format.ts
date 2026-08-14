export function fmtMoney(v: number): string {
  return v.toFixed(2);
}

export function fmtDuration(totalSeconds: number): string {
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = Math.floor(totalSeconds % 60);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(h)}:${pad(m)}:${pad(s)}`;
}

export const statusText: Record<string, string> = {
  before_work: "待上班",
  working: "上班中",
  lunch: "午休中",
  after_work: "已下班",
  rest_day: "休息日",
};

export const modeText: Record<string, string> = {
  topmost: "置顶",
  desktop: "桌面",
  normal: "普通",
};
