# Route B preflight

Date: 2026-10-22 KST. Status: **STOPPED_TOOL_UNAVAILABLE**. No generation was submitted; measured job count and credits spent by this preflight are both zero. This is an availability result, not a structural or artistic failure.

## Local checks and recovery

| Check / command | Observed result |
|---|---|
| `command -v higgsfield`; `higgsfield --help` before installation | No executable; shell exit 127. |
| `npm i -g @higgsfield/cli` | Exit 0; official CLI installed. |
| `higgsfield version` | `higgsfield 1.2.2 (53301eadf7311fc6a6a4c024cf4aff432028e974) built 2026-10-16T23:42:35Z` |
| `higgsfield account status --json` | `Error: No workspace selected.` Hint: `hf workspace set <workspace_id>`. |
| `higgsfield workspace list --json` | `Error: higgsfield: request failed (no response received)`; hint: `hf auth login`. |
| `higgsfield model get image_to_3d --json` | `Error: No workspace selected.` |
| Meshy secret presence, checked without printing values | No nonempty Meshy key/token in inherited environment; no Meshy setting in `~/.env`; no repository `.env`; `~/.meshy` and `~/.config/meshy` absent. No assertion about unsearched secret stores. |
| `command -v blender`; app checks | Blender executable absent; no Blender.app in `/Applications` or `~/Applications`. Version unavailable. |

The CLI does not expose a non-secret `auth status` command. `auth token` was deliberately not called. Account/workspace checks establish that generation is not usable in this session; they do not independently establish why the account request received no response. Interactive login was not launched: the goal says to use Meshy when Higgsfield is not ready and stop B when no key exists. Blender installation was deferred because the required 3D-generation prerequisite is unavailable; installing a renderer would not repair that prerequisite.

Raw, sanitized evidence: `/Users/rexxa/work/arttrial3-data/preflight/summary.json`, `higgsfield-install.log`, `higgsfield-account-status.{json,stderr}`, `higgsfield-workspace-list.{json,stderr}`, `image-to-3d-schema.{json,stderr}`, and `higgsfield-help.txt`. Original upstream references are downloaded into the same directory. Nothing in these records is an authenticated successful generation, rigging, animation, or render result.

## Verified upstream capability and next commands

The [official CLI README](https://github.com/higgsfield-ai/cli/blob/main/README.md), [model schema reference](https://github.com/higgsfield-ai/cli/blob/main/MODELS.md), and [official animated 3D asset guide](https://github.com/higgsfield-ai/skills/blob/main/higgsfield-websites/references/game-3d-animation.md) document `image_to_3d`, `3d_rigging`, and the `animation-action` preset catalog. The game deployment command is not the mesh-generation command. Native 3D generation can use Higgsfield workspace credits without a separate Meshy key. These are public source capabilities; the live model schema and account entitlement remain unverified.

If authentication and workspace access become available in a later authorized run, validate them before spending credits:

```bash
higgsfield workspace list --json
higgsfield workspace set <verified-workspace-id>
higgsfield account status --json
higgsfield model get image_to_3d --json
higgsfield model get 3d_rigging --json
higgsfield preset list animation-action --query idle --json
higgsfield preset list animation-action --query walk --json
higgsfield preset list animation-action --query axe --json
higgsfield preset list animation-action --query carry --json
higgsfield preset list animation-action --query sleep --json
brew install --cask blender
/Applications/Blender.app/Contents/MacOS/Blender --version
```

The exact action IDs must come from those catalog responses; do not invent them. A suitable full-body A-pose reference and verified action ID are required before the documented generation command can be instantiated:

```bash
higgsfield generate create image_to_3d \
  --image /Users/rexxa/work/arttrial3-data/route_b/character-reference.png \
  --should_texture true --enable_rigging true --enable_animation true \
  --pose_mode a-pose --animation_action_id <verified-idle-action-id> \
  --wait --json
higgsfield generate create 3d_rigging \
  --model_url <verified-output-glb-url> \
  --enable_animation true --animation_action_id <verified-additional-action-id> \
  --wait --json
```

These commands are documented follow-up templates, not executed results. Separate outfit/hair meshes, all five required clip types, object-pass separation, 4.x Blender compatibility, and fixed-upper-body compliance remain unverified. `brew install --cask blender` must be version-checked because the unversioned cask does not itself prove the required 4.x version. No substitute procedural character was created to disguise the unavailable generation route.
