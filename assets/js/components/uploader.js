import React from 'react'
import Dropzone from 'react-dropzone-uploader'
import MD5Hasher from 'browser-md5-file'
import 'react-dropzone-uploader/dist/styles.css'

const getMD5Hash = (file) =>
  new Promise((resolve, reject) => {
    const hasher = new MD5Hasher()
    hasher.md5(
      file,
      (err, md5) => {
        if (err || !md5) {
          reject(err)
        } else {
          resolve(md5)
        }
      },
      () => {}
    )
  })

const Uploader = ({ handleSubmit, uploadLimit }) => {
  const hasUploadLimit = uploadLimit !== null

  const getUploadParams = async (fileWithMeta) => {
    try {
      const md5 = await getMD5Hash(fileWithMeta.file)

      const response = await fetch('/upload', {
        method: 'POST',
        body: JSON.stringify({
          file_name: fileWithMeta.meta.name,
          content_length: fileWithMeta.file.size,
          md5_hash: md5,
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
    } catch (error) {
      fileWithMeta.meta.error = error
      return {}
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

  const props = hasUploadLimit
    ? {
        maxFiles: uploadLimit,
        multiple: uploadLimit > 1,
      }
    : {
        maxFiles: 25,
        multiple: true,
      }

  return (
    <Dropzone
      getUploadParams={getUploadParams}
      onChangeStatus={handleChangeStatus}
      onSubmit={handleSubmit}
      submitButtonContent={'Continue'}
      accept='video/*'
      {...props}
    />
  )
}

export default Uploader
