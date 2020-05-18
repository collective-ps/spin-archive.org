import React from 'react'
import Dropzone from 'react-dropzone-uploader'
import 'react-dropzone-uploader/dist/styles.css'

const Uploader = ({ handleSubmit }) => {
  const getUploadParams = async ({ file, meta: { name } }) => {
    const { id, url } = await fetch('/upload', {
      method: 'POST',
      body: JSON.stringify({
        file_name: name,
        content_length: file.size,
      }),
      headers: {
        'Content-Type': 'application/json',
      },
    })
      .then((response) => response.json())
      .catch(() => {
        return { id: null, url: null }
      })

    return {
      body: file,
      method: 'PUT',
      headers: {
        'x-amz-acl': 'public-read',
        'x-amz-content-sha256': 'UNSIGNED-PAYLOAD',
      },
      meta: {
        file_id: id,
      },
      url,
    }
  }

  const handleChangeStatus = ({ meta, file }, status) => {}

  return (
    <Dropzone
      getUploadParams={getUploadParams}
      onChangeStatus={handleChangeStatus}
      onSubmit={handleSubmit}
      submitButtonContent={'Continue'}
      accept='video/*'
    />
  )
}

export default Uploader
