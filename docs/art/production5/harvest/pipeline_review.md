# Independent pipeline review and resolution

Round1 found four concrete gaps: hard key/conversion errors discarded; normalized installer pixels not joined to original review binding; content-only changes missed by metadata-only coverage; and ordered animation pixels/review artifacts incompletely bound.

Round2 verified prepared candidate, original pixel, ordered animation and content baseline bindings. An independent mutation of the already-covered V_a2_05 rendered mask still passed initially. The final fix compares every harvested adult rendered mask to its approved mask (with separately verified later approved overrides). The identical read-only mutation now returns `V_a2_05:installed_mask_changed`.

Independent read-only recomputation of actual final A20: PASS,2 installations,74 bound frames,61 changed villager frames,20 animation groups,zero errors. Reviewer separately ran10 harvest tests and5 binding tests. Full executor suite:274 passed. No remaining blocker in these four review concerns. This establishes installation integrity, not visual/gameplay acceptance.
