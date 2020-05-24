import React, { useState, useCallback } from 'react'
import SVG from 'react-inlinesvg'

import searchIcon from '../../../svg/icon-search.svg'
import InnerComponent from './index'

const Component = ({ query }) => {
  const [value, setValue] = useState(query)

  const onChange = (newValue) => {
    setValue(newValue)
  }

  const onSubmit = useCallback(
    (ev) => {
      ev.preventDefault()

      const query = encodeURI(value.trim())

      window.location = `/?q=${query}`
    },
    [value]
  )

  return (
    <form action='/' method='GET' onSubmit={onSubmit}>
      <InnerComponent query={query} onChange={onChange}>
        <button type='submit' className='search-btn'>
          <SVG src={searchIcon} className='icon' />
        </button>
      </InnerComponent>
    </form>
  )
}

export default Component
