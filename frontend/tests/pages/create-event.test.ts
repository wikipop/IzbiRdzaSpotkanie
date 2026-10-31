import { flushPromises, mount } from '@vue/test-utils'
import { mockNuxtImport } from '@nuxt/test-utils/runtime'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import CreateEventPage from '~/pages/index.vue'

const { createEventMock, navigateToMock, useApiMock } = vi.hoisted(() => ({
  createEventMock: vi.fn(),
  navigateToMock: vi.fn(),
  useApiMock: vi.fn(),
}))

mockNuxtImport('useApi', () => useApiMock)
mockNuxtImport('navigateTo', () => navigateToMock)
mockNuxtImport('useHead', () => vi.fn())

const defaultDays = ['pn', 'wt', 'sr', 'cz', 'pt'] as const

async function mountPage() {
  return mount(CreateEventPage, { attachTo: document.body })
}

async function clearDefaultDays(wrapper: Awaited<ReturnType<typeof mountPage>>): Promise<void> {
  for (const day of defaultDays) await wrapper.get(`#day-${day}`).trigger('click')
}

beforeEach(() => {
  createEventMock.mockResolvedValue({ ok: true, data: { event_id: 'e_created' } })
  useApiMock.mockReturnValue({ gateway: { createEvent: createEventMock } })
})

afterEach(() => {
  createEventMock.mockReset()
  navigateToMock.mockReset()
  useApiMock.mockReset()
  document.body.replaceChildren()
})

describe('create event page', () => {
  it('submits the simple range mode to the gateway', async () => {
    const wrapper = await mountPage()
    await wrapper.get('#event-name').setValue('Planszówki')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(createEventMock).toHaveBeenCalledOnce()
    expect(createEventMock).toHaveBeenCalledWith({
      name: 'Planszówki',
      days: ['pn', 'wt', 'sr', 'cz', 'pt'],
      hours: { from: 9, to: 17 },
    })
    expect(navigateToMock).toHaveBeenCalledWith('/e/e_created')
  })

  it('rejects an empty selection in specific-hours mode', async () => {
    const wrapper = await mountPage()
    await wrapper.get('#event-name').setValue('Spacer')
    await clearDefaultDays(wrapper)
    await wrapper.findAll('button').find(button => button.text() === 'Konkretne godziny')!.trigger('click')

    expect(wrapper.text()).toContain('Wybrano: 0')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(wrapper.get('#event-slots-error').text()).toBe('Wybierz co najmniej jedną godzinę.')
    expect(document.activeElement).toBe(wrapper.get('#slot-pn-00').element)
    expect(createEventMock).not.toHaveBeenCalled()
  })

  it('submits exactly the selected non-contiguous slots', async () => {
    const wrapper = await mountPage()
    await wrapper.get('#event-name').setValue('Kawa')
    await clearDefaultDays(wrapper)
    await wrapper.findAll('button').find(button => button.text() === 'Konkretne godziny')!.trigger('click')

    await wrapper.get('#slot-pn-09').trigger('click')
    await wrapper.get('#slot-sr-14').trigger('click')
    expect(wrapper.text()).toContain('Wybrano: 2')

    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(createEventMock).toHaveBeenCalledOnce()
    expect(createEventMock).toHaveBeenCalledWith({
      name: 'Kawa',
      slots: ['pn-09', 'sr-14'],
    })
    expect(navigateToMock).toHaveBeenCalledWith('/e/e_created')
  })

  it('uses one keyboard tab stop and arrow navigation in the explicit grid', async () => {
    const wrapper = await mountPage()
    await wrapper.findAll('button').find(button => button.text() === 'Konkretne godziny')!.trigger('click')

    const slotButtons = wrapper.findAll('button[id^="slot-"]')
    expect(slotButtons.filter(button => button.attributes('tabindex') === '0')).toHaveLength(1)
    const first = wrapper.get('#slot-pn-00')
    first.element.focus()
    await first.trigger('keydown', { key: 'ArrowDown' })

    expect(document.activeElement).toBe(wrapper.get('#slot-pn-01').element)
    expect(wrapper.get('#slot-pn-01').attributes('tabindex')).toBe('0')

    await wrapper.get('#slot-pn-01').trigger('keydown', { key: 'ArrowRight' })
    expect(document.activeElement).toBe(wrapper.get('#slot-wt-01').element)
  })
})
