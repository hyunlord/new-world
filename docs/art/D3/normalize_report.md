# Normalization report

RGB key distance <=60; HSV spill +/-20 degrees, S>0.45. Area-weighted 5-bit block mode; CIELAB DeltaE76 nearest palette; outside 8-neighbor outline restricted to perimeter-connected transparent background. Grid bounds use floor(i*size/count), covering arbitrary dimensions without lost pixels.

Top/side output edges must stay transparent. Bottom outline may touch the baseline. Source cell edge contact is a failure. Aspect fit may differ by at most one target texel due to integer rounding. Luma uses encoded sRGB Rec.709 coefficients, including outline.

|Cell|Size|Source opaque|Source edge|Opaque|Colors|Palette %|Partial alpha|Forbidden edge|Baseline|Luma|Pass|
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
|01 campfire|32x32|9515|0|490|15|100.0|0|0|8|0.2848|True|
|02 lean_to|32x32|16438|0|547|11|100.0|0|0|4|0.2264|True|
|03 storage_pit|32x32|19008|0|545|9|100.0|0|0|15|0.2716|True|
|04 shelter_frame|32x32|12119|0|434|10|100.0|0|0|7|0.1753|True|
|05 berry_bush_full|16x16|12799|0|146|11|100.0|0|0|7|0.2136|True|
|06 berry_bush_two|16x16|11806|0|143|11|100.0|0|0|7|0.2088|True|
|07 berry_bush_one|16x16|11485|0|155|10|100.0|0|0|7|0.2093|True|
|08 berry_bush_bare|16x16|11562|0|155|11|100.0|0|0|7|0.2181|True|
|09 villager_gatherer|16x24|9665|0|232|6|100.0|0|0|9|0.1947|True|
|10 villager_builder|16x24|10715|0|197|6|100.0|0|0|9|0.1947|True|
|11 villager_elder|16x24|10792|0|227|7|100.0|0|0|12|0.2073|True|
|12 villager_child|16x24|6003|0|265|5|100.0|0|0|11|0.2239|True|
|13 boulder|16x16|9707|0|122|13|100.0|0|0|11|0.2358|True|
|14 young_tree|16x32|12117|0|157|11|100.0|0|0|7|0.2238|True|
|15 reed_tuft|16x16|9092|0|166|12|100.0|0|0|10|0.2516|True|
|16 berry_basket|16x16|7882|0|178|4|100.0|0|0|7|0.2160|True|
