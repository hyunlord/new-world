# Pass 17 Stage A — PENDING

Mode: tuning.

| Rule | Verdict |
|---|---|
| R1 | PASS |
| R3-P | PASS |
| R3-NS | PASS |
| R3-RD | PASS |
| R2 | PASS |
| R4 | PENDING |
| R5 | PENDING |

| Metric | Candidate | Baseline | Estimate | 95% CI | Valid bootstrap draws | Bar | Verdict |
|---|---:|---:|---:|---|---:|---|---|
| R1 | 0.136308 | 0.569667 | 0.136308 | [0.126366, 0.148105] | 2000/2000 | 0.10 <= pooled adult Idle share <= 0.45 | PASS |
| R1-baseline | undefined | undefined | 0.569667 | [0.546916, 0.591928] | 2000/2000 | reference only | PASS |
| R2-alive | 42.4375 | 42.5625 | -0.00293686 | [-0.00980432, 0] | 2000/2000 | relative change within +/-0.10 | PASS |
| R2-deaths | 0.8125 | 0.6875 | 0.125 | [-0.25, 0.625] | 2000/2000 | non-old-age deaths difference <= +1.5/world | PASS |
| R2-shelters | 3.5625 | 3.6875 | 0.966102 | [0.877535, 1] | 2000/2000 | completed shelters ratio >= 0.90 | PASS |
| R2-food | 355.089 | 357.572 | -0.00694499 | [-0.0184974, 0.00551171] | 2000/2000 | noon food relative change within +/-0.15 | PASS |
| R2-wood | 91.1104 | 91.4 | -0.00316831 | [-0.0165085, 0.0100748] | 2000/2000 | noon wood relative change within +/-0.15 | PASS |
| R2-social | 1.69143 | 1.69332 | 0.998886 | [0.989052, 1.0079] | 2000/2000 | pooled adult social rate ratio >= 0.90 | PASS |
| R3-P | undefined | undefined | 0.888711 | [0.828884, 0.938249] | 2000/2000 | rho >= 0.25; CI excludes zero | PASS |
| R3-NS | undefined | undefined | 0.444882 | [0.374248, 0.507634] | 2000/2000 | rho >= 0.25; CI excludes zero | PASS |
| R3-RD | undefined | undefined | 0.351151 | [0.304187, 0.403923] | 2000/2000 | rho >= 0.20; CI excludes zero | PASS |

## Methods

- bootstrap: 2,000 world resamples; fresh numpy.default_rng(0) per statistic; percentile 95% CI; undefined draws excluded and count reported
- R1: Pooled nonelder-adult hourly observations from 08:00 through 17:00 inclusive; Idle state 0
- R2: Paired worlds; equal-world health means; daily noon stores sum settlements; social primary pools completions/adult-days, mean world rate is sensitivity
- shelters: Final minus initial complete Shelter count, including daughter-camp boot shelters
- R3: Pooled-person Spearman average tie ranks; >=14,400 nonelder-adult ticks; zero-activity NS/RD shares=0; P requires >=3 normally completed activities
- activity_minutes: Consuming Activity ticks only, excluding travel; adult and all-life-stage totals shown separately

## Eligibility and sensitivity

```json
{
  "R2-shelters": {
    "definition": "final minus initial complete Shelter count, including daughter-camp boot shelters"
  },
  "R2-social": {
    "candidate_mean_world_rate": 1.6737805611389784,
    "baseline_mean_world_rate": 1.676920716021309,
    "candidate_sensitivity_worlds": 16,
    "baseline_sensitivity_worlds": 16
  },
  "R3-P": {
    "eligible_people": 926,
    "adult_10day_people": 934,
    "zero_activity_people": 0,
    "no_place_redraws": 3170
  },
  "R3-NS": {
    "eligible_people": 934,
    "adult_10day_people": 934,
    "zero_activity_people": 0,
    "no_place_redraws": 3170
  },
  "R3-RD": {
    "eligible_people": 934,
    "adult_10day_people": 934,
    "zero_activity_people": 0,
    "no_place_redraws": 3170
  }
}
```

## game_v20

16 worlds × 60 days; seeds [17100, 17101, 17102, 17103, 17104, 17105, 17106, 17107, 17108, 17109, 17110, 17111, 17112, 17113, 17114, 17115].

