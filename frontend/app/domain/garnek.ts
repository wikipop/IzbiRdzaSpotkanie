import type { OverlapEvaluation } from './availability'

export function shouldShowGarnek(options: {
  comparisonRequested: boolean
  evaluation: OverlapEvaluation
  dismissed: boolean
}): boolean {
  return options.comparisonRequested
    && options.evaluation.status === 'no-overlap'
    && !options.dismissed
}
