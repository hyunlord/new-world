# Normalization report

RGB key distance <=60; HSV spill +/-20 degrees, S>0.45. Area-weighted 5-bit block mode; CIELAB DeltaE76 nearest palette; outside 8-neighbor outline restricted to perimeter-connected transparent background. Grid bounds use floor(i*size/count), covering arbitrary dimensions without lost pixels.

Top/side output edges must stay transparent. Bottom outline may touch the baseline. Source cell edge contact is a failure. Aspect fit may differ by at most one target texel due to integer rounding. Luma uses encoded sRGB Rec.709 coefficients, including outline.

|Cell|Size|Source opaque|Source edge|Opaque|Colors|Palette %|Partial alpha|Forbidden edge|Baseline|Luma|Pass|
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
|01 campfire|32x32|7896|0|469|27|100.0|0|0|7|0.3314|True|
|02 lean_to|32x32|16815|0|479|26|100.0|0|0|4|0.2171|True|
|03 storage_pit|32x32|19124|0|531|28|100.0|0|0|8|0.2936|True|
|04 shelter_frame|32x32|9453|0|473|21|100.0|0|0|8|0.1842|True|
|05 berry_bush_full|16x16|15035|0|136|15|100.0|0|0|4|0.1769|True|
|06 berry_bush_two|16x16|11713|0|131|16|100.0|0|0|4|0.1832|True|
|07 berry_bush_one|16x16|10565|0|131|16|100.0|0|0|4|0.1918|True|
|08 berry_bush_bare|16x16|10458|0|127|17|100.0|0|0|4|0.1959|True|
|09 villager_gatherer|16x24|8121|0|213|21|100.0|0|0|4|0.2522|True|
|10 villager_builder|16x24|9510|0|218|14|100.0|0|0|9|0.2313|True|
|11 villager_elder|16x24|9023|0|263|20|100.0|0|0|8|0.2339|True|
|12 villager_child|16x24|4532|0|187|18|100.0|0|0|4|0.2465|True|
|13 boulder|16x16|11593|0|117|20|100.0|0|0|6|0.2651|True|
|14 young_tree|16x32|12643|0|164|20|100.0|0|0|3|0.1814|True|
|15 reed_tuft|16x16|10306|0|126|22|100.0|0|0|9|0.1589|True|
|16 berry_basket|16x16|6322|0|168|14|100.0|0|0|6|0.2198|True|