| Hour | Life stage | Observations | Idle | Seeking | ConsumingAgent | ConsumingOther | Activity |
|---|---|---:|---:|---:|---:|---:|---:|
| 0 | adult | 26120 | 0 | 0.0966692 | 0 | 0.903331 | 0 |
| 0 | child | 2862 | 0 | 0.00104822 | 0 | 0.998952 | 0 |
| 0 | elder | 6140 | 0 | 0.00309446 | 0 | 0.996906 | 0 |
| 60 | adult | 26122 | 0.000114846 | 0.0534415 | 0 | 0.946444 | 0 |
| 60 | child | 2864 | 0 | 0 | 0 | 1 | 0 |
| 60 | elder | 6137 | 0 | 0.00195535 | 0 | 0.998045 | 0 |
| 120 | adult | 26119 | 3.82863e-05 | 0.0272598 | 0 | 0.972702 | 0 |
| 120 | child | 2866 | 0 | 0 | 0 | 1 | 0 |
| 120 | elder | 6135 | 0 | 0.00391198 | 0 | 0.996088 | 0 |
| 180 | adult | 26120 | 0.000267994 | 0.0824273 | 0 | 0.917305 | 0 |
| 180 | child | 2869 | 0 | 0 | 0 | 1 | 0 |
| 180 | elder | 6139 | 0 | 0.0848672 | 0 | 0.915133 | 0 |
| 240 | adult | 26113 | 0.00015318 | 0.0940528 | 0 | 0.905794 | 0 |
| 240 | child | 2868 | 0 | 0 | 0 | 1 | 0 |
| 240 | elder | 6136 | 0 | 0.0914276 | 0 | 0.908572 | 0 |
| 300 | adult | 26105 | 0.00145566 | 0.136257 | 0 | 0.862287 | 0 |
| 300 | child | 2867 | 0 | 0.000348797 | 0 | 0.999651 | 0 |
| 300 | elder | 6127 | 0.000489636 | 0.129264 | 0 | 0.870246 | 0 |
| 360 | adult | 26115 | 0.840437 | 0.114417 | 0 | 0.0451465 | 0 |
| 360 | child | 2868 | 0.980474 | 0.0170851 | 0 | 0.00244073 | 0 |
| 360 | elder | 6117 | 0.86186 | 0.0974334 | 0 | 0.0407062 | 0 |
| 420 | adult | 26153 | 0.160096 | 0.824571 | 0.0149505 | 0.000382365 | 0 |
| 420 | child | 2872 | 0.914345 | 0.0856546 | 0 | 0 | 0 |
| 420 | elder | 6125 | 0.517388 | 0.474122 | 0.00832653 | 0.000163265 | 0 |
| 480 | adult | 26020 | 0.283666 | 0.67917 | 0.00184473 | 0.035319 | 0 |
| 480 | child | 2826 | 0.958953 | 0.0215853 | 0 | 0.0194621 | 0 |
| 480 | elder | 6057 | 0.604425 | 0.377249 | 0.000660393 | 0.0176655 | 0 |
| 540 | adult | 26071 | 0.358789 | 0.633002 | 0.00609873 | 0.00210962 | 0 |
| 540 | child | 2829 | 0.977024 | 0.0229763 | 0 | 0 | 0 |
| 540 | elder | 6046 | 0.686239 | 0.310949 | 0.00248098 | 0.000330797 | 0 |
| 600 | adult | 26078 | 0.332157 | 0.633446 | 0.00138047 | 0.0330163 | 0 |
| 600 | child | 2830 | 0.964311 | 0.0257951 | 0 | 0.00989399 | 0 |
| 600 | elder | 6061 | 0.736182 | 0.242864 | 0.00131991 | 0.0196337 | 0 |
| 660 | adult | 26068 | 0.503261 | 0.490678 | 0.00540893 | 0.000652141 | 0 |
| 660 | child | 2833 | 0.978821 | 0.021179 | 0 | 0 | 0 |
| 660 | elder | 6063 | 0.936995 | 0.0615207 | 0.00148441 | 0 | 0 |
| 720 | adult | 26073 | 0.673532 | 0.280827 | 0.0014958 | 0.0441453 | 0 |
| 720 | child | 2835 | 0.951323 | 0.037037 | 0 | 0.0116402 | 0 |
| 720 | elder | 6056 | 0.928005 | 0.0521797 | 0.000825627 | 0.0189894 | 0 |
| 780 | adult | 26066 | 0.661667 | 0.335303 | 0.00268549 | 0.000345277 | 0 |
| 780 | child | 2836 | 0.96756 | 0.0313822 | 0 | 0.00105783 | 0 |
| 780 | elder | 6059 | 0.944545 | 0.0541343 | 0.00132035 | 0 | 0 |
| 840 | adult | 26063 | 0.750144 | 0.217166 | 0.00103595 | 0.0316541 | 0 |
| 840 | child | 2840 | 0.957042 | 0.0320423 | 0 | 0.0109155 | 0 |
| 840 | elder | 6056 | 0.926024 | 0.0564729 | 0.000825627 | 0.0166777 | 0 |
| 900 | adult | 26062 | 0.685596 | 0.310836 | 0.00329982 | 0.00026859 | 0 |
| 900 | child | 2843 | 0.967992 | 0.0320084 | 0 | 0 | 0 |
| 900 | elder | 6054 | 0.944665 | 0.0536835 | 0.0016518 | 0 | 0 |
| 960 | adult | 26057 | 0.759489 | 0.210155 | 0.000844303 | 0.0295122 | 0 |
| 960 | child | 2844 | 0.943741 | 0.0414909 | 0 | 0.0147679 | 0 |
| 960 | elder | 6046 | 0.923586 | 0.0572279 | 0.000330797 | 0.0188554 | 0 |
| 1020 | adult | 26068 | 0.688085 | 0.308462 | 0.00337579 | 7.67224e-05 | 0 |
| 1020 | child | 2845 | 0.965905 | 0.0340949 | 0 | 0 | 0 |
| 1020 | elder | 6044 | 0.9363 | 0.0630377 | 0.000661813 | 0 | 0 |
| 1080 | adult | 26058 | 0.75639 | 0.212756 | 0.000690767 | 0.0301635 | 0 |
| 1080 | child | 2847 | 0.959607 | 0.0302072 | 0 | 0.0101862 | 0 |
| 1080 | elder | 6049 | 0.918003 | 0.0600099 | 0 | 0.0219871 | 0 |
| 1140 | adult | 26086 | 0.58725 | 0.404968 | 0.00770528 | 7.66695e-05 | 0 |
| 1140 | child | 2851 | 0.974044 | 0.0259558 | 0 | 0 | 0 |
| 1140 | elder | 6044 | 0.93233 | 0.0645268 | 0.00314361 | 0 | 0 |
| 1200 | adult | 26078 | 0.706841 | 0.258379 | 0.00394969 | 0.0308306 | 0 |
| 1200 | child | 2854 | 0.958304 | 0.033637 | 0 | 0.00805886 | 0 |
| 1200 | elder | 6120 | 0.913725 | 0.0655229 | 0.00245098 | 0.0183007 | 0 |
| 1260 | adult | 26127 | 0.316531 | 0.587783 | 0.001952 | 0.0937345 | 0 |
| 1260 | child | 2857 | 0.445222 | 0.334967 | 0 | 0.219811 | 0 |
| 1260 | elder | 6123 | 0.468724 | 0.361261 | 0.000816593 | 0.169198 | 0 |
| 1320 | adult | 26132 | 0 | 0.391015 | 0 | 0.608985 | 0 |
| 1320 | child | 2860 | 0 | 0.0377622 | 0 | 0.962238 | 0 |
| 1320 | elder | 6132 | 0 | 0.0521853 | 0 | 0.947815 | 0 |
| 1380 | adult | 26119 | 3.82863e-05 | 0.206516 | 0 | 0.793445 | 0 |
| 1380 | child | 2861 | 0 | 0.00734009 | 0 | 0.99266 | 0 |
| 1380 | elder | 6147 | 0 | 0.00959818 | 0 | 0.990402 | 0 |

