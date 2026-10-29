import { describe, expect, it } from 'vitest'
import { rememberGarnekDismissal, shouldShowGarnek } from '~/domain/garnek'

describe('Garnek dialogue frequency', () => {
  it('keeps every dismissed result hidden when comparison changes A → B → A', () => {
    let dismissed: string[] = []
    dismissed = rememberGarnekDismissal(dismissed, 'result-a')
    dismissed = rememberGarnekDismissal(dismissed, 'result-b')

    expect(shouldShowGarnek({
      comparisonRequested: true,
      evaluation: { status: 'no-overlap', signature: 'result-a' },
      dismissedSignatures: dismissed,
    })).toBe(false)
    expect(dismissed).toEqual(['result-a', 'result-b'])
  })

  it('requires an armed comparison and never reopens after any dismissal', () => {
    expect(shouldShowGarnek({
      comparisonRequested: false,
      evaluation: { status: 'no-overlap', signature: 'new-result' },
      dismissedSignatures: [],
    })).toBe(false)
    expect(shouldShowGarnek({
      comparisonRequested: true,
      evaluation: { status: 'no-overlap', signature: 'new-result' },
      dismissedSignatures: [],
    })).toBe(true)
    expect(shouldShowGarnek({
      comparisonRequested: true,
      evaluation: { status: 'no-overlap', signature: 'different-result' },
      dismissedSignatures: ['old-result'],
    })).toBe(false)
  })
})
