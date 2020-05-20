import '../css/reset.css'
import 'wingcss'
import '../css/index.css'
import '../css/login.css'

import React from 'react'
import ReactDOM from 'react-dom'

import UploadPage from './pages/upload'

if (document.getElementById('upload-page')) {
  let page = document.getElementById('upload-page')
  ReactDOM.render(<UploadPage />, page)
}
