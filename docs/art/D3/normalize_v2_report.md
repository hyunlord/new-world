# D3 normalization v2

No native resampling; maxima exclude outside outline. Source/grid/palette/semantic details are in normalize_v2_report.json.

|Sprite|Native|Canvas|Max|Status|
|---|---|---|---|---|
|01_campfire|[23, 26]|[25, 28]|[24, 26]|regenerate|
|02_lean_to|[32, 28]|[34, 30]|[34, 30]|regenerate|
|03_storage_pit|[31, 21]|[33, 23]|[34, 30]|regenerate|
|04_shelter_frame|[32, 27]|[34, 29]|[34, 30]|regenerate|
|05_berry_bush_full|[25, 22]|[27, 24]|[24, 22]|regenerate|
|06_berry_bush_two|[23, 22]|[25, 24]|[24, 22]|pass|
|07_berry_bush_one|[22, 22]|[24, 24]|[24, 22]|pass|
|08_berry_bush_bare|[22, 22]|[24, 24]|[24, 22]|regenerate|
|09_villager_gatherer|[15, 28]|[17, 30]|[18, 28]|regenerate|
|10_villager_builder|[20, 28]|[22, 30]|[18, 28]|regenerate|
|11_villager_elder|[19, 28]|[21, 30]|[18, 28]|regenerate|
|12_villager_child|[8, 21]|[10, 23]|[14, 22]|regenerate|
|13_boulder|[24, 18]|[26, 20]|[24, 22]|regenerate|
|14_young_tree|[23, 28]|[25, 30]|[24, 34]|pass|
|15_reed_tuft|[18, 22]|[20, 24]|[24, 22]|regenerate|
|16_berry_basket|[17, 18]|[19, 20]|[24, 22]|regenerate|

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
      "full": 79,
      "two": 21,
      "one": 10,
      "bare": 2
    },
    "fire_orange_yellow_texels": 55,
    "adult_skin_ratios": {
      "09_villager_gatherer": 0.15129151291512916,
      "10_villager_builder": 0.14332247557003258,
      "11_villager_elder": 0.15878378378378377
    },
    "child_height": 21,
    "adult_heights": {
      "09_villager_gatherer": 28,
      "10_villager_builder": 27,
      "11_villager_elder": 27
    },
    "basket_box_area": 289,
    "bush_box_areas": {
      "05_berry_bush_full": 550,
      "06_berry_bush_two": 506,
      "07_berry_bush_one": 484,
      "08_berry_bush_bare": 484
    },
    "mean_delta_e": 7.241145416832665
  },
  "sprites": {
    "01_campfire": {
      "opaque_texels": 282,
      "size": [
        23,
        26
      ],
      "mean_delta_e": 13.908251963792933,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "02_lean_to": {
      "opaque_texels": 478,
      "size": [
        32,
        28
      ],
      "mean_delta_e": 6.842674805663264,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "03_storage_pit": {
      "opaque_texels": 543,
      "size": [
        31,
        21
      ],
      "mean_delta_e": 7.871529533970119,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "04_shelter_frame": {
      "opaque_texels": 341,
      "size": [
        32,
        27
      ],
      "mean_delta_e": 8.730220443744939,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "05_berry_bush_full": {
      "opaque_texels": 375,
      "size": [
        25,
        22
      ],
      "mean_delta_e": 6.609800804000279,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "06_berry_bush_two": {
      "opaque_texels": 348,
      "size": [
        23,
        22
      ],
      "mean_delta_e": 5.658650243323124,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "07_berry_bush_one": {
      "opaque_texels": 325,
      "size": [
        22,
        22
      ],
      "mean_delta_e": 5.135440999592462,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "08_berry_bush_bare": {
      "opaque_texels": 335,
      "size": [
        22,
        22
      ],
      "mean_delta_e": 4.330224498590763,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "09_villager_gatherer": {
      "opaque_texels": 271,
      "size": [
        15,
        28
      ],
      "mean_delta_e": 6.4220522673264036,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "10_villager_builder": {
      "opaque_texels": 307,
      "size": [
        20,
        28
      ],
      "mean_delta_e": 5.325573659728759,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "11_villager_elder": {
      "opaque_texels": 296,
      "size": [
        19,
        28
      ],
      "mean_delta_e": 6.258676381455454,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "12_villager_child": {
      "opaque_texels": 111,
      "size": [
        8,
        21
      ],
      "mean_delta_e": 5.630438500643355,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "13_boulder": {
      "opaque_texels": 293,
      "size": [
        24,
        18
      ],
      "mean_delta_e": 11.611968376872403,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "14_young_tree": {
      "opaque_texels": 358,
      "size": [
        23,
        28
      ],
      "mean_delta_e": 5.639402986311766,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "15_reed_tuft": {
      "opaque_texels": 261,
      "size": [
        18,
        22
      ],
      "mean_delta_e": 9.732264725044528,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "16_berry_basket": {
      "opaque_texels": 231,
      "size": [
        17,
        18
      ],
      "mean_delta_e": 6.9213867178746495,
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
  "scene_count": 19,
  "sprite_median_cut_count": 32,
  "final_count": 57,
  "added_colors": [
    {
      "role": "berry_red",
      "color": "#870502",
      "source": "native opaque texel"
    },
    {
      "role": "berry_red",
      "color": "#FB2A3A",
      "source": "native opaque texel"
    },
    {
      "role": "skin",
      "color": "#5A3C22",
      "source": "native opaque texel"
    },
    {
      "role": "skin",
      "color": "#F1BF85",
      "source": "native opaque texel"
    },
    {
      "role": "skin",
      "color": "#FBF2D1",
      "source": "native opaque texel"
    },
    {
      "role": "fire_orange",
      "color": "#81340B",
      "source": "native opaque texel"
    },
    {
      "role": "fire_yellow",
      "color": "#AD9D7C",
      "source": "native opaque texel"
    },
    {
      "role": "fire_hot_white",
      "color": "#D9C5C7",
      "source": "native opaque texel"
    },
    {
      "role": "water_highlight",
      "color": "#D9D8D9",
      "source": "native opaque texel"
    }
  ],
  "accent_ramps": {
    "berry_red": [
      "#870502",
      "#FB2A3A"
    ],
    "skin": [
      "#6A472D",
      "#C19256",
      "#7B6A3B",
      "#F6E6B0",
      "#D7BD77",
      "#D69E58",
      "#9F7F53",
      "#5A3C22",
      "#F1BF85",
      "#FBF2D1",
      "#AD9D7C"
    ],
    "fire_orange": [
      "#81340B"
    ],
    "fire_yellow": [
      "#AD9D7C"
    ],
    "fire_hot_white": [
      "#D9C5C7"
    ],
    "water_highlight": [
      "#D9D8D9"
    ]
  },
  "missing_accents": [],
  "classification": "Skin: HSV H12..48 S.15...65 V>=.35; palette shades within \u0394E12 of native adult candidates. Color proxy can include clothing. Water: nearest native to #BEDCE6; no water sprite."
}
```
