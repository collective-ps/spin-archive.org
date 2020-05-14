import React from 'react'
import Dropzone from 'react-dropzone-uploader'
import 'react-dropzone-uploader/dist/styles.css'

const Uploader = () => {
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
    }).then((response) => response.json())

    console.log(id, url)

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

  const handleChangeStatus = ({ meta, file }, status) => {
    console.log(status, meta, file)
  }

  const handleSubmit = (files, allFiles) => {
    console.log(files)
  }

  return (
    <Dropzone
      getUploadParams={getUploadParams}
      onChangeStatus={handleChangeStatus}
      onSubmit={handleSubmit}
      accept='video/*'
    />
  )
}

export default Uploader
