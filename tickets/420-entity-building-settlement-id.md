# T-420: Entity/Building Settlement ID

## Action: DISPATCH (Codex)
## Files: scripts/core/entity_data.gd, scripts/core/building_data.gd

### entity_data.gd
- Add `var settlement_id: int = 0` field
- Update to_dict(): add "settlement_id": settlement_id
- Update from_dict(): e.settlement_id = data.get("settlement_id", 0)

### building_data.gd
- Add `var settlement_id: int = 0` field
- Update to_dict(): add "settlement_id": settlement_id
- Update from_dict(): b.settlement_id = data.get("settlement_id", 0)
