import React, { useCallback, useState } from 'react'

import TagInput from '../search_box'
import useInput from '../../lib/use_input'

import './index.css'

const Component = () => {
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState(null)
  const [tags, setTags] = useState('')
  const { value: tweetUrl, bind: bindTweetUrl } = useInput('')

  const onSubmit = useCallback(
    (ev) => {
      ev.preventDefault()
      setError(null)
      setIsSubmitting(true)

      fetch('/api/v1/uploads/twitter', {
        method: 'POST',
        body: JSON.stringify({
          url: tweetUrl,
          tags: tags,
        }),
        headers: {
          'Content-Type': 'application/json',
        },
      })
        .then((response) => response.json())
        .then((json) => {
          if (json.status && json.reason) {
            setError(json.reason)
          } else {
            window.location = json.url
          }
        })
        .catch(() => {
          setError('Server error, please try again!')
        })
        .finally(() => {
          setIsSubmitting(false)
        })
    },
    [tags, tweetUrl]
  )

  return (
    <div className='twitter-uploader-wrapper'>
      <div className='banner'>Upload from Twitter</div>

      {error && <div className='error-box'>{error}</div>}

      {isSubmitting && <div>Fetching tweet...</div>}

      {!isSubmitting && (
        <form onSubmit={onSubmit}>
          <fieldset>
            <label>
              Tweet URL
              <input
                type='text'
                id='tweet-url'
                name='tweet-url'
                required
                value={tweetUrl}
                {...bindTweetUrl}
              ></input>
            </label>
          </fieldset>
          <fieldset>
            <label>
              Tags (space-separated)
              <TagInput query='' onChange={(value) => setTags(value)} />
            </label>
          </fieldset>
          <button type='submit'>Upload from Twitter</button>
        </form>
      )}
    </div>
  )
}

export default Component
