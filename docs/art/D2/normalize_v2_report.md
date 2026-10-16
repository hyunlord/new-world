# D2 normalization v2

No native resampling; maxima exclude outside outline. Source/grid/palette/semantic details are in normalize_v2_report.json.

|Sprite|Native|Canvas|Max|Status|
|---|---|---|---|---|
|01_campfire|[15, 13]|[17, 15]|[24, 26]|regenerate|
|02_lean_to|[23, 20]|[25, 22]|[34, 30]|regenerate|
|03_storage_pit|[23, 15]|[25, 17]|[34, 30]|regenerate|
|04_shelter_frame|[19, 29]|[21, 31]|[34, 30]|regenerate|
|05_berry_bush_full|[20, 16]|[22, 18]|[24, 22]|regenerate|
|06_berry_bush_two|[17, 15]|[19, 17]|[24, 22]|regenerate|
|07_berry_bush_one|[16, 15]|[18, 17]|[24, 22]|regenerate|
|08_berry_bush_bare|[24, 15]|[26, 17]|[24, 22]|regenerate|
|09_villager_gatherer|[11, 20]|[13, 22]|[18, 28]|regenerate|
|10_villager_builder|[19, 20]|[21, 22]|[18, 28]|regenerate|
|11_villager_elder|[17, 19]|[19, 21]|[18, 28]|regenerate|
|12_villager_child|[6, 15]|[8, 17]|[14, 22]|regenerate|
|13_boulder|[18, 14]|[20, 16]|[24, 22]|regenerate|
|14_young_tree|[23, 31]|[25, 33]|[24, 34]|regenerate|
|15_reed_tuft|[15, 27]|[17, 29]|[24, 22]|regenerate|
|16_berry_basket|[11, 11]|[13, 13]|[24, 22]|regenerate|

## Semantic evidence

```json
{
  "passed": false,
  "checks": {
    "berry_stages": false,
    "fire": true,
    "skin": true,
    "child_smaller": true,
    "basket_smaller": true,
    "fidelity": false,
    "palette_membership": true,
    "binary_alpha": true,
    "accent_ramps": true
  },
  "metrics": {
    "berry_red_counts": {
      "full": 23,
      "two": 4,
      "one": 3,
      "bare": 2
    },
    "fire_orange_yellow_texels": 40,
    "adult_skin_ratios": {
      "09_villager_gatherer": 0.2743362831858407,
      "10_villager_builder": 0.18324607329842932,
      "11_villager_elder": 0.28888888888888886
    },
    "child_height": 15,
    "adult_heights": {
      "09_villager_gatherer": 18,
      "10_villager_builder": 19,
      "11_villager_elder": 18
    },
    "basket_box_area": 121,
    "bush_box_areas": {
      "05_berry_bush_full": 320,
      "06_berry_bush_two": 255,
      "07_berry_bush_one": 240,
      "08_berry_bush_bare": 360
    },
    "mean_delta_e": 9.591184821655945
  },
  "sprites": {
    "01_campfire": {
      "opaque_texels": 106,
      "size": [
        15,
        13
      ],
      "mean_delta_e": 12.793016189874072,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "02_lean_to": {
      "opaque_texels": 234,
      "size": [
        23,
        20
      ],
      "mean_delta_e": 7.340582966099666,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "03_storage_pit": {
      "opaque_texels": 255,
      "size": [
        23,
        15
      ],
      "mean_delta_e": 7.630748810518194,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "04_shelter_frame": {
      "opaque_texels": 174,
      "size": [
        19,
        29
      ],
      "mean_delta_e": 11.607935132654058,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "05_berry_bush_full": {
      "opaque_texels": 211,
      "size": [
        20,
        16
      ],
      "mean_delta_e": 8.809050201017229,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "06_berry_bush_two": {
      "opaque_texels": 156,
      "size": [
        17,
        15
      ],
      "mean_delta_e": 8.00428970274065,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "07_berry_bush_one": {
      "opaque_texels": 147,
      "size": [
        16,
        15
      ],
      "mean_delta_e": 7.763025026357452,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "08_berry_bush_bare": {
      "opaque_texels": 212,
      "size": [
        24,
        15
      ],
      "mean_delta_e": 8.608961518952434,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "09_villager_gatherer": {
      "opaque_texels": 113,
      "size": [
        11,
        20
      ],
      "mean_delta_e": 9.37178306953723,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "10_villager_builder": {
      "opaque_texels": 191,
      "size": [
        19,
        20
      ],
      "mean_delta_e": 9.882186427073604,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "11_villager_elder": {
      "opaque_texels": 180,
      "size": [
        17,
        19
      ],
      "mean_delta_e": 9.466209108483902,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "12_villager_child": {
      "opaque_texels": 62,
      "size": [
        6,
        15
      ],
      "mean_delta_e": 9.88276496023807,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "13_boulder": {
      "opaque_texels": 171,
      "size": [
        18,
        14
      ],
      "mean_delta_e": 11.678109741947246,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "14_young_tree": {
      "opaque_texels": 343,
      "size": [
        23,
        31
      ],
      "mean_delta_e": 10.59529928916214,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "15_reed_tuft": {
      "opaque_texels": 228,
      "size": [
        15,
        27
      ],
      "mean_delta_e": 11.597366643277427,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "16_berry_basket": {
      "opaque_texels": 82,
      "size": [
        11,
        11
      ],
      "mean_delta_e": 9.118422634500341,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    }
  },
  "definitions": {
    "red": "R>130, G<90, B<90",
    "fire": "HSV H15..65 degrees, S>=0.35, V>=0.45",
    "skin": "Skin: HSV H12..48 S.15...65 V>=.35; palette shades within \u0394E12 of native adult candidates. Color proxy can include clothing. Water: nearest native to #BEDCE6; no water sprite.",
    "spill": "Silhouette changes are diagnostic because native spill removal is intentional. Lost opaque texels still contribute to fidelity against final RGB.",
    "size": "Opaque bounding-box height for child; opaque bounding-box area for basket.",
    "fidelity": "Opaque-texel weighted direction mean and every sprite mean must be <=6 \u0394E76. Input native is pre-spill step 3; final is before outline."
  }
}
```

