/**
 * bench-hit-test.mjs - 命中检测路径的微基准
 *
 * 目的：量化 usePixelHitTest 中两级缓存的实际收益，验证"字符串解析开销"
 * 与"重复坐标查询"两类场景的改善幅度。
 *
 * 说明：本脚本复刻 hitTest 的核心计算路径（不依赖 DOM/canvas），
 * 因为 getImageData 的真实开销无法在 Node 中测量。canvas 回读部分
 * 以固定的模拟代价表示，以便对比缓存命中与否的差异。
 *
 * 运行：node bench-hit-test.mjs
 */

/** 模拟 getImageData 的开销：真实环境约 0.1–1ms，此处取保守的 0.05ms。 */
const CANVAS_READ_COST_MS = 0.05

/** 基准参数 */
const ITERATIONS = 200_000
const CONTAINER_SIZE = 240

/** ——— 被测实现 A：无缓存（原始版本） ——— */
function hitTestNoCache(divX, divY, bgSize, bgPos, naturalWidth, naturalHeight) {
  const size = bgSize.split(' ').map(parseFloat)
  const pos = bgPos.split(' ').map(parseFloat)
  const scale = size[0] / naturalWidth
  if (!Number.isFinite(scale) || scale <= 0) return false

  const srcX = Math.round((divX - pos[0]) / scale)
  const srcY = Math.round((divY - pos[1]) / scale)
  if (srcX < 0 || srcX >= naturalWidth || srcY < 0 || srcY >= naturalHeight) return false

  // 模拟 canvas 回读
  busyWait(CANVAS_READ_COST_MS)
  return srcX % 3 !== 0
}

/** ——— 被测实现 B：带两级缓存（当前版本） ——— */
function createCachedHitTest() {
  let alphaCache = new Map()
  let alphaCacheFrame = ''
  let parsedSizeFor = ''
  let parsedSize = []
  let parsedPosFor = ''
  let parsedPos = []

  function setFrameKey(frameKey) {
    if (frameKey === alphaCacheFrame) return
    alphaCacheFrame = frameKey
    alphaCache.clear()
  }

  function hitTest(divX, divY, bgSize, bgPos, naturalWidth, naturalHeight) {
    if (bgSize !== parsedSizeFor) {
      parsedSizeFor = bgSize
      parsedSize = bgSize.split(' ').map(parseFloat)
    }
    if (bgPos !== parsedPosFor) {
      parsedPosFor = bgPos
      parsedPos = bgPos.split(' ').map(parseFloat)
    }

    const scale = parsedSize[0] / naturalWidth
    if (!Number.isFinite(scale) || scale <= 0) return false

    const srcX = Math.round((divX - parsedPos[0]) / scale)
    const srcY = Math.round((divY - parsedPos[1]) / scale)
    if (srcX < 0 || srcX >= naturalWidth || srcY < 0 || srcY >= naturalHeight) return false

    const key = `${srcX},${srcY}`
    const cached = alphaCache.get(key)
    if (cached !== undefined) return cached

    busyWait(CANVAS_READ_COST_MS)
    const hit = srcX % 3 !== 0
    alphaCache.set(key, hit)
    return hit
  }

  return { hitTest, setFrameKey }
}

/** 忙等待指定毫秒，模拟同步阻塞开销（比 setTimeout 更贴近真实回读）。 */
function busyWait(ms) {
  const end = performance.now() + ms
  while (performance.now() < end) {
    /* spin */
  }
}

function percentile(sorted, p) {
  return sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * p))]
}

function measure(label, fn) {
  const samples = []
  const start = performance.now()
  for (let i = 0; i < ITERATIONS; i++) {
    const t0 = performance.now()
    fn(i)
    samples.push(performance.now() - t0)
  }
  const total = performance.now() - start
  samples.sort((a, b) => a - b)
  return {
    label,
    totalMs: total,
    avgUs: (total / ITERATIONS) * 1000,
    p50Us: percentile(samples, 0.5) * 1000,
    p99Us: percentile(samples, 0.99) * 1000,
  }
}

