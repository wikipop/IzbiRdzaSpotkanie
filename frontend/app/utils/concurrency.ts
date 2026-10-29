export async function runWithConcurrency<T>(
  items: readonly T[],
  limit: number,
  task: (item: T) => Promise<void>,
): Promise<void> {
  if (!Number.isInteger(limit) || limit < 1) throw new Error('Concurrency limit must be a positive integer')

  let cursor = 0
  async function worker(): Promise<void> {
    while (cursor < items.length) {
      const item = items[cursor++]
      if (item === undefined) return
      await task(item)
    }
  }

  await Promise.all(Array.from(
    { length: Math.min(limit, items.length) },
    () => worker(),
  ))
}
