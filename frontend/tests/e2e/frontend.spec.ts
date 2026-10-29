import AxeBuilder from '@axe-core/playwright'
import { expect, test, type Page } from '@playwright/test'

async function expectNoSeriousA11yViolations(page: Page) {
  const result = await new AxeBuilder({ page }).analyze()
  expect(result.violations.filter(violation => ['critical', 'serious'].includes(violation.impact ?? ''))).toEqual([])
}

test.beforeEach(async ({ request }) => {
  await request.post('http://127.0.0.1:8080/__reset')
})

test('creates an event with explicit non-contiguous slots and passes axe', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('heading', { name: 'Kiedy się widzimy?' })).toBeVisible()
  await expectNoSeriousA11yViolations(page)

  await page.getByLabel('Nazwa spotkania').fill('Wspólne izbienie')
  await page.getByRole('button', { name: 'Konkretne godziny' }).click()
  for (const day of ['Poniedziałek', 'Wtorek', 'Środa', 'Czwartek', 'Piątek']) {
    await page.getByRole('button', { name: `Wybierz cały dzień: ${day}` }).click()
    await page.getByRole('button', { name: `Wyczyść cały dzień: ${day}` }).click()
  }
  await page.locator('#slot-pn-09').click()
  await page.locator('#slot-sr-14').click()
  await page.getByRole('button', { name: 'Utwórz spotkanie' }).click()
  await expect(page).toHaveURL(/\/e\/e_e2e$/)
  await expect(page.getByRole('heading', { name: 'Wspólne izbienie' })).toBeVisible()
})

test('supports keyboard editing, automatic one-time Garnek check and session reload', async ({ page }) => {
  await page.goto('/e/e_e2e')
  await expect(page.getByText('2 online')).toBeVisible()
  await page.getByRole('button', { name: 'Dodaj swoją dostępność' }).click()
  await page.getByLabel('Pseudonim').fill('Iza')
  await page.getByRole('button', { name: 'Przejdź do zaznaczania' }).click()

  await expect(page.getByRole('button', { name: /Iza.*Ty/ })).toHaveAttribute('aria-pressed', 'true')
  await expect(page.getByRole('button', { name: 'Ola' })).toHaveAttribute('aria-pressed', 'false')

  const firstCell = page.locator('button[data-slot="pn-09"]')
  await firstCell.focus()
  await page.keyboard.press('Enter')
  await expect(firstCell).toHaveAttribute('aria-label', /Twój wybór: tak/)
  await page.keyboard.press('ArrowDown')
  await expect(page.locator('button[data-slot="pn-10"]')).toBeFocused()

  await expect(page.getByRole('dialog')).toContainText('Jest mi smutno, że nie wszyscy mogą być razem')
  await page.getByRole('button', { name: 'Zamknij wypowiedź Garnka' }).click()

  await page.reload()
  await expect(page.getByText(/Edytujesz jako/)).toContainText('Iza')
  await expect(page.getByRole('button', { name: /Iza.*Ty/ })).toHaveAttribute('aria-pressed', 'true')
  await expect(page.getByRole('button', { name: 'Ola' })).toHaveAttribute('aria-pressed', 'false')
  await page.waitForTimeout(2_500)
  await expect(page.getByRole('dialog')).toHaveCount(0)
  expect(await page.content()).not.toContain('e2e-u_iza-token')
  await expectNoSeriousA11yViolations(page)
})

test('keeps mobile schedule usable with reduced motion', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' })
  await page.goto('/e/e_e2e')
  await expect(page.getByText('2 online')).toBeVisible()
  const slot = page.locator('button[data-slot="pn-10"]')
  if (await page.evaluate(() => matchMedia('(pointer: coarse)').matches)) await slot.tap()
  else await slot.click()
  await expect(page.getByRole('heading', { name: /Poniedziałek, 10:00/ })).toBeVisible()
  expect(await page.evaluate(() => matchMedia('(prefers-reduced-motion: reduce)').matches)).toBe(true)
})
