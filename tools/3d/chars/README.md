# Living asset extension (M3D-ASSETS)

Set-B source subjects come from `v8/art-3d-chars` commit
`f0b91f999e96c9095e32b9d649a835b40e37f6e7` (TRELLIS.2 reconstruction and
M3D-2b corrected shared skins/clips). Original body/atlas and receipts are in
`assets/3d/living/{man,woman,child,goat,aurochs}`. The inherited Blender modules
retain their earlier pipeline; extension entry points below are the current API.

- `blender -b --factory-startup -P tools/3d/chars/blender/wardrobes.py`
  exports18 combinations of child/adult/elder, M/F and neolithic/bronze/iron.
  Wardrobes are attachment variants: fibre band, bronze pin with teal shoulder wrap,
  iron pin with rust shoulder wrap;
  they retain the neutral tunic and share the original atlas/animation. Elders
  receive a grey hair attachment; children use one sex-neutral body.
- `blender -b --factory-startup -P tools/3d/chars/blender/extend_animals.py -- RAW NEW_DIRECTORY SPECIES`
  uses the set-B remesh/atlas path, shared12-bone quadruped and idle/walk/graze/lie_down.
  Species: wolf,deer,wild_sheep,wild_boar,reindeer. Height is measured at the front
  trunk withers band, not antler tips. Verify raw orientation and silhouette.
- `blender -b --factory-startup -P tools/3d/chars/blender/validate_living.py`
  reimports all actual outputs, checks budgets, shared normalized rest skeletons,
  skin influence sums, finite/grounded/looped clips and actual GLB PBR colours.
- `blender -b --factory-startup -P tools/3d/chars/blender/living_manifest.py`
  writes the collision-free `living_*` metadata with dimensions and triangles.
  These are skinned AnimationPlayer GLBs; VAT is not baked in this asset pass.

Blender Z-up, front-Y; exported Godot Y-up, front+Z, metre units, feet at ground.
The skeleton names and animation names are the integration API. A valid skin and
finite animation samples do not prove anatomical or visual acceptance.

DGX scripts are inherited from the validated set-B pipeline. Execute only inside
`~/worldsim-3d`; every generation goes through `generate-safe.sh`, exclusive lock,
GPU<70°C/load1m<8 admission,80°C stop and30s cooldown. Never print or record the
existing authentication file. New transparent concepts bypass background removal.
No new dependency is installed. Evidence/raw meshes remain outside the repo in
`/Users/rexxa/work/3d-data/m3d-assets/living` and the DGX workspace.