### activity_minutes

```json
[
  {
    "kind": "TendFire",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  },
  {
    "kind": "Craft",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  },
  {
    "kind": "Repair",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  },
  {
    "kind": "HearthCircle",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  },
  {
    "kind": "Explore",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  },
  {
    "kind": "Play",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  },
  {
    "kind": "SitByFire",
    "all_life_stages_minutes": 0,
    "adult_minutes": 0,
    "all_life_stages_share": null,
    "adult_share": null,
    "all_life_stages_starts": 0
  }
]
```

### health_by_world

```json
[
  {
    "alive": 34.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 350.78333333333336,
    "wood": 90.76666666666667,
    "seed": 17100,
    "old_age_deaths": 32,
    "social_completed": 2361.0,
    "adult_days": 1478.7916666666667,
    "social_rate": 1.5965737792679833,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 61.0,
    "deaths": 8.0,
    "shelters": 7.0,
    "food": 306.06666666666666,
    "wood": 94.46666666666667,
    "seed": 17101,
    "old_age_deaths": 33,
    "social_completed": 3331.0,
    "adult_days": 1962.7916666666667,
    "social_rate": 1.6970726219033263,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 36.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 389.4166666666667,
    "wood": 91.2,
    "seed": 17102,
    "old_age_deaths": 22,
    "social_completed": 2727.0,
    "adult_days": 1550.7083333333333,
    "social_rate": 1.7585511997205578,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 27.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 286.6,
    "wood": 75.95,
    "seed": 17103,
    "old_age_deaths": 25,
    "social_completed": 2025.0,
    "adult_days": 1173.9166666666667,
    "social_rate": 1.724994675942358,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 35.0,
    "deaths": 0.0,
    "shelters": 4.0,
    "food": 304.3,
    "wood": 84.2,
    "seed": 17104,
    "old_age_deaths": 30,
    "social_completed": 2358.0,
    "adult_days": 1432.125,
    "social_rate": 1.6465043205027494,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 48.0,
    "deaths": 0.0,
    "shelters": 4.0,
    "food": 472.85,
    "wood": 80.63333333333334,
    "seed": 17105,
    "old_age_deaths": 27,
    "social_completed": 3362.0,
    "adult_days": 2040.2916666666667,
    "social_rate": 1.6478036228480404,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 64.0,
    "deaths": 0.0,
    "shelters": 7.0,
    "food": 461.76666666666665,
    "wood": 122.2,
    "seed": 17106,
    "old_age_deaths": 20,
    "social_completed": 3670.0,
    "adult_days": 2070.5416666666665,
    "social_rate": 1.7724830459018375,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 27.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 248.51666666666668,
    "wood": 81.53333333333333,
    "seed": 17107,
    "old_age_deaths": 27,
    "social_completed": 2011.0,
    "adult_days": 1183.5416666666667,
    "social_rate": 1.6991374757965145,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 27.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 246.56666666666666,
    "wood": 77.9,
    "seed": 17108,
    "old_age_deaths": 26,
    "social_completed": 1987.0,
    "adult_days": 1159.625,
    "social_rate": 1.7134849628112536,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 82.0,
    "deaths": 0.0,
    "shelters": 9.0,
    "food": 615.0,
    "wood": 130.31666666666666,
    "seed": 17109,
    "old_age_deaths": 26,
    "social_completed": 4690.0,
    "adult_days": 2667.4166666666665,
    "social_rate": 1.7582554906432566,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 19.0,
    "deaths": 1.0,
    "shelters": 1.0,
    "food": 208.08333333333334,
    "wood": 72.8,
    "seed": 17110,
    "old_age_deaths": 23,
    "social_completed": 1260.0,
    "adult_days": 907.7916666666666,
    "social_rate": 1.3879836599807225,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 91.0,
    "deaths": 0.0,
    "shelters": 13.0,
    "food": 510.5,
    "wood": 114.05,
    "seed": 17111,
    "old_age_deaths": 29,
    "social_completed": 4882.0,
    "adult_days": 2723.25,
    "social_rate": 1.7927109152666851,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 29.0,
    "deaths": 1.0,
    "shelters": 1.0,
    "food": 280.0,
    "wood": 83.78333333333333,
    "seed": 17112,
    "old_age_deaths": 27,
    "social_completed": 2215.0,
    "adult_days": 1263.0416666666667,
    "social_rate": 1.7537030317025697,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 35.0,
    "deaths": 1.0,
    "shelters": 2.0,
    "food": 387.8833333333333,
    "wood": 81.06666666666666,
    "seed": 17113,
    "old_age_deaths": 28,
    "social_completed": 2639.0,
    "adult_days": 1600.0833333333333,
    "social_rate": 1.6492890995260665,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 25.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 269.3,
    "wood": 88.2,
    "seed": 17114,
    "old_age_deaths": 26,
    "social_completed": 1782.0,
    "adult_days": 1116.3333333333333,
    "social_rate": 1.596297402209615,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 41.0,
    "deaths": 0.0,
    "shelters": 3.0,
    "food": 383.51666666666665,
    "wood": 93.33333333333333,
    "seed": 17115,
    "old_age_deaths": 32,
    "social_completed": 2881.0,
    "adult_days": 1761.125,
    "social_rate": 1.6358861523174106,
    "no_place_redraws": 0,
    "no_place_failures": 0
  }
]
```

