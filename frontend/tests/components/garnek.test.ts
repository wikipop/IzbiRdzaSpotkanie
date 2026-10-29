import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it } from 'vitest'
import GarnekDialog from '~/components/garnek/Dialog.vue'

afterEach(() => {
  document.body.replaceChildren()
})

describe('GarnekDialog', () => {
  it('pleads for a shared time and returns the user to editing', async () => {
    const wrapper = mount(GarnekDialog, {
      attachTo: document.body,
      props: {
        visible: true,
        participantNames: ['Ala', 'Ola'],
      },
    })

    const dialog = wrapper.get('dialog')
    expect((dialog.element as HTMLDialogElement).open).toBe(true)
    expect(wrapper.text()).toContain('Jest mi smutno, że nie wszyscy mogą być razem')
    expect(wrapper.text()).toContain('Ala, Ola')
    expect(wrapper.get('img').attributes('src')).toBe('/assets/garnek/garnek-sad.png')

    await wrapper.get('button.button-primary').trigger('click')
    expect(wrapper.emitted('returnToGrid')).toEqual([[]])
  })

})
