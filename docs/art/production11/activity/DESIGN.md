# Activity icon presentation

Inherit production8/DESIGN.md: Galmuri9, 12 px screen text, dark backing #14120F at 0.86, nearest-neighbour pixels. No browser components.

Six procedural icons use 16×16 source pixels with a one-pixel dark #211C25 outline. Craft retains the accepted U1_04 18×18 texture. Colors: TendFire #F39A43, Repair #91BED9, HearthCircle #D6A3E0, Explore #98D47B, Play #F3D86B, SitByFire #79D5CD. Silhouette carries meaning independently of hue.

Agent icons retain zoom>=2, walking/arrived gate and existing anchor. Village summaries use the same textures at source size under the existing label, 18px high, count font12px, icon/count gap3px, item gap8px; up to three pairs. Hidden for zoom>=2, abandoned villages or empty counts. Count ordering descending, kind ascending on ties. Summary backing included in roof-clearance geometry.

Verification is the GOAL's Godot tests and independent real-capture blind protocol; this prep artifact makes no visual acceptance claim.
