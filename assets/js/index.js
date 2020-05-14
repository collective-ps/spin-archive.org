import '../css/reset.css'
import 'wingcss'
import '../css/index.css'

import React from 'react'
import ReactDOM from 'react-dom'

import Uploader from './components/uploader'

if (document.getElementById('uploader')) {
  let uploader = document.getElementById('uploader')

  ReactDOM.render(<Uploader />, uploader)
}
