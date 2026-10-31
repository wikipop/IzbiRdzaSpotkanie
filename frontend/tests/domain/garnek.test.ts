import { describe, expect, it } from 'vitest'
import { shouldShowGarnek } from '~/domain/garnek'

describe('Garnek dialogue frequency', () => {
  it('requires an armed comparison and a missing overlap', () => {
    expect(shouldShowGarnek({
      comparisonRequested: false,
      evaluation: { status: 'no-overlap' },
      dismissed: false,
    })).toBe(false)
    expect(shouldShowGarnek({
      comparisonRequested: true,
      evaluation: { status: 'overlap' },
      dismissed: false,
    })).toBe(false)
    expect(shouldShowGarnek({
      comparisonRequested: true,
      evaluation: { status: 'no-overlap' },
      dismissed: false,
    })).toBe(true)
  })

  it('never reopens after a dismissal', () => {
    expect(shouldShowGarnek({
      comparisonRequested: true,
      evaluation: { status: 'no-overlap' },
      dismissed: true,
    })).toBe(false)
  })
})
