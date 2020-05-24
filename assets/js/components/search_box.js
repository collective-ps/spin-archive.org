import React from 'react'

import useInput from '../lib/use_input'

const Component = ({ query }) => {
  const { value, bind: bindQuery } = useInput(query)

  return (
    <form action='/' method='GET'>
      <input type='text' name='q' value={value} {...bindQuery}></input>
    </form>
  )
}

export default Component
