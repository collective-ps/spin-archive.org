#!/bin/sh
# Encodes one spin-archive upload, then exits (the Machine auto-destroys).
#
# Downloads INPUT_URL, writes an H.264 MP4 and a JPG thumbnail to the presigned
# PUT URLs, and reports the result to WEBHOOK_URL in the same shape the app's
# /webhooks/video endpoint expects: {"job_id", "event", "data"}.
set -u

: "${INPUT_URL:?}" "${VIDEO_PUT_URL:?}" "${THUMBNAIL_PUT_URL:?}" "${WEBHOOK_URL:?}"

JOB_ID="${FLY_MACHINE_ID:-local}"
WORK=$(mktemp -d)
LOG="$WORK/log.txt"
touch "$LOG"

notify() {
  payload=$(jq -cn --arg job "$JOB_ID" --arg event "$1" --argjson data "$2" \
    '{job_id: $job, event: $event, data: $data}')

  curl -fsS --retry 8 --retry-delay 5 --retry-connrefused \
    -H 'Content-Type: application/json' -d "$payload" "$WEBHOOK_URL" >/dev/null \
    || echo "failed to deliver $1 webhook" >&2
}

fail() {
  echo "encode failed: $1" >&2
  tail -c 2000 "$LOG" >&2
  notify job.failed "$(jq -cn --arg error "$1" --arg log "$(tail -c 2000 "$LOG")" \
    '{error: $error, log: $log}')"
  exit 1
}

cd "$WORK"

curl -fsSL --retry 5 -o input "$INPUT_URL" >>"$LOG" 2>&1 \
  || fail "download failed"

WIDTH=$(ffprobe -v error -select_streams v:0 -show_entries stream=width   -of default=noprint_wrappers=1:nokey=1 input 2>>"$LOG" | head -n 1 | tr -dc '0-9')
[ -n "$WIDTH" ] || fail "no video stream found"

# Output height by source width (matches the previous Coconut outputs), never upscaling.
if [ "$WIDTH" -lt 1280 ]; then
  HEIGHT=480
elif [ "$WIDTH" -lt 1980 ]; then
  HEIGHT=720
else
  HEIGHT=1080
fi

timeout 3h ffmpeg -hide_banner -nostdin -y -i input \
  -map 0:v:0 -map '0:a:0?' \
  -vf "yadif=deint=interlaced,scale=-2:'trunc(min($HEIGHT,ih)/2)*2',format=yuv420p" \
  -c:v libx264 -preset veryfast -crf 23 -profile:v high \
  -c:a aac -b:a 128k -ac 2 \
  -max_muxing_queue_size 4096 -movflags +faststart \
  video.mp4 >>"$LOG" 2>&1 \
  || fail "ffmpeg encode failed"

# Thumbnail from a quarter of the way in, falling back to the first frame.
DURATION=$(ffprobe -v error -show_entries format=duration   -of default=noprint_wrappers=1:nokey=1 input 2>/dev/null | head -n 1 | tr -dc '0-9.')
OFFSET=$(echo "$DURATION" | awk '{ if ($1 + 0 > 0) printf "%.2f", $1 / 4; else print 0 }')

ffmpeg -hide_banner -nostdin -y -ss "$OFFSET" -i video.mp4 -frames:v 1 \
  -vf scale=300:-2 -q:v 3 thumbnail.jpg >>"$LOG" 2>&1 \
  || ffmpeg -hide_banner -nostdin -y -i video.mp4 -frames:v 1 \
    -vf scale=300:-2 -q:v 3 thumbnail.jpg >>"$LOG" 2>&1 \
  || fail "thumbnail failed"

curl -fsS --retry 5 -X PUT -H 'Content-Type: video/mp4' \
  --upload-file video.mp4 "$VIDEO_PUT_URL" >>"$LOG" 2>&1 \
  || fail "video upload failed"

curl -fsS --retry 5 -X PUT -H 'Content-Type: image/jpeg' \
  --upload-file thumbnail.jpg "$THUMBNAIL_PUT_URL" >>"$LOG" 2>&1 \
  || fail "thumbnail upload failed"

notify job.completed "$(jq -cn --argjson height "$HEIGHT" --argjson width "$WIDTH" \
  '{source_width: $width, height: $height}')"

echo "encoded ${WIDTH}px source to ${HEIGHT}p"
