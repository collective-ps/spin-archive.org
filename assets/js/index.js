import '../css/reset.css'
import 'wingcss'
import '../css/index.css'
import '../css/login.css'

import React from 'react'
import ReactDOM from 'react-dom'
import Plyr from 'plyr'

import UploadPage from './pages/upload'

if (document.getElementById('upload-page')) {
  let page = document.getElementById('upload-page')
  ReactDOM.render(<UploadPage />, page)
}

window.addEventListener('DOMContentLoaded', () => {
  if (document.getElementById('video-player')) {
    const player = new Plyr('#video-player')
  }
})
