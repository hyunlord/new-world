# Normalization report

RGB key distance <=60; HSV spill +/-20 degrees, S>0.45. Area-weighted 5-bit block mode; CIELAB DeltaE76 nearest palette; outside 8-neighbor outline restricted to perimeter-connected transparent background. Grid bounds use floor(i*size/count), covering arbitrary dimensions without lost pixels.

Top/side output edges must stay transparent. Bottom outline may touch the baseline. Source cell edge contact is a failure. Aspect fit may differ by at most one target texel due to integer rounding. Luma uses encoded sRGB Rec.709 coefficients, including outline.

|Cell|Size|Source opaque|Source edge|Opaque|Colors|Palette %|Partial alpha|Forbidden edge|Baseline|Luma|Pass|
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
|01 campfire|32x32|19812|0|431|15|100.0|0|0|5|0.2780|True|
|02 lean_to|32x32|36705|0|530|13|100.0|0|0|8|0.2833|True|
|03 storage_pit|32x32|42233|0|557|16|100.0|0|0|10|0.2642|True|
|04 shelter_frame|32x32|23250|0|468|15|100.0|0|0|4|0.2240|True|
|05 berry_bush_full|16x16|28582|0|132|13|100.0|0|0|3|0.2113|True|
|06 berry_bush_two|16x16|26025|0|128|13|100.0|0|0|5|0.2006|True|
|07 berry_bush_one|16x16|25605|0|125|12|100.0|0|0|4|0.1928|True|
|08 berry_bush_bare|16x16|24432|0|128|14|100.0|0|0|4|0.1964|True|
|09 villager_gatherer|16x24|18166|0|206|12|100.0|0|0|4|0.1805|True|
|10 villager_builder|16x24|21500|0|189|11|100.0|0|0|4|0.2366|True|
|11 villager_elder|16x24|20365|0|229|10|100.0|0|0|5|0.1916|True|
|12 villager_child|16x24|10403|0|227|11|100.0|0|0|5|0.2119|True|
|13 boulder|16x16|22487|0|115|16|100.0|0|0|8|0.2647|True|
|14 young_tree|16x32|24864|0|169|16|100.0|0|0|5|0.2282|True|
|15 reed_tuft|16x16|21936|0|142|17|100.0|0|0|5|0.2153|True|
|16 berry_basket|16x16|21393|0|147|12|100.0|0|0|6|0.1551|True|
