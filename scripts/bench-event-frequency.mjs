/**
 * bench-event-frequency.mjs - 悬停事件调用频率分析
 *
 * 目的：量化下列三项优化的实际效果（以"每次鼠标移动触发的 hitTest 次数"计）：
 * 1. 移除 pointermove / mousemove 双重绑定
 * 2. 按 requestAnimationFrame 节流 pointermove
 * 3. 单次 hitTest 的耗时下降（由 bench-hit-test.mjs 测得）
 *
 * 关键前提：现代浏览器对一次物理鼠标移动会同时派发 pointermove 与 mousemove。
 * 因此原实现每次移动执行 2 次 hitTest。
 *
 * 运行：node bench-event-frequency.mjs
 */

/** 真实环境中单次 hitTest 的耗时（微秒）。取自 bench-hit-test.mjs 的实测值。 */
const HIT_TEST_US_NO_CACHE = 51.24
const HIT_TEST_US_CACHED = 35.87
const HIT_TEST_US_REPEAT = 0.22

/**
 * 模拟不同鼠标移动频率与刷新率下的每秒 hitTest 调用次数。
 *
 * @param {object} opts
 * @param {number} opts.moveEventsPerSec 物理鼠标移动事件频率（游戏鼠标可达 500+）
 * @param {number} opts.displayHz 显示器刷新率，决定 RAF 节流上限
 */
function analyze({ moveEventsPerSec, displayHz }) {
  // —— 原实现 ——
  // 每次移动派发 pointermove + mousemove，各执行 1 次 hitTest，无节流
  const beforeCallsPerSec = moveEventsPerSec * 2
  const beforeMsPerSec = (beforeCallsPerSec * HIT_TEST_US_NO_CACHE) / 1000

  // —— 修复后 ——
  // 仅 pointermove，且合并到 RAF：每帧最多 1 次
  const afterCallsPerSec = Math.min(moveEventsPerSec, displayHz)
  const afterMsPerSec = (afterCallsPerSec * HIT_TEST_US_CACHED) / 1000

  return {
    moveEventsPerSec,
    displayHz,
    beforeCallsPerSec,
    afterCallsPerSec,
    beforeMsPerSec,
    afterMsPerSec,
    reductionPct: ((beforeCallsPerSec - afterCallsPerSec) / beforeCallsPerSec) * 100,
    cpuReductionPct: ((beforeMsPerSec - afterMsPerSec) / beforeMsPerSec) * 100,
  }
}

const scenarios = [
  { label: '普通鼠标 · 60Hz', moveEventsPerSec: 125, displayHz: 60 },
  { label: '普通鼠标 · 144Hz', moveEventsPerSec: 125, displayHz: 144 },
  { label: '高刷鼠标 · 60Hz', moveEventsPerSec: 500, displayHz: 60 },
  { label: '高刷鼠标 · 144Hz', moveEventsPerSec: 1000, displayHz: 144 },
  { label: '高刷鼠标 · 240Hz', moveEventsPerSec: 1000, displayHz: 240 },
]

console.log('\n悬停路径 CPU 占用分析（每次 hitTest 无缓存 = '
  + HIT_TEST_US_NO_CACHE + 'µs，带缓存 = ' + HIT_TEST_US_CACHED + 'µs）\n')
console.log(
  '场景'.padEnd(20) +
  '修复前调用/秒'.padStart(14) +
  '修复后调用/秒'.padStart(14) +
  '调用降幅'.padStart(10) +
  '修复前CPU(ms/s)'.padStart(16) +
  '修复后CPU(ms/s)'.padStart(16) +
  'CPU降幅'.padStart(10),
)
console.log('-'.repeat(102))

for (const s of scenarios) {
  const r = analyze(s)
  console.log(
    s.label.padEnd(20) +
    r.beforeCallsPerSec.toLocaleString().padStart(14) +
    r.afterCallsPerSec.toLocaleString().padStart(14) +
    (r.reductionPct.toFixed(0) + '%').padStart(10) +
    r.beforeMsPerSec.toFixed(0).padStart(16) +
    r.afterMsPerSec.toFixed(0).padStart(16) +
    (r.cpuReductionPct.toFixed(0) + '%').padStart(10),
  )
}

// ——— 极端场景：持续快速划动 1 秒的累计 CPU 占用占比 ———
console.log('\n单核 CPU 占用占比（假设单核 1000ms/s 可用）：\n')
for (const s of scenarios) {
  const r = analyze(s)
  console.log(
    s.label.padEnd(20) +
    `修复前 ${((r.beforeMsPerSec / 1000) * 100).toFixed(2)}%`.padStart(24) +
    `  →  修复后 ${((r.afterMsPerSec / 1000) * 100).toFixed(2)}%`.padStart(24),
  )
}

console.log('\n注：以上为"鼠标持续快速划动"的最坏情况。')
console.log('常态下指针静止或缓慢移动，调用频率远低于此，实际开销可忽略。')
