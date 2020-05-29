import React from 'react'

import './index.css'

const Component = ({
  tags,
  uploader,
  commentCount,
  humanizedPast,
  uploaderRole,
  originalFileName,
}) => (
  <div className='content-wrapper'>
    <div className='original-file-name'>{originalFileName}</div>
    <div className='header'>
      <div className='left'>
        <div className='uploader'>
          {uploader}{' '}
          <span className='role' data-role={`${uploaderRole.toLowerCase()}`}>
            [{uploaderRole}]
          </span>
        </div>
      </div>
      <div className='right'>
        <div className='humanized-past'>{humanizedPast}</div>
        <div className='comments'>{commentCount} comments</div>
      </div>
    </div>
    <div className='tags'>{tags}</div>
  </div>
)

export default Component
