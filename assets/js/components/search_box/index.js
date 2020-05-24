import React, { useState, useRef, useCallback, useEffect } from 'react'
import SVG from 'react-inlinesvg'

import './index.css'
import searchIcon from '../../../svg/icon-search.svg'

const TAG_REGEX = /([-~]*)?([a-z\:]*)$/i

const parseQuery = (text, caretIdx) => {
  let beforeCaretText = text.substring(0, caretIdx)
  let match = beforeCaretText.match(TAG_REGEX)

  let operator = match[1]
  let tagQuery = match[2] ? match[2].toLowerCase() : ''

  return {
    operator,
    tagQuery,
  }
}

const fetchTagSuggestions = (query) => {
  const sanitizedQuery = encodeURI(query.trim())
  const url = `/api/v1/tags/suggestions?q=${sanitizedQuery}`

  return fetch(url, {
    headers: {
      'Content-Type': 'application/json',
    },
  }).then((response) => response.json())
}

const insertSuggestion = (completion, inputRef) => {
  var beforeCaretText = inputRef.current.value
    .substring(0, inputRef.current.selectionStart)
    .replace(/^[ \t]+|[ \t]+$/gm, '')

  var afterCaretText = inputRef.current.value
    .substring(inputRef.current.selectionStart)
    .replace(/^[ \t]+|[ \t]+$/gm, '')

  beforeCaretText = beforeCaretText.replace(TAG_REGEX, '$1') + completion + ' '

  inputRef.current.value = beforeCaretText + afterCaretText
  inputRef.current.selectionStart = inputRef.selectionEnd =
    beforeCaretText.length
}

const Component = ({ query }) => {
  const [suggestions, setSuggestions] = useState([])
  const [isFetching, setFetching] = useState(false)
  const [isFocused, setIsFocused] = useState(false)
  const [suggestionIndex, setSuggestionIndex] = useState(0)
  const inputRef = useRef(null)

  const onChange = useCallback(
    (ev) => {
      if (isFetching) {
        return
      }

      let parsedQuery = parseQuery(
        ev.target.value,
        inputRef.current.selectionStart
      )

      setFetching(true)

      fetchTagSuggestions(parsedQuery.tagQuery)
        .then((data) => {
          setSuggestionIndex(0)
          setSuggestions(data.tags)
        })
        .finally(() => {
          setFetching(false)
        })
    },
    [inputRef, isFetching]
  )

  const onFocus = () => {
    setIsFocused(true)
  }

  const onBlur = () => {
    setIsFocused(false)
  }

  const onKeyDown = useCallback(
    (ev) => {
      if (ev.key === 'Enter') {
        let suggestedTag = suggestions[suggestionIndex]

        if (suggestedTag) {
          ev.preventDefault()
          insertSuggestion(suggestedTag.name, inputRef)
          setSuggestions([])
        }
      }

      if (ev.key == 'ArrowUp') {
        let newIndex = suggestionIndex - 1 < 0 ? 0 : suggestionIndex - 1
        setSuggestionIndex(newIndex)
        ev.preventDefault()
      } else if (ev.key == 'ArrowDown' || ev.key == 'Tab') {
        let newIndex =
          suggestionIndex + 1 > suggestions.length
            ? suggestions.length
            : suggestionIndex + 1

        ev.preventDefault()
        setSuggestionIndex(newIndex)
      }
    },
    [suggestionIndex, suggestions]
  )

  const onSubmit = useCallback(
    (ev) => {
      ev.preventDefault()

      const query = encodeURI(inputRef.current.value.trim())

      window.location = `/?q=${query}`
    },
    [inputRef]
  )

  const clickedSuggestion = (idx) => {
    let suggestedTag = suggestions[idx]

    if (suggestedTag) {
      insertSuggestion(suggestedTag.name, inputRef)
      setSuggestions([])
    }
  }

  return (
    <form action='/' method='GET' onSubmit={onSubmit}>
      <div className='search'>
        <input
          ref={inputRef}
          type='text'
          className='search-input'
          name='q'
          onChange={onChange}
          onFocus={onFocus}
          onBlur={onBlur}
          onKeyDown={onKeyDown}
          autoComplete='off'
          defaultValue={query}
        ></input>

        <button type='submit' className='search-btn'>
          <SVG src={searchIcon} className='icon' />
        </button>
      </div>

      <div class={`suggestions ${isFocused ? 'visible' : ''}`}>
        {suggestions.map((tag, idx) => {
          const classNames = [
            'suggestion',
            idx == suggestionIndex ? 'selected' : '',
          ].join(' ')

          return (
            <div
              key={tag.name}
              className={classNames}
              onMouseDown={() => clickedSuggestion(idx)}
            >
              {tag.name} <small>({tag.upload_count})</small>
            </div>
          )
        })}
      </div>
    </form>
  )
}

export default Component
