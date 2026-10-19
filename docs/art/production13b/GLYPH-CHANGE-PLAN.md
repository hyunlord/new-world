# Part1 glyph rule implementation plan

Prepared during Part0; implemented in Part1.

User-authorized replacement: evaluate distinct syllables at12px design scale with a minimum1 differing binary-alpha pixel; actual Life normal/bold size24 and both integer multiples of12. Keep prior production13 evidence immutable.

Implementation boundary:
1. Change only Life normal/bold overrides48->24 in inspector.
2. Copy the existing atlas/mask implementation to a p13b-named helper, preserving TextServer glyph shaping, baseline-origin alignment, alpha>=0.5 threshold, fallback/missing-glyph/error checks, full PersonNames table and Latin capital carry closure.
3. Render12px design masks and independently render masks at actual normal/bold size. Report actual screen differences, not only an assumed scale-squared multiplier. Report all pairs and10 closest design pairs per script/weight, plus source font hashes/settings and both actual sizes.
4. Replace the existing p13 gate entrypoint with a thin explicit forwarder to the p13b rule. This is the user's replaced specification, not a relaxation to hide a failure. Keep the gate's existing P13_GLYPH_OUT contract and preserve historical24/36/48 outputs. Name helper without test_ prefix so the same rule does not run twice in full discovery.
5. Run the design-glyph check headless using CPU TextServer atlases; leader runs both existing Life layout/scroll tests windowed at24px. Preserve every target invocation and three-strike counts. The CPU atlas check is machine glyph evidence only; screenshots and fresh fact reading remain necessary.

Do not escalate font size in response to the obsolete12-screen-pixel rule. Do not alter PersonNames, font assets or historical evidence to obtain a pass.