## game_v21

16 worlds × 60 days; seeds [17100, 17101, 17102, 17103, 17104, 17105, 17106, 17107, 17108, 17109, 17110, 17111, 17112, 17113, 17114, 17115].

| Hour | Life stage | Observations | Idle | Seeking | ConsumingAgent | ConsumingOther | Activity |
|---|---|---:|---:|---:|---:|---:|---:|
| 0 | adult | 26106 | 0 | 0.0936566 | 0 | 0.906343 | 0 |
| 0 | child | 2868 | 0 | 0.00244073 | 0 | 0.997559 | 0 |
| 0 | elder | 6146 | 0 | 0.00195249 | 0 | 0.998048 | 0 |
| 60 | adult | 26108 | 0 | 0.0528574 | 0 | 0.947143 | 0 |
| 60 | child | 2870 | 0 | 0 | 0 | 1 | 0 |
| 60 | elder | 6144 | 0 | 0.00113932 | 0 | 0.998861 | 0 |
| 120 | adult | 26105 | 7.66137e-05 | 0.0277342 | 0 | 0.972189 | 0 |
| 120 | child | 2872 | 0 | 0 | 0 | 1 | 0 |
| 120 | elder | 6141 | 0 | 0.00341964 | 0 | 0.99658 | 0 |
| 180 | adult | 26105 | 0.000421375 | 0.0854242 | 0 | 0.914154 | 0 |
| 180 | child | 2874 | 0 | 0 | 0 | 1 | 0 |
| 180 | elder | 6145 | 0 | 0.085598 | 0 | 0.914402 | 0 |
| 240 | adult | 26095 | 0.000344894 | 0.096302 | 0 | 0.903353 | 0 |
| 240 | child | 2875 | 0 | 0 | 0 | 1 | 0 |
| 240 | elder | 6141 | 0 | 0.0928188 | 0 | 0.907181 | 0 |
| 300 | adult | 26088 | 0.0015716 | 0.138454 | 0 | 0.859974 | 0 |
| 300 | child | 2875 | 0 | 0.000347826 | 0 | 0.999652 | 0 |
| 300 | elder | 6134 | 0.000326052 | 0.128627 | 0 | 0.871047 | 0 |
| 360 | adult | 26099 | 0.839457 | 0.116211 | 0 | 0.0443312 | 0 |
| 360 | child | 2876 | 0.984353 | 0.0128651 | 0 | 0.00278164 | 0 |
| 360 | elder | 6125 | 0.862694 | 0.0974694 | 0 | 0.0398367 | 0 |
| 420 | adult | 26136 | 0.147 | 0.833563 | 0.0151515 | 0.000420875 | 0.0038644 |
| 420 | child | 2878 | 0.899236 | 0.0806115 | 0 | 0 | 0.0201529 |
| 420 | elder | 6130 | 0.504568 | 0.478467 | 0.00685155 | 0.000652529 | 0.00946166 |
| 480 | adult | 26008 | 0.051638 | 0.687827 | 0.00165334 | 0.0379883 | 0.220894 |
| 480 | child | 2832 | 0.0222458 | 0.0257768 | 0 | 0.0155367 | 0.936441 |
| 480 | elder | 6062 | 0.00445398 | 0.387826 | 0.000164962 | 0.0158364 | 0.591719 |
| 540 | adult | 26054 | 0.0962232 | 0.646849 | 0.00560375 | 0.00191909 | 0.249405 |
| 540 | child | 2835 | 0.148854 | 0.0388007 | 0 | 0 | 0.812346 |
| 540 | elder | 6051 | 0.112709 | 0.335647 | 0.00495786 | 0 | 0.546686 |
| 600 | adult | 26058 | 0.118812 | 0.639727 | 0.0019188 | 0.0349605 | 0.204582 |
| 600 | child | 2836 | 0.366714 | 0.0299718 | 0 | 0.0109309 | 0.592384 |
| 600 | elder | 6066 | 0.210188 | 0.268381 | 0.00164853 | 0.0201121 | 0.49967 |
| 660 | adult | 26048 | 0.165886 | 0.515779 | 0.00460688 | 0.000767813 | 0.312961 |
| 660 | child | 2840 | 0.192958 | 0.0316901 | 0 | 0 | 0.775352 |
| 660 | elder | 6063 | 0.156193 | 0.0715817 | 0.000989609 | 0 | 0.771235 |
| 720 | adult | 26059 | 0.192985 | 0.295483 | 0.000729115 | 0.0465482 | 0.464254 |
| 720 | child | 2843 | 0.267323 | 0.0439676 | 0 | 0.0126627 | 0.676046 |
| 720 | elder | 6062 | 0.209502 | 0.0704388 | 0.00082481 | 0.0191356 | 0.700099 |
| 780 | adult | 26052 | 0.169123 | 0.348342 | 0.00330109 | 0.000460617 | 0.478773 |
| 780 | child | 2844 | 0.259494 | 0.0334037 | 0 | 0 | 0.707103 |
| 780 | elder | 6065 | 0.211047 | 0.0633141 | 0.00197857 | 0 | 0.72366 |
| 840 | adult | 26050 | 0.146449 | 0.232783 | 0.00130518 | 0.0336276 | 0.585835 |
| 840 | child | 2847 | 0.244117 | 0.027046 | 0 | 0.0112399 | 0.717597 |
| 840 | elder | 6060 | 0.213531 | 0.0559406 | 0.000660066 | 0.0244224 | 0.705446 |
| 900 | adult | 26049 | 0.149718 | 0.334677 | 0.00337825 | 0.000153557 | 0.512073 |
| 900 | child | 2851 | 0.239565 | 0.0389337 | 0 | 0 | 0.721501 |
| 900 | elder | 6057 | 0.217765 | 0.0549777 | 0.000990589 | 0 | 0.726267 |
| 960 | adult | 26044 | 0.140263 | 0.228306 | 0.000729535 | 0.0300261 | 0.600676 |
| 960 | child | 2854 | 0.251577 | 0.042747 | 0 | 0.0133146 | 0.692362 |
| 960 | elder | 6050 | 0.218182 | 0.0601653 | 0.000495868 | 0.0236364 | 0.697521 |
| 1020 | adult | 26055 | 0.131836 | 0.332259 | 0.00291691 | 0 | 0.532988 |
| 1020 | child | 2855 | 0.265149 | 0.0353765 | 0 | 0 | 0.699475 |
| 1020 | elder | 6049 | 0.209125 | 0.0684411 | 0.000991899 | 0 | 0.721442 |
| 1080 | adult | 26045 | 0.729929 | 0.237474 | 0.000345556 | 0.0322519 | 0 |
| 1080 | child | 2856 | 0.953431 | 0.0360644 | 0 | 0.0105042 | 0 |
| 1080 | elder | 6055 | 0.916268 | 0.0635838 | 0.000165153 | 0.0199835 | 0 |
| 1140 | adult | 26075 | 0.594861 | 0.397009 | 0.00809204 | 3.83509e-05 | 0 |
| 1140 | child | 2858 | 0.969559 | 0.0304409 | 0 | 0 | 0 |
| 1140 | elder | 6050 | 0.930579 | 0.0669421 | 0.00247934 | 0 | 0 |
| 1200 | adult | 26067 | 0.687536 | 0.272835 | 0.0031841 | 0.0364445 | 0 |
| 1200 | child | 2860 | 0.961538 | 0.0311189 | 0 | 0.00734266 | 0 |
| 1200 | elder | 6125 | 0.907918 | 0.0690612 | 0.00244898 | 0.0205714 | 0 |
| 1260 | adult | 26110 | 0.317082 | 0.582574 | 0.00199157 | 0.0983531 | 0 |
| 1260 | child | 2862 | 0.446191 | 0.343466 | 0 | 0.210342 | 0 |
| 1260 | elder | 6128 | 0.463283 | 0.359987 | 0.000979112 | 0.175751 | 0 |
| 1320 | adult | 26112 | 0 | 0.380055 | 0 | 0.619945 | 0 |
| 1320 | child | 2865 | 0 | 0.0422339 | 0 | 0.957766 | 0 |
| 1320 | elder | 6137 | 0 | 0.0477432 | 0 | 0.952257 | 0 |
| 1380 | adult | 26099 | 7.66313e-05 | 0.196406 | 0 | 0.803517 | 0 |
| 1380 | child | 2867 | 0 | 0.00802232 | 0 | 0.991978 | 0 |
| 1380 | elder | 6147 | 0 | 0.00911014 | 0 | 0.99089 | 0 |

