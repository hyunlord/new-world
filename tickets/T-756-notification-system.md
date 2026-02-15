# T-756: Notification System

**Priority:** Medium | **Status:** Open

## Description
Replace single toast with multi-notification manager. Up to 5 stacked notifications, 3s duration, fade out. Event-driven.

## Triggers
- settlement_founded: orange
- population milestones (50/100/150/200): green
- entity_starved (5+ simultaneous): red famine warning
- building_completed: yellow
- game_saved / game_loaded: white

## Done Definition
- [ ] Up to 5 simultaneous notifications
- [ ] Proper fade out animation
- [ ] All event triggers working
- [ ] docs/VISUAL_GUIDE.md, SYSTEMS.md updated
- [ ] CHANGELOG.md updated
- [ ] Gate PASS
