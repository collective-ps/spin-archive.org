import React from 'react'
import Dropzone from 'react-dropzone-uploader'
import 'react-dropzone-uploader/dist/styles.css'

// Converts a canvas's dataURL output to a Blob format for uploading.
const dataURLtoBlob = (dataurl) => {
  let arr = dataurl.split(','),
    mime = arr[0].match(/:(.*?);/)[1],
    bstr = atob(arr[1]),
    n = bstr.length,
    u8arr = new Uint8Array(n)
  while (n--) {
    u8arr[n] = bstr.charCodeAt(n)
  }

  return new Blob([u8arr], { type: mime })
}

// Generates a .jpg thumbnail blob from a video file. Only the first frame.
const generateThumbnailFromFile = async (file) => {
  let fileURL = URL.createObjectURL(file)
  let video = document.createElement('video')

  return new Promise((resolve, reject) => {
    var timeupdate = function () {
      let thumbnail = snapImage()
      video.removeEventListener('timeupdate', timeupdate)
      video.pause()
      resolve(thumbnail)
    }

    video.addEventListener('loadeddata', function () {
      let thumbnail = snapImage()
      video.removeEventListener('timeupdate', timeupdate)
      resolve(thumbnail)
    })

    var snapImage = function () {
      var canvas = document.createElement('canvas')
      canvas.width = video.videoWidth
      canvas.height = video.videoHeight
      canvas
        .getContext('2d')
        .drawImage(video, 0, 0, canvas.width, canvas.height)

      var image = dataURLtoBlob(canvas.toDataURL('image/jpeg', 0.5))

      URL.revokeObjectURL(fileURL)

      return image
    }

    video.addEventListener('timeupdate', timeupdate)

    video.preload = 'metadata'
    video.src = fileURL
    video.muted = true
    video.playsInline = true
    video.play().catch((err) => {
      reject(err)
    })
  })
}

const Uploader = ({ handleSubmit }) => {
  const getUploadParams = async ({ file, meta: { name } }) => {
    let thumbnail = null

    if (file.type.includes('video/')) {
      try {
        thumbnail = await generateThumbnailFromFile(file)
      } catch (err) {
        console.error(err)
      }
    }

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

    return {
      body: file,
      method: 'PUT',
      headers: {
        'x-amz-acl': 'public-read',
        'x-amz-content-sha256': 'UNSIGNED-PAYLOAD',
      },
      meta: {
        file_id: id,
        thumbnail,
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
