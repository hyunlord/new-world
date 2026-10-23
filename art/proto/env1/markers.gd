# Purpose: draw the specified half-zoom people markers above environment sprites.
# Bridge methods called: none.
extends Node2D

const AXE = preload("res://axe.png")
const LOG = preload("res://log.png")
var positions: Array[Vector2] = []
var colours: Array[Color] = []
var phase := 0

func _draw() -> void:
    for index: int in positions.size():
        var point := positions[index]
        draw_circle(point, 5.0, Color("261f19"))
        draw_circle(point, 4.0, colours[index])
        var action := (phase + index) % 5
        match action:
            2:
                draw_texture(AXE, point + Vector2(6, -7))
            3:
                draw_texture(LOG, point + Vector2(6, -7))
            4:
                draw_string(ThemeDB.fallback_font, point + Vector2(5, -3), "zz", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color("eddfbe"))