## Palette additions

```json
{
  "scene_count": 48,
  "sprite_median_cut_count": 32,
  "final_count": 86,
  "added_colors": [
    {
      "role": "berry_red",
      "color": "#8A1006",
      "source": "native opaque texel"
    },
    {
      "role": "berry_red",
      "color": "#F73B31",
      "source": "native opaque texel"
    },
    {
      "role": "skin",
      "color": "#5B3520",
      "source": "native opaque texel"
    },
    {
      "role": "skin",
      "color": "#A47D58",
      "source": "native opaque texel"
    },
    {
      "role": "skin",
      "color": "#FCE8A8",
      "source": "native opaque texel"
    },
    {
      "role": "fire_orange",
      "color": "#76311A",
      "source": "native opaque texel"
    },
    {
      "role": "fire_yellow",
      "color": "#F3A504",
      "source": "native opaque texel"
    },
    {
      "role": "fire_hot_white",
      "color": "#F6E2C5",
      "source": "native opaque texel"
    },
    {
      "role": "water_highlight",
      "color": "#ECE2D5",
      "source": "native opaque texel"
    }
  ],
  "accent_ramps": {
    "berry_red": [
      "#8A1006",
      "#F73B31"
    ],
    "skin": [
      "#79482F",
      "#5D4732",
      "#BC8550",
      "#D4995F",
      "#816140",
      "#E0AB73",
      "#99794A",
      "#A5834B",
      "#BF9853",
      "#D8AB55",
      "#887643",
      "#DEBF66",
      "#B19947",
      "#EDDDA6",
      "#BCA44A",
      "#E5A757",
      "#B58A50",
      "#826E42",
      "#825833",
      "#69472D",
      "#5B3520",
      "#A47D58",
      "#FCE8A8"
    ],
    "fire_orange": [
      "#76311A"
    ],
    "fire_yellow": [
      "#F3A504"
    ],
    "fire_hot_white": [
      "#F6E2C5"
    ],
    "water_highlight": [
      "#ECE2D5"
    ]
  },
  "missing_accents": [],
  "classification": "Skin: HSV H12..48 S.15...65 V>=.35; palette shades within \u0394E12 of native adult candidates. Color proxy can include clothing. Water: nearest native to #BEDCE6; no water sprite."
}
```
