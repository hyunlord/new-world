# D3 v3 measurement

Fidelity uses only common opaque pixels; keyed-out pixels reported separately. All16 installed; remaining failures are provisional.

|Sprite|Native|Mean ΔE|Keyed out|Size|Status|
|---|---|---:|---:|---|---|
|direction_01_campfire|[23, 26]|10.559|7|True|provisional|
|direction_02_lean_to|[32, 28]|5.042|33|True|accepted|
|direction_03_storage_pit|[31, 21]|7.620|1|True|provisional|
|direction_04_shelter_frame|[32, 27]|6.893|49|True|provisional|
|direction_05_berry_bush_full|[25, 22]|6.249|3|False|provisional|
|direction_06_berry_bush_two|[23, 22]|5.047|4|True|accepted|
|direction_07_berry_bush_one|[22, 22]|4.454|2|True|accepted|
|direction_08_berry_bush_bare|[22, 22]|4.531|3|True|accepted|
|direction_09_villager_gatherer|[15, 28]|5.420|30|True|accepted|
|direction_10_villager_builder|[20, 28]|4.501|37|False|provisional|
|direction_11_villager_elder|[19, 28]|5.121|48|False|provisional|
|direction_12_villager_child|[12, 21]|4.368|21|True|accepted|
|direction_13_boulder|[24, 18]|6.660|1|True|provisional|
|direction_14_young_tree|[24, 28]|5.860|7|True|accepted|
|direction_15_reed_tuft|[18, 22]|9.737|3|True|provisional|
|direction_16_berry_basket|[17, 18]|6.907|11|True|provisional|

## Semantic checks

```json
{
  "berry_counts": [
    79,
    18,
    7,
    0
  ],
  "berry_pass": true,
  "fire_counts": [
    33
  ],
  "fire_pass": true,
  "child_pass": true,
  "basket_pass": true,
  "passed": true
}
```

Animation checks do not apply to these single-pose direction cells.
JPEG recovery tests measure colour accuracy after matching the known production palette. Raw JPEG medians do not consistently reach 90%.
The specified edge-minus-midpoint score does not guarantee rejection of a strong 8px harmonic for a 4px fundamental; production-scale 5.9px harmonic fixtures pass.
