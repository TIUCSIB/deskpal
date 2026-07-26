import type { ClassValue } from 'clsx'
import { clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

/** 合并 Tailwind 与条件类名 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
