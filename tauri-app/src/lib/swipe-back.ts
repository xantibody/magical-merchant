import { onMount, onCleanup } from "solid-js";

/** 戻りスワイプと判定する閾値。 */
const EDGE_ZONE = 30; // 開始点が画面左端からこの範囲(px)
const MIN_DISTANCE = 70; // 右方向への最小移動(px)
const MAX_OFF_AXIS = 45; // 許容する縦ブレ(px)
const MAX_DURATION = 500; // 速いフリックのみ受け付ける(ms)

export interface SwipePoint {
  x: number;
  y: number;
  t: number;
}

/**
 * 左端から始まり、速く・概ね水平に・十分な距離だけ右へ動いたかを判定する。
 * iOS / Android の「戻る」エッジスワイプと同じ手触りを狙う。
 */
export function isBackSwipe(start: SwipePoint, end: SwipePoint): boolean {
  const dx = end.x - start.x;
  const dy = Math.abs(end.y - start.y);
  const dt = end.t - start.t;
  return (
    start.x <= EDGE_ZONE && dx >= MIN_DISTANCE && dy <= MAX_OFF_AXIS && dt > 0 && dt <= MAX_DURATION
  );
}

/**
 * 左端スワイプで onBack を呼ぶ。enabled() が false の間は無視するので、
 * 戻り先がない画面（入力/一覧ルート）では発火しない。
 */
export function createSwipeBack(onBack: () => void, enabled: () => boolean = () => true): void {
  let start: SwipePoint | null = null;

  const onStart = (e: TouchEvent) => {
    if (!enabled() || e.touches.length !== 1 || e.touches[0].clientX > EDGE_ZONE) {
      start = null;
      return;
    }
    const t = e.touches[0];
    start = { x: t.clientX, y: t.clientY, t: e.timeStamp };
  };

  const onEnd = (e: TouchEvent) => {
    if (!start) return;
    const t = e.changedTouches[0];
    const end: SwipePoint = { x: t.clientX, y: t.clientY, t: e.timeStamp };
    if (isBackSwipe(start, end)) onBack();
    start = null;
  };

  onMount(() => {
    document.addEventListener("touchstart", onStart, { passive: true });
    document.addEventListener("touchend", onEnd, { passive: true });
    onCleanup(() => {
      document.removeEventListener("touchstart", onStart);
      document.removeEventListener("touchend", onEnd);
    });
  });
}
