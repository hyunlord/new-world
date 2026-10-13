"""Pinned natural capture contract from Part 7, with no intervention substitution."""
from typing import Final

INHERITED_STEMS: Final = {f'l_d{day}_1200_F2' for day in (10, 18, 24, 32, 40)} | {
    'l_d24_1200_F1', 'l_d24_2300_F2N', 'l_d10_1500_F2', 'l_d10_1500_F5'}
LONG_STEMS: Final = INHERITED_STEMS | {
    *(f'l_d{day}_0200_F2N' for day in (23, 25, 27, 28, 29)),
    'l_d15_1500_F2', 'l_d15_1530_FC', 'l_d16_1500_FC',
    'l_d28_0200_F2Z', 'l_d29_0630_F2', 'l_d29_0715_F2', 'l_d27_0715_F2'}

LONG_STEMS |= {*(f'l_d{day}_0200_F2N' for day in (56,57,84,85)), 'l_d57_0301_DE', 'l_d85_0401_DE'}
