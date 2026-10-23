# Wall texture prompt template, schema version 1

Create one game texture component of {style_description}, made of {material}, era
{era}. Restrained earth-colour wash and dark-brown ink, matching the project's
natural-history illustration anchors; upper-left lighting. No text or labels.
Attach the selected style anchor. Keep construction scale identical across pieces.

Fill COMPONENT with one of the following (one request per component):
- face: seamless horizontal wall face, a straight 256×44 strip, no top surface.
- top_ew: seamless horizontal 256×64 strip, construction seen from above with its
  longitudinal run east–west. Material structure runs along the horizontal axis.
- top_ns: seamless vertical 64×256 strip, construction seen from above with its
  longitudinal run north–south. Generate independently; light still upper-left.
- end: 24×44 termination of the vertical face, no top surface or ground shadow.
- junction: 64×64 top-down crossing/turning construction piece, compatible with
  incoming strips on every side; no complete building, face or ground.
- door_closed / door_open: matching 64×60 closed slab or open frame in this wall
  height. Entire standing silhouette, transparent outside; open aperture transparent.

Repeating pieces fill the entire rectangular canvas without border, perspective
vanishing point, text, margins, cast shadows or isolated object silhouettes.
The dimensions above describe the normalized output. Fill the generator's entire
available canvas; never letterbox a narrow strip on a white background. Use local
upper-left facet light without a global gradient across repeating edges. Opposite
edges must share course positions and comparable mean colour. For end pieces,
request actual transparency outside the complete face-end silhouette, never white.
Nonrepeating pieces preserve transparent outside alpha. Each component gets at
most three attempts; preserve every generated original and request provenance.
