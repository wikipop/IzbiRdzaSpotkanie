import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import AvailabilityGrid from '~/components/schedule/AvailabilityGrid.vue'
import type { SlotReadState } from '~/composables/useEventSchedule'
import type { SlotSummary } from '~/domain/availability'
import type { SlotId } from '~/types/api/contracts'

afterEach(() => {
  document.body.replaceChildren()
  vi.restoreAllMocks()
})

const summaries: SlotSummary[] = [
  {
    slot: 'pn-09',
    entries: [
      { user_id: 'u_1', nick: 'Iza', state: 'yes', updated_at: 1 },
      { user_id: 'u_2', nick: 'Ola', state: 'yes', updated_at: 1 },
      { user_id: 'u_3', nick: 'Ela', state: 'maybe', updated_at: 1 },
    ],
    yesCount: 2,
    maybeCount: 1,
  },
  {
    slot: 'pn-10',
    entries: [{ user_id: 'u_3', nick: 'Ela', state: 'maybe', updated_at: 2 }],
    yesCount: 0,
    maybeCount: 1,
  },
]
const participants = [
  { userId: 'u_1', nick: 'Iza' },
  { userId: 'u_2', nick: 'Ola' },
  { userId: 'u_3', nick: 'Ela' },
]

describe('AvailabilityGrid', () => {
  it('shows only the selected participant in counts and cell details', async () => {
    const selectedSummary: SlotSummary = {
      ...summaries[0]!,
      entries: [summaries[0]!.entries[0]!],
      yesCount: 1,
      maybeCount: 0,
    }
    const wrapper = mount(AvailabilityGrid, {
      props: {
        slots: ['pn-09'],
        reads: {
          'pn-09': { status: 'success', data: { slot: 'pn-09', entries: summaries[0]!.entries } },
        },
        summaries: [selectedSummary],
        participants: [participants[0]!],
        participantCount: 1,
        busy: false,
        editable: false,
        writeStates: {},
      },
    })

    const cell = wrapper.get('button[aria-label="Poniedziałek, 09:00–10:00: serwis online; 1 tak, 0 może"]')
    expect(cell.classes()).toContain('availability-level-4')
    await cell.trigger('click')

    expect(wrapper.text()).toContain('Tak · 1')
    expect(wrapper.text()).toContain('Iza')
    expect(wrapper.text()).toContain('Może · 0')
    expect(wrapper.text()).not.toContain('Ola')
    expect(wrapper.text()).not.toContain('Ela')
  })

  it('uses color depth instead of visible counts and emits failed-slot retries', async () => {
    const reads: Partial<Record<SlotId, SlotReadState>> = {
      'pn-09': {
        status: 'success',
        data: { slot: 'pn-09', entries: summaries[0]!.entries },
      },
      'pn-10': {
        status: 'success',
        data: { slot: 'pn-10', entries: summaries[1]!.entries },
      },
      'wt-10': {
        status: 'failed',
        error: { kind: 'network', message: 'offline' },
      },
    }

    const wrapper = mount(AvailabilityGrid, {
      props: {
        slots: ['pn-09', 'pn-10', 'wt-10'],
        reads,
        summaries,
        participants,
        participantCount: 3,
        busy: false,
        editable: false,
        writeStates: {},
      },
    })

    const popular = wrapper.get('[aria-label="Poniedziałek, 09:00–10:00: serwis online; 2 tak, 1 może"]')
    const sparse = wrapper.get('[aria-label="Poniedziałek, 10:00–11:00: serwis online; 0 tak, 1 może"]')
    expect(popular.classes()).toContain('availability-level-4')
    expect(sparse.classes()).toContain('availability-level-1')
    expect(popular.text()).not.toMatch(/[0-9]/)
    expect(popular.findAll('.bg-current')).toHaveLength(4)
    expect(sparse.findAll('.bg-current')).toHaveLength(1)
    expect(wrapper.findAll('[aria-label="Poza zakresem spotkania"]')).toHaveLength(1)

    await popular.trigger('click')
    expect(wrapper.text()).toContain('Tak · 2')
    expect(wrapper.text()).toContain('Iza, Ola')
    expect(wrapper.text()).toContain('Może · 1')

    const retry = wrapper.get('button[aria-label="Wtorek, 10:00–11:00: nie udało się odczytać danych z serwisu godziny, spróbuj ponownie"]')
    expect(retry.text()).toContain('Ponów')
    await retry.trigger('click')

    expect(wrapper.emitted('retry')).toEqual([['wt-10']])
  })

  it('cycles the current choice, exposes save state, and supports arrow navigation', async () => {
    const reads: Partial<Record<SlotId, SlotReadState>> = {
      'pn-09': { status: 'success', data: { slot: 'pn-09', entries: summaries[0]!.entries } },
      'pn-10': { status: 'success', data: { slot: 'pn-10', entries: summaries[1]!.entries } },
      'wt-10': { status: 'success', data: { slot: 'wt-10', entries: [] } },
    }
    const wrapper = mount(AvailabilityGrid, {
      attachTo: document.body,
      props: {
        slots: ['pn-09', 'pn-10', 'wt-10'],
        reads,
        summaries,
        participants,
        participantCount: 3,
        busy: false,
        editable: true,
        currentUserId: 'u_1',
        writeStates: {},
      },
    })

    const first = wrapper.get('button[data-slot="pn-09"]')
    expect(first.attributes('tabindex')).toBe('0')
    expect(first.attributes('aria-label')).toContain('Twój wybór: tak')
    await first.trigger('click')
    expect(wrapper.emitted('edit')).toEqual([['pn-09', 'maybe']])

    await first.trigger('keydown', { key: 'ArrowDown' })
    const mondayTen = wrapper.get('button[data-slot="pn-10"]')
    expect(document.activeElement).toBe(mondayTen.element)
    await mondayTen.trigger('keydown', { key: 'End' })
    expect(document.activeElement).toBe(wrapper.get('button[data-slot="wt-10"]').element)
    await wrapper.get('button[data-slot="wt-10"]').trigger('keydown', { key: 'Home' })
    expect(document.activeElement).toBe(mondayTen.element)

    await wrapper.setProps({ writeStates: { 'pn-09': { status: 'saving', desired: 'maybe', ownerUserId: 'u_1' } } })
    expect(wrapper.get('button[data-slot="pn-09"]').attributes('aria-label')).toContain('zapisywanie')

    await wrapper.setProps({ slots: ['wt-10'], summaries: [], writeStates: {} })
    expect(wrapper.get('button[data-slot="wt-10"]').attributes('tabindex')).toBe('0')
  })

  it('retries a failed cell without advancing its desired choice', async () => {
    const reads: Partial<Record<SlotId, SlotReadState>> = {
      'pn-09': { status: 'success', data: { slot: 'pn-09', entries: summaries[0]!.entries } },
    }
    const wrapper = mount(AvailabilityGrid, {
      props: {
        slots: ['pn-09'],
        reads,
        summaries: [summaries[0]!],
        participants,
        participantCount: 3,
        busy: false,
        editable: true,
        currentUserId: 'u_1',
        writeStates: {
          'pn-09': { status: 'failed', desired: 'maybe', ownerUserId: 'u_1', message: 'Nie udało się zapisać.' },
        },
      },
    })

    await wrapper.get('button[data-slot="pn-09"]').trigger('click')
    expect(wrapper.emitted('retryWrite')).toEqual([['pn-09']])
    expect(wrapper.emitted('edit')).toBeUndefined()
  })

  it('preserves focus while a failed read is retried', async () => {
    const failedReads: Partial<Record<SlotId, SlotReadState>> = {
      'pn-09': { status: 'failed', error: { kind: 'network', message: 'offline' } },
    }
    const wrapper = mount(AvailabilityGrid, {
      attachTo: document.body,
      props: {
        slots: ['pn-09'],
        reads: failedReads,
        summaries: [],
        participants: [],
        participantCount: 0,
        busy: false,
        editable: false,
        writeStates: {},
      },
    })

    const cell = wrapper.get('button[data-slot="pn-09"]')
    cell.element.focus()
    await wrapper.setProps({ reads: { 'pn-09': { status: 'loading' } } })

    expect(document.activeElement).toBe(wrapper.get('button[data-slot="pn-09"]').element)
  })

  it('paints one desired state across cells during a pointer drag', async () => {
    const reads: Partial<Record<SlotId, SlotReadState>> = {
      'pn-09': { status: 'success', data: { slot: 'pn-09', entries: summaries[0]!.entries } },
      'pn-10': { status: 'success', data: { slot: 'pn-10', entries: summaries[1]!.entries } },
    }
    const wrapper = mount(AvailabilityGrid, {
      attachTo: document.body,
      props: {
        slots: ['pn-09', 'pn-10'],
        reads,
        summaries,
        participants,
        participantCount: 3,
        busy: false,
        editable: true,
        currentUserId: 'u_1',
        writeStates: {},
      },
    })
    const first = wrapper.get('button[data-slot="pn-09"]')
    const second = wrapper.get('button[data-slot="pn-10"]')
    vi.spyOn(document, 'elementFromPoint').mockReturnValue(second.element)

    await first.trigger('pointerdown', { button: 0, pointerId: 7 })
    await wrapper.get('.schedule-scroll').trigger('pointermove', { pointerId: 7, clientX: 10, clientY: 10 })
    await wrapper.get('.schedule-scroll').trigger('pointerup', { pointerId: 7 })

    expect(wrapper.emitted('edit')).toEqual([
      ['pn-09', 'maybe'],
      ['pn-10', 'maybe'],
    ])
  })

  it('keeps touch panning available and changes a cell on tap', async () => {
    const reads: Partial<Record<SlotId, SlotReadState>> = {
      'pn-09': { status: 'success', data: { slot: 'pn-09', entries: summaries[0]!.entries } },
    }
    const wrapper = mount(AvailabilityGrid, {
      props: {
        slots: ['pn-09'],
        reads,
        summaries: [summaries[0]!],
        participants,
        participantCount: 3,
        busy: false,
        editable: true,
        currentUserId: 'u_1',
        writeStates: {},
      },
    })
    const cell = wrapper.get('button[data-slot="pn-09"]')

    await cell.trigger('pointerdown', { button: 0, pointerId: 4, pointerType: 'touch' })
    expect(wrapper.emitted('edit')).toBeUndefined()
    await cell.trigger('click', { detail: 1 })
    expect(wrapper.emitted('edit')).toEqual([['pn-09', 'maybe']])
  })
})
