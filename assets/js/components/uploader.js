import React from 'react'
import Dropzone from 'react-dropzone-uploader'
import 'react-dropzone-uploader/dist/styles.css'

const Uploader = ({ handleSubmit }) => {
  const getUploadParams = async (fileWithMeta) => {
    const response = await fetch('/upload', {
      method: 'POST',
      body: JSON.stringify({
        file_name: fileWithMeta.meta.name,
        content_length: fileWithMeta.file.size,
      }),
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (!response.ok) {
      return {}
    }

    const json = await response.json()

    if (json['status'] == 'error') {
      fileWithMeta.meta.error = json['reason']
      return {}
    }

    const { id, url } = json

    return {
      body: fileWithMeta.file,
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

  const handleChangeStatus = (data, status) => {
    if (status == 'error_upload_params') {
      if (data.meta.error) {
        data.meta.status = 'error_validation'
        data.meta.validationError = data.meta.error
      }
    }
  }

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
