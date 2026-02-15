# T-920: Popup Close Fix (3-way guarantee)

## Status: TODO

## Description
Fix stats/entity/building detail panels not closing.
3-way close: keyboard (G/E/Esc), [X] button, dim overlay background click.
Add Esc key handler in main.gd. Add toggle_stats (show/hide) in hud.gd.

## Files
- main.gd: KEY_ESCAPE handler
- hud.gd: toggle_stats toggle, close_all_popups
- stats_detail_panel.gd: background click close
- entity_detail_panel.gd: background click close
- building_detail_panel.gd: background click close

## Done
- [ ] G/E/Esc close works
- [ ] [X] click works
- [ ] Dim overlay click works
- [ ] docs/ updated
