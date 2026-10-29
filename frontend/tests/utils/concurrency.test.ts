import { describe, expect, it } from 'vitest'
import { runWithConcurrency } from '~/utils/concurrency'

describe('bounded concurrency', () => {
  it('processes every item without exceeding the limit', async () => {
    let active = 0
    let maximumActive = 0
    const completed: number[] = []

    await runWithConcurrency([1, 2, 3, 4, 5, 6], 2, async (item) => {
      active++
      maximumActive = Math.max(maximumActive, active)
      await Promise.resolve()
      completed.push(item)
      active--
    })

    expect(maximumActive).toBe(2)
    expect(completed.sort((a, b) => a - b)).toEqual([1, 2, 3, 4, 5, 6])
  })

  it('rejects invalid limits', async () => {
    await expect(runWithConcurrency([1], 0, async () => {})).rejects.toThrow('positive integer')
  })

  it('keeps all 168 hourly services within the browser concurrency budget', async () => {
    let active = 0
    let maximumActive = 0
    let completed = 0

    await runWithConcurrency(Array.from({ length: 168 }, (_, index) => index), 8, async () => {
      active++
      maximumActive = Math.max(maximumActive, active)
      await Promise.resolve()
      completed++
      active--
    })

    expect(completed).toBe(168)
    expect(maximumActive).toBe(8)
  })
})
