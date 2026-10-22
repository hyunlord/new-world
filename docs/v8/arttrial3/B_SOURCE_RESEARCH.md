# TRIAL3B free-model source research

Checked 2026-10-22. Preparatory research only: no model archive downloaded,
no rig imported, and no candidate consumed from the three-base-model limit.
Official pack pages and publisher preview images were inspected. Mesh bounds,
bone names, material separation and retarget quality remain unverified.

## Recommendation

Start with **Kenney Mini Characters**, selecting the least obstructed humanoid
after archive inspection. The official preview shows large heads and compact
bodies, visually closer to the TEMPLATE's approximately 30% head target than
the realistic-proportioned Quaternius Standard preview. This is a visual
inference, not a measured head/body ratio. Measure the chosen mesh and adjust
in Blender before acceptance; inspect its skin weights and animation rig.

Use author-made Blender keyframes within the existing six-keyframe allowance
for the requested actions. No Mixamo login or paid generation is necessary
for that route. Existing pack clips, if present, are not automatically accepted
as the requested action set.

## Official sources and download routes

| Candidate pack | Official source and license | Free download route | Verified scope |
|---|---|---|---|
| Kenney Mini Characters | [Pack](https://kenney.nl/assets/mini-characters), [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) | [Direct ZIP](https://kenney.nl/media/pages/assets/mini-characters/bfc7e272b4-1774770718/kenney_mini-characters.zip) | Official page identifies animation and CC0. Anonymous HTTP HEAD returned 200, application/zip. Archive not fetched. |
| Kenney Animated Characters Protagonists | [Pack](https://kenney.nl/assets/animated-characters-protagonists), [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) | [Direct ZIP](https://kenney.nl/media/pages/assets/animated-characters-protagonists/608191acc4-1774773108/kenney_animated-characters-protagonists.zip) | Official page identifies CC0. Anonymous HTTP HEAD returned 200, application/zip. Archive and proportions not inspected. |
| Quaternius Universal Base Characters Standard | [Pack](https://quaternius.com/packs/universalbasecharacters.html), [official itch page](https://quaternius.itch.io/universal-base-characters), [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) | Official itch page → Download Now → free Standard archive; no stable direct archive URL established in this research | Publisher lists humanoid retargetable rig and FBX/glTF. Standard preview explicitly shows **two** free base models and five hairstyles. Six bases describe the broader kit, not verified free entitlement. Anonymous archive-download flow not exercised. |

The Kenney donation modal has an explicit “Continue without donating” link;
the direct URLs above are that link's destination. No account, payment or
donation request was submitted. CC0 permits adaptation and commercial use;
retain publisher attribution and the archive license in B_MODEL_LOG.md for
provenance even though attribution is not a CC0 condition.

Official visual references inspected:

- [Kenney Mini Characters preview](https://kenney.nl/media/pages/assets/mini-characters/357d1a9452-1721210569/preview.png)
- [Quaternius Standard preview](https://quaternius.com/assets/images/fullres/universalbasecharacters/standard.jpg)

## Tool availability check

`blender` was absent from PATH. No Blender app was found directly under
`/Applications` or `/Users/rexxa/Applications`; Spotlight found support/source
directories but no executable. A bounded check of common portable-install
locations also found no executable. This is not an exhaustive filesystem
claim. Install or locate a verified Blender 4.x executable before rendering.

## Handoff boundaries

Finish Part 0 before downloading/importing any model. When Part 1 starts,
record actual archive URL, SHA-256, included license text, selected member,
measured head/body ratio and retained rig in B_MODEL_LOG.md. The three rows
above are research alternatives, not three attempted base models. Do not
equate pack-page animation claims with verified rig or action compatibility.
