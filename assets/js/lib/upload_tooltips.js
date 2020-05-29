import React from 'react'
import ReactDOM from 'react-dom'
import tippy from 'tippy.js/headless'

import TooltipComponent from '../components/tooltip'

let $indexPage = document.getElementById('index-page')

if ($indexPage) {
  tippy('.upload', {
    animation: true,
    placement: 'top-end',
    interactive: true,
    offset: [0, 0],
    appendTo: document.body,
    onHide: (instance) => {
      requestAnimationFrame(instance.unmount)
    },
    render(instance) {
      const parent = instance.reference
      const tags = parent.dataset.tags || ''
      const uploader = parent.dataset.uploader || ''
      const commentCount = parent.dataset.commentCount || ''
      const humanizedPast = parent.dataset.humanizedPast || ''
      const uploaderRole = parent.dataset.uploaderRole || ''
      const originalFileName = parent.dataset.originalFileName || ''

      const props = {
        tags,
        uploader,
        commentCount,
        humanizedPast,
        uploaderRole,
        originalFileName,
      }

      // The recommended structure is to use the popper as an outer wrapper
      // element, with an inner `box` element
      const popper = document.createElement('div')
      const box = document.createElement('div')

      popper.appendChild(box)

      box.className = 'sa-tooltip'
      box.textContent = instance.props.content

      ReactDOM.render(<TooltipComponent {...props} />, box)

      // Return an object with two properties:
      // - `popper` (the root popper element)
      // - `onUpdate` callback whenever .setProps() or .setContent() is called
      return {
        popper,
      }
    },
  })
}
