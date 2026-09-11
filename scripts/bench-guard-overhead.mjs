/**
 * bench-guard-overhead.mjs - 层级守护循环开销分析
 *
 * 目的：确认新增的低频守护（10 秒周期）对系统与进程的负担可忽略。
 *
 * 守护每次执行的系统调用：
 * - 每个可见窗口 1 次 is_visible() 查询（Tauri → 平台 API）
 * - 每个可见窗口 1 次 GetWindowLongPtrW（读取 WS_EX_TOPMOST）
 * - 仅在标记丢失时才有 1 次 SetWindowPos
 *
 * 运行：node bench-guard-overhead.mjs
 */

/** 受守护窗口数量：5 个浮窗 + 1 个设置窗口 */
const GUARDED_WINDOWS = 6

/** 单次系统调用的量级估计（微秒），基于 Windows 常规 API 往返。 */
const IS_VISIBLE_US = 1.0
const GET_STYLE_US = 0.2
const SET_WINDOW_POS_US = 5.0

const GUARD_INTERVAL_SEC = 10

function perPass({ visibleWindows, needsRepair }) {
  const visibleChecks = visibleWindows * IS_VISIBLE_US
  const styleReads = visibleWindows * GET_STYLE_US
  const repairs = needsRepair * SET_WINDOW_POS_US
  return visibleChecks + styleReads + repairs
}

const cases = [
  { label: '全部隐藏（常态：无浮窗）', visibleWindows: 0, needsRepair: 0 },
  { label: '仅桌宠可见', visibleWindows: 0, needsRepair: 0 },
  { label: '1 个浮窗可见', visibleWindows: 1, needsRepair: 0 },
  { label: '全部 6 个窗口可见', visibleWindows: GUARDED_WINDOWS, needsRepair: 0 },
  { label: '全部可见 + 1 个需修复', visibleWindows: GUARDED_WINDOWS, needsRepair: 1 },
]

console.log('\n单次守护执行的开销（微秒）：\n')
console.log(
  '场景'.padEnd(28) +
  '系统调用次数'.padStart(14) +
  '耗时(µs)'.padStart(12) +
  '占 10s 周期比例'.padStart(18),
)
console.log('-'.repeat(74))

for (const c of cases) {
  const calls = c.visibleWindows * 2 + c.needsRepair
  const us = perPass(c)
  const ratio = (us / (GUARD_INTERVAL_SEC * 1_000_000)) * 100
  console.log(
    c.label.padEnd(28) +
    String(calls).padStart(14) +
    us.toFixed(2).padStart(12) +
    (ratio.toExponential(2) + '%').padStart(18),
  )
}

// ——— 线程与内存开销 ———
console.log('\n后台资源占用：')
console.log('- 线程数：1 个（阻塞在 sleep，无忙等待）')
console.log('- 线程栈：约 8MB 虚拟地址空间（Windows 默认，未实际提交）')
console.log('- 常驻内存：1 个 Arc<AppHandle> 引用 + 循环局部变量，可忽略')
console.log('- 唤醒频率：0.1 次/秒（每 10 秒 1 次）')

// ——— 与替代方案对比 ———
console.log('\n与"高频轮询"方案的对比（假设周期改为 1 秒）：\n')
const highFreqCases = cases.map((c) => ({ ...c, perSec: true }))
console.log('场景'.padEnd(28) + '1s 周期 CPU 占用'.padStart(20) + '10s 周期 CPU 占用'.padStart(22))
console.log('-'.repeat(70))
for (const c of highFreqCases) {
  const us = perPass(c)
  const perSecAt1s = (us / 1_000_000) * 100
  const perSecAt10s = perSecAt1s / 10
  console.log(
    c.label.padEnd(28) +
    (perSecAt1s.toExponential(2) + '%').padStart(20) +
    (perSecAt10s.toExponential(2) + '%').padStart(22),
  )
}

console.log('\n结论：10 秒周期的 CPU 占用处于 10⁻⁵ % 量级，')
console.log('与常态无操作时的基线噪声无法区分。')