function report(rows) {
  console.log(
    '\n' + '场景'.padEnd(30) + '总耗时(ms)'.padStart(12) +
    '均值(µs)'.padStart(12) + 'P50(µs)'.padStart(10) + 'P99(µs)'.padStart(10),
  )
  console.log('-'.repeat(76))
  for (const r of rows) {
    console.log(
      r.label.padEnd(30) +
      r.totalMs.toFixed(1).padStart(12) +
      r.avgUs.toFixed(2).padStart(12) +
      r.p50Us.toFixed(2).padStart(10) +
      r.p99Us.toFixed(2).padStart(10),
    )
  }
}

const BG_SIZE = `${CONTAINER_SIZE}px ${CONTAINER_SIZE}px`
const BG_POS = '0px 0px'
const NAT_W = 512
const NAT_H = 512

// ——— 场景 1：模拟快速移动 —— 坐标持续变化，每帧内无重复查询 ———
console.log('基准：每次 getImageData 模拟开销 =', CANVAS_READ_COST_MS, 'ms')
console.log('迭代次数 =', ITERATIONS.toLocaleString())

const movingRows = []
{
  const coords = []
  for (let i = 0; i < ITERATIONS; i++) {
    const angle = (i / 60) * Math.PI * 2
    coords.push([
      CONTAINER_SIZE / 2 + Math.cos(angle) * 80 + (i % 7),
      CONTAINER_SIZE / 2 + Math.sin(angle) * 80 + (i % 5),
    ])
  }

  movingRows.push(measure('移动中 · 无缓存', (i) => {
    const [x, y] = coords[i]
    return hitTestNoCache(x, y, BG_SIZE, BG_POS, NAT_W, NAT_H)
  }))

  const cached = createCachedHitTest()
  let frame = 0
  movingRows.push(measure('移动中 · 带缓存', (i) => {
    // 每 600 次迭代模拟一次帧切换（约等于 10fps 下的相对频率）
    if (i % 600 === 0) cached.setFrameKey(`frame-${frame++}`)
    const [x, y] = coords[i]
    return cached.hitTest(x, y, BG_SIZE, BG_POS, NAT_W, NAT_H)
  }))
}

// ——— 场景 2：同帧内重复查询同一坐标（mousedown → click 相邻触发） ———
const repeatRows = []
{
  repeatRows.push(measure('同帧重复 · 无缓存', () => {
    return hitTestNoCache(120, 130, BG_SIZE, BG_POS, NAT_W, NAT_H)
  }))

  const cached = createCachedHitTest()
  repeatRows.push(measure('同帧重复 · 带缓存', () => {
    return cached.hitTest(120, 130, BG_SIZE, BG_POS, NAT_W, NAT_H)
  }))
}

// ——— 场景 3：仅字符串解析开销（排除 canvas 回读） ———
const parseRows = []
{
  parseRows.push(measure('解析路径 · 无缓存', () => {
    const size = BG_SIZE.split(' ').map(parseFloat)
    const pos = BG_POS.split(' ').map(parseFloat)
    return size[0] + pos[0]
  }))

  let parsedSizeFor = ''
  let parsedSize = []
  let parsedPosFor = ''
  let parsedPos = []
  parseRows.push(measure('解析路径 · 带缓存', () => {
    if (BG_SIZE !== parsedSizeFor) {
      parsedSizeFor = BG_SIZE
      parsedSize = BG_SIZE.split(' ').map(parseFloat)
    }
    if (BG_POS !== parsedPosFor) {
      parsedPosFor = BG_POS
      parsedPos = BG_POS.split(' ').map(parseFloat)
    }
    return parsedSize[0] + parsedPos[0]
  }))
}

report([...movingRows, ...repeatRows, ...parseRows])

console.log('\n结论提示：')
console.log('- "同帧重复"场景体现逐像素缓存收益（消除重复 canvas 回读）')
console.log('- "解析路径"体现字符串解析缓存收益（与 canvas 回读无关）')
console.log('- "移动中"场景的收益取决于帧内坐标重复率，通常接近无缓存')
