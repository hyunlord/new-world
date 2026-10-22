# Route A hair generation

Six overlay-only edits on the original 1024x1536 body canvases. No subtraction, box extraction, hole repair, refitting, quantization or outline forcing. All six pass the current per-layer structural checker; combination checks and blind readability remain separate.

| Layer | Tries | Structure | Tool wall seconds |
|---|---:|---|---:|
| hair_short_S | 1 | PASS | 21.2 |
| hair_short_E | 1 | PASS | 23.0 |
| hair_short_N | 1 | PASS | 41.8 |
| hair_long_tied_S | 1 | PASS | 60.9 |
| hair_long_tied_E | 1 | PASS | 83.4 |
| hair_long_tied_N | 1 | PASS | 106.4 |

All masks assign region 2 to every visible hair pixel. The accepted PNGs are 64x96 (scale1), created with prepare_a chroma key, whole-canvas Lanczos downscale and alpha<16 cleanup. Actual prompts, four references and their SHA256, raw SHA256, tool start/time, processing operations and checker results live in each sidecar. Raw attempts and checks are retained at `/Users/rexxa/work/arttrial3-data/route_a/hair_*/try1/`.

Limits: short_S generated a brown wash despite requested neutral grey; retained unchanged as a reported palette limitation. Short E/N are shaggy cropped hair; tied variants have prominent attached ponytails. The requested pale wash is not a structural gate. The checker does not independently establish semantic scalp fit or style. Hair tufts can extend beyond the head box; these generated extents were not forced inward. No rejected attempts, retries, credits exposed by the built-in tool, or capped failures in this slice. Generation wall seconds include queue time for five concurrent requests, not exclusive model compute time.

Manual composite inspection: `hair_composition_inspection.png` in the external route_a directory shows all six over their matching idle bodies at4x nearest-neighbour display. All sit on the scalp; south/east faces remain visible and ponytails are attached. This creator inspection is not the blind GIF readability test.
