# D1 normalization v2

No native resampling; maxima exclude outside outline. Source/grid/palette/semantic details are in normalize_v2_report.json.

|Sprite|Native|Canvas|Max|Status|
|---|---|---|---|---|
|01_campfire|[33, 40]|[35, 42]|[24, 26]|regenerate|
|02_lean_to|[32, 45]|[34, 47]|[34, 30]|regenerate|
|03_storage_pit|[46, 35]|[48, 37]|[34, 30]|regenerate|
|04_shelter_frame|[30, 41]|[32, 43]|[34, 30]|regenerate|
|05_berry_bush_full|[41, 36]|[43, 38]|[24, 22]|regenerate|
|06_berry_bush_two|[36, 34]|[38, 36]|[24, 22]|regenerate|
|07_berry_bush_one|[36, 34]|[38, 36]|[24, 22]|regenerate|
|08_berry_bush_bare|[35, 34]|[37, 36]|[24, 22]|regenerate|
|09_villager_gatherer|[23, 43]|[25, 45]|[18, 28]|regenerate|
|10_villager_builder|[33, 44]|[35, 46]|[18, 28]|regenerate|
|11_villager_elder|[32, 39]|[34, 41]|[18, 28]|regenerate|
|12_villager_child|[17, 32]|[19, 34]|[14, 22]|regenerate|
|13_boulder|[39, 29]|[41, 31]|[24, 22]|regenerate|
|14_young_tree|[33, 44]|[35, 46]|[24, 34]|regenerate|
|15_reed_tuft|[35, 37]|[37, 39]|[24, 22]|regenerate|
|16_berry_basket|[30, 28]|[32, 30]|[24, 22]|regenerate|

## Semantic evidence

```json
{
  "passed": false,
  "checks": {
    "berry_stages": true,
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
      "full": 112,
      "two": 20,
      "one": 9,
      "bare": 0
    },
    "fire_orange_yellow_texels": 196,
    "adult_skin_ratios": {
      "09_villager_gatherer": 0.30256410256410254,
      "10_villager_builder": 0.33152173913043476,
      "11_villager_elder": 0.2709497206703911
    },
    "child_height": 32,
    "adult_heights": {
      "09_villager_gatherer": 43,
      "10_villager_builder": 44,
      "11_villager_elder": 39
    },
    "basket_box_area": 812,
    "bush_box_areas": {
      "05_berry_bush_full": 1476,
      "06_berry_bush_two": 1224,
      "07_berry_bush_one": 1190,
      "08_berry_bush_bare": 1156
    },
    "mean_delta_e": 6.69524310417799
  },
  "sprites": {
    "01_campfire": {
      "opaque_texels": 570,
      "size": [
        33,
        40
      ],
      "mean_delta_e": 13.00543590633321,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "02_lean_to": {
      "opaque_texels": 751,
      "size": [
        32,
        45
      ],
      "mean_delta_e": 5.917881882800127,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "03_storage_pit": {
      "opaque_texels": 1206,
      "size": [
        46,
        35
      ],
      "mean_delta_e": 5.8817329643183935,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "04_shelter_frame": {
      "opaque_texels": 483,
      "size": [
        30,
        41
      ],
      "mean_delta_e": 7.564125197492428,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "05_berry_bush_full": {
      "opaque_texels": 916,
      "size": [
        41,
        36
      ],
      "mean_delta_e": 6.833870667664272,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "06_berry_bush_two": {
      "opaque_texels": 746,
      "size": [
        36,
        34
      ],
      "mean_delta_e": 6.065717879266947,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "07_berry_bush_one": {
      "opaque_texels": 733,
      "size": [
        36,
        34
      ],
      "mean_delta_e": 5.977584494153423,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "08_berry_bush_bare": {
      "opaque_texels": 686,
      "size": [
        35,
        34
      ],
      "mean_delta_e": 5.8524751095094585,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "09_villager_gatherer": {
      "opaque_texels": 585,
      "size": [
        23,
        43
      ],
      "mean_delta_e": 5.688716100577326,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "10_villager_builder": {
      "opaque_texels": 736,
      "size": [
        33,
        44
      ],
      "mean_delta_e": 4.924134695022971,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "11_villager_elder": {
      "opaque_texels": 716,
      "size": [
        32,
        39
      ],
      "mean_delta_e": 5.162917704906277,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "12_villager_child": {
      "opaque_texels": 358,
      "size": [
        17,
        32
      ],
      "mean_delta_e": 4.568507352585247,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "13_boulder": {
      "opaque_texels": 667,
      "size": [
        39,
        29
      ],
      "mean_delta_e": 9.02756745768407,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "14_young_tree": {
      "opaque_texels": 740,
      "size": [
        33,
        44
      ],
      "mean_delta_e": 7.708956983596569,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "15_reed_tuft": {
      "opaque_texels": 621,
      "size": [
        35,
        37
      ],
      "mean_delta_e": 7.7303748819433125,
      "palette_membership": 1.0,
      "binary_alpha": true,
      "silhouette_unchanged": false
    },
    "16_berry_basket": {
      "opaque_texels": 566,
      "size": [
        30,
        28
      ],
      "mean_delta_e": 6.685515243183244,
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
  "scene_count": 31,
  "sprite_median_cut_count": 32,
  "final_count": 67,
  "added_colors": [
    {
      "role": "berry_red",
      "color": "#84120B",
      "source": "native opaque texel"
    },
    {
      "role": "berry_red",
      "color": "#F95813",
      "source": "native opaque texel"
    },
    {
      "role": "skin",
      "color": "#5D3921",
      "source": "native opaque texel"
    },
    {
      "role": "skin",
      "color": "#E18B4F",
      "source": "native opaque texel"
    },
    {
      "role": "skin",
      "color": "#E9D4BA",
      "source": "native opaque texel"
    },
    {
      "role": "fire_orange",
      "color": "#7B330E",
      "source": "native opaque texel"
    },
    {
      "role": "fire_yellow",
      "color": "#9B9074",
      "source": "native opaque texel"
    },
    {
      "role": "water_highlight",
      "color": "#F2ECE3",
      "source": "native opaque texel"
    }
  ],
  "accent_ramps": {
    "berry_red": [
      "#84120B",
      "#F95813"
    ],
    "skin": [
      "#694B31",
      "#86633F",
      "#A2804B",
      "#AE8C50",
      "#D1B681",
      "#B29453",
      "#BEA366",
      "#B29B5A",
      "#E6C17F",
      "#CAA55A",
      "#E79752",
      "#B07F50",
      "#5D3921",
      "#E18B4F",
      "#E9D4BA",
      "#9B9074"
    ],
    "fire_orange": [
      "#7B330E"
    ],
    "fire_yellow": [
      "#9B9074"
    ],
    "fire_hot_white": [
      "#E9D4BA"
    ],
    "water_highlight": [
      "#F2ECE3"
    ]
  },
  "missing_accents": [],
  "classification": "Skin: HSV H12..48 S.15...65 V>=.35; palette shades within \u0394E12 of native adult candidates. Color proxy can include clothing. Water: nearest native to #BEDCE6; no water sprite."
}
```
