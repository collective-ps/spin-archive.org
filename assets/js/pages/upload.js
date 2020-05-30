import React, { useState, useEffect } from 'react'
import Uploader from '../components/uploader'
import PromisePool from 'es6-promise-pool'

import './upload.css'

import useInput from '../lib/use_input'
import TagInput from '../components/search_box'

const File = ({ file }) => {
  const [tags, setTags] = useState('')
  const { value: source, bind: bindSource } = useInput('')
  const { value: description, bind: bindDescription } = useInput('')
  const { value: originalUploadDate, bind: bindOriginalUploadDate } = useInput(
    ''
  )

  useEffect(() => {
    file.meta.tags = tags
    file.meta.source = source
    file.meta.description = description
    file.meta.originalUploadDate = originalUploadDate
  }, [file, tags, source, description, originalUploadDate])

  return (
    <div className='file'>
      <label>{file.meta.name}</label>

      <fieldset>
        <label>
          Tags (space-separated)
          <TagInput query='' onChange={(value) => setTags(value)} />
        </label>
      </fieldset>
      <fieldset>
        <label>
          Source (URL)
          <input type='text' value={source} {...bindSource} />
        </label>
      </fieldset>
      <fieldset>
        <label>
          Original Upload Date{' '}
          <small>
            (if uncertain, use first day of month / year approximately)
          </small>
          <input
            type='date'
            value={originalUploadDate}
            {...bindOriginalUploadDate}
          />
        </label>
      </fieldset>
      <fieldset>
        <label>
          Description
          <textarea value={description} {...bindDescription} />
        </label>
      </fieldset>
    </div>
  )
}

const EditComponent = ({ files, next }) => {
  let fileComponents = files.map((file) => {
    return <File key={file.meta.file_id} file={file} />
  })

  const onClick = () => {
    next(files)
  }

  return (
    <div className='edit-component'>
      <div className='file-grid'>{fileComponents}</div>
      <button onClick={onClick}>Publish</button>
    </div>
  )
}

const STATE = {
  upload: 'upload',
  edit: 'edit',
  done: 'done',
  error: 'error',
}

const UploadPage = ({ uploadLimit }) => {
  const [files, setFiles] = useState([])
  const [state, setState] = useState(STATE.upload)
  const hasUploadLimit = uploadLimit !== null

  const handleAfterUpload = (files) => {
    setFiles(files)
    setState(STATE.edit)
  }

  const handleDone = (files) => {
    let file_idx = 0
    let file_count = files.length

    let promiseProducer = function () {
      if (file_idx < files.length) {
        const file = files[file_idx]
        const url = `/upload/${file.meta.file_id}/finalize`

        file_idx += 1

        return fetch(url, {
          method: 'POST',
          body: JSON.stringify({
            tags: file.meta.tags,
            source: file.meta.source,
            description: file.meta.description,
            original_upload_date: file.meta.originalUploadDate,
          }),
          headers: {
            'Content-Type': 'application/json',
          },
        }).then((response) => response.json())
      } else {
        return null
      }
    }

    let concurrency = 3
    let pool = new PromisePool(promiseProducer, concurrency)
    let poolPromise = pool.start()

    poolPromise.then(
      function () {
        setState(STATE.done)
      },
      function (err) {
        console.error(err)
        setState(STATE.error)
      }
    )
  }

  const reset = () => {
    setFiles([])
    setState(STATE.upload)
  }

  if (hasUploadLimit && uploadLimit === 0) {
    return (
      <div>
        <div>Sorry, you have reached your limit of daily uploads.</div>
        <div>
          This limit is set in place to help us monitor the site and keep it
          clean. If you have more content that you wish to share, contact us in
          the{' '}
          <a
            rel='noopener noreferrer'
            target='_blank'
            href='https://discord.gg/NMMWPnA'
          >
            Discord
          </a>
          .
        </div>
      </div>
    )
  } else if (state == STATE.upload) {
    return (
      <div>
        {hasUploadLimit && (
          <div>
            <div>You have {uploadLimit} upload(s) left for the day.</div>
            <div>
              This limit is set in place to help us monitor the site and keep it
              clean. If you have more content that you wish to share, contact us
              in the{' '}
              <a
                rel='noopener noreferrer'
                target='_blank'
                href='https://discord.gg/NMMWPnA'
              >
                Discord
              </a>
              .
            </div>
            <div>
              It's recommended to read the #contributor-guide in the Discord
              before uploading.
            </div>
          </div>
        )}
        <Uploader handleSubmit={handleAfterUpload} uploadLimit={uploadLimit} />
      </div>
    )
  } else if (state == STATE.edit) {
    return <EditComponent files={files} next={handleDone} />
  } else if (state == STATE.done) {
    return (
      <div>
        <div>Videos are now processing and will be sent for approval.</div>
        <button onClick={reset}>Upload more</button>
      </div>
    )
  } else if (state == STATE.error) {
    return (
      <div>
        <div>
          An unknown error occured. Any successfully uploads will still be
          processed.
        </div>
        <button onClick={reset}>Retry</button>
      </div>
    )
  } else {
    return <div></div>
  }
}

export default UploadPage