### activity_minutes

```json
[
  {
    "kind": "TendFire",
    "all_life_stages_minutes": 793581,
    "adult_minutes": 793246,
    "all_life_stages_share": 0.1281263224039631,
    "adult_share": 0.24937345879666364,
    "all_life_stages_starts": 25285
  },
  {
    "kind": "Craft",
    "all_life_stages_minutes": 607304,
    "adult_minutes": 606502,
    "all_life_stages_share": 0.09805127403657145,
    "adult_share": 0.1906665794811371,
    "all_life_stages_starts": 20005
  },
  {
    "kind": "Repair",
    "all_life_stages_minutes": 618523,
    "adult_minutes": 617841,
    "all_life_stages_share": 0.09986261933219982,
    "adult_share": 0.1942312311141682,
    "all_life_stages_starts": 18854
  },
  {
    "kind": "HearthCircle",
    "all_life_stages_minutes": 1859881,
    "adult_minutes": 883159,
    "all_life_stages_share": 0.3002840449040555,
    "adult_share": 0.27763948951195805,
    "all_life_stages_starts": 46162
  },
  {
    "kind": "Explore",
    "all_life_stages_minutes": 275472,
    "adult_minutes": 274860,
    "all_life_stages_share": 0.04447588120842677,
    "adult_share": 0.0864079855238488,
    "all_life_stages_starts": 16545
  },
  {
    "kind": "Play",
    "all_life_stages_minutes": 1042307,
    "adult_minutes": 5348,
    "all_life_stages_share": 0.16828397192713482,
    "adult_share": 0.0016812555722241992,
    "all_life_stages_starts": 14884
  },
  {
    "kind": "SitByFire",
    "all_life_stages_minutes": 996671,
    "adult_minutes": 0,
    "all_life_stages_share": 0.16091588618764852,
    "adult_share": 0.0,
    "all_life_stages_starts": 18053
  }
]
```

