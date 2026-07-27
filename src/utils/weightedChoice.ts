/** weightedChoice.ts - 可注入随机源的加权选择工具 */

export interface WeightedChoice<T> {
  value: T
  weight: number
}

export function chooseWeighted<T>(
  choices: readonly WeightedChoice<T>[],
  random: () => number = Math.random,
): T | null {
  const validChoices = choices.filter((choice) => Number.isFinite(choice.weight) && choice.weight > 0)
  const totalWeight = validChoices.reduce((total, choice) => total + choice.weight, 0)
  if (totalWeight <= 0) return null

  let remaining = Math.min(Math.max(random(), 0), 0.999999999) * totalWeight
  for (const choice of validChoices) {
    remaining -= choice.weight
    if (remaining < 0) return choice.value
  }

  return validChoices[validChoices.length - 1]?.value ?? null
}
