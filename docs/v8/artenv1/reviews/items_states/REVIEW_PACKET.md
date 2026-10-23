# Item quantity and state observations

Open only `blind_states_32px.png`. It contains 67 opaque IDs. Inspect it at its native 800×820 pixels: each item fits within 32×32 pixels, with no enlargement. Do not consult source files, asset names, manifests, answer keys, or other reviewers.

For every ID, record:

1. Your primary name for the depicted item.
2. The depicted quantity bucket: `1`, `2–8`, `9+`, or `unclear`.
3. Any visible condition, material treatment, or container contents; use `not apparent` when you cannot tell.
4. Confidence and any ambiguity. Keep the first selected answer distinct from alternatives.

Do not assume that every picture is unique. Keep a separate answer for every ID, including repeated-looking images. Record the observed image-open and answer-completion times. These are model/tool elapsed times, not human reaction times. This packet does not introduce a new acceptance threshold.

Use this response shape without adding a source guess:

```json
{
  "reviewer": "assigned reviewer ID",
  "opened_at_utc": "observed timestamp",
  "completed_at_utc": "observed timestamp",
  "answers": [
    {
      "id": "opaque ID from image",
      "primary_name": "your observation",
      "quantity_bucket": "1, 2–8, 9+, or unclear",
      "visible_state": "your observation or not apparent",
      "confidence": "high, medium, or low",
      "ambiguity": "alternatives or none"
    }
  ]
}
```
