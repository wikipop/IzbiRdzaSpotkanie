import type { OverlapEvaluation } from './availability'

export function shouldShowGarnek(options: {
  comparisonRequested: boolean
  evaluation: OverlapEvaluation
  dismissedSignatures: string[]
}): boolean {
  return options.comparisonRequested
    && options.evaluation.status === 'no-overlap'
    && options.dismissedSignatures.length === 0
}

export function rememberGarnekDismissal(dismissedSignatures: string[], signature: string): string[] {
  return dismissedSignatures.includes(signature)
    ? dismissedSignatures
    : [...dismissedSignatures, signature]
}