### health_by_world

```json
[
  {
    "alive": 34.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 356.46666666666664,
    "wood": 89.75,
    "seed": 17100,
    "old_age_deaths": 32,
    "social_completed": 2407.0,
    "adult_days": 1478.8333333333333,
    "social_rate": 1.6276343964837148,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 61.0,
    "deaths": 6.0,
    "shelters": 7.0,
    "food": 299.45,
    "wood": 97.41666666666667,
    "seed": 17101,
    "old_age_deaths": 33,
    "social_completed": 3328.0,
    "adult_days": 1939.5,
    "social_rate": 1.7159061613817994,
    "no_place_redraws": 430,
    "no_place_failures": 0
  },
  {
    "alive": 36.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 377.5,
    "wood": 92.48333333333333,
    "seed": 17102,
    "old_age_deaths": 22,
    "social_completed": 2818.0,
    "adult_days": 1550.7083333333333,
    "social_rate": 1.817234059703899,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 27.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 284.65,
    "wood": 71.8,
    "seed": 17103,
    "old_age_deaths": 25,
    "social_completed": 1921.0,
    "adult_days": 1173.9166666666667,
    "social_rate": 1.6364023567828494,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 33.0,
    "deaths": 0.0,
    "shelters": 2.0,
    "food": 306.8,
    "wood": 87.3,
    "seed": 17104,
    "old_age_deaths": 30,
    "social_completed": 2334.0,
    "adult_days": 1432.125,
    "social_rate": 1.629746006808065,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 48.0,
    "deaths": 0.0,
    "shelters": 4.0,
    "food": 451.2,
    "wood": 83.66666666666667,
    "seed": 17105,
    "old_age_deaths": 27,
    "social_completed": 3350.0,
    "adult_days": 2040.375,
    "social_rate": 1.6418550511548122,
    "no_place_redraws": 18,
    "no_place_failures": 0
  },
  {
    "alive": 64.0,
    "deaths": 0.0,
    "shelters": 7.0,
    "food": 453.23333333333335,
    "wood": 124.36666666666666,
    "seed": 17106,
    "old_age_deaths": 20,
    "social_completed": 3638.0,
    "adult_days": 2095.25,
    "social_rate": 1.7363083164300204,
    "no_place_redraws": 833,
    "no_place_failures": 0
  },
  {
    "alive": 27.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 247.08333333333334,
    "wood": 79.36666666666666,
    "seed": 17107,
    "old_age_deaths": 27,
    "social_completed": 1951.0,
    "adult_days": 1183.5,
    "social_rate": 1.6485002112378537,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 27.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 260.1,
    "wood": 78.08333333333333,
    "seed": 17108,
    "old_age_deaths": 26,
    "social_completed": 1941.0,
    "adult_days": 1159.6666666666667,
    "social_rate": 1.6737568266743317,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 82.0,
    "deaths": 1.0,
    "shelters": 9.0,
    "food": 617.7833333333333,
    "wood": 125.0,
    "seed": 17109,
    "old_age_deaths": 26,
    "social_completed": 4692.0,
    "adult_days": 2663.0833333333335,
    "social_rate": 1.7618675094658447,
    "no_place_redraws": 979,
    "no_place_failures": 0
  },
  {
    "alive": 19.0,
    "deaths": 1.0,
    "shelters": 1.0,
    "food": 212.33333333333334,
    "wood": 72.11666666666666,
    "seed": 17110,
    "old_age_deaths": 23,
    "social_completed": 1328.0,
    "adult_days": 907.7916666666666,
    "social_rate": 1.4628907146463488,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 91.0,
    "deaths": 0.0,
    "shelters": 13.0,
    "food": 491.8,
    "wood": 114.45,
    "seed": 17111,
    "old_age_deaths": 29,
    "social_completed": 4906.0,
    "adult_days": 2723.25,
    "social_rate": 1.8015239144404664,
    "no_place_redraws": 909,
    "no_place_failures": 0
  },
  {
    "alive": 29.0,
    "deaths": 4.0,
    "shelters": 1.0,
    "food": 279.8833333333333,
    "wood": 80.6,
    "seed": 17112,
    "old_age_deaths": 27,
    "social_completed": 2166.0,
    "adult_days": 1250.5416666666667,
    "social_rate": 1.7320494452403956,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 35.0,
    "deaths": 1.0,
    "shelters": 2.0,
    "food": 390.6,
    "wood": 82.6,
    "seed": 17113,
    "old_age_deaths": 28,
    "social_completed": 2629.0,
    "adult_days": 1600.125,
    "social_rate": 1.6429966408874306,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 25.0,
    "deaths": 0.0,
    "shelters": 1.0,
    "food": 271.26666666666665,
    "wood": 86.51666666666667,
    "seed": 17114,
    "old_age_deaths": 26,
    "social_completed": 1783.0,
    "adult_days": 1116.3333333333333,
    "social_rate": 1.5971931919976112,
    "no_place_redraws": 0,
    "no_place_failures": 0
  },
  {
    "alive": 41.0,
    "deaths": 0.0,
    "shelters": 3.0,
    "food": 381.26666666666665,
    "wood": 92.25,
    "seed": 17115,
    "old_age_deaths": 32,
    "social_completed": 2914.0,
    "adult_days": 1761.125,
    "social_rate": 1.6546241748882107,
    "no_place_redraws": 1,
    "no_place_failures": 0
  }
]
```

## identity

```json
{
  "verdict": "PENDING",
  "source": null,
  "sha256": null,
  "payload": null,
  "error": "identity evidence was not supplied"
}
```

## performance

```json
{
  "verdict": "PENDING",
  "source": null,
  "sha256": null,
  "payload": null,
  "error": "performance evidence was not supplied"
}
```
