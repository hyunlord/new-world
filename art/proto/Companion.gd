# Four procedural ink-language companion examples beside generated art.
# Bridge methods: none. Jitter is deterministic and constrained to half a pixel.
extends Node2D

const INK: Color = Color("#30291f")

func _line(a: Vector2, b: Vector2, salt: int) -> void:
    var offset: float = (float((salt * 37) % 101) / 100.0 - 0.5)
    var mid: Vector2 = (a + b) * 0.5 + Vector2(offset, -offset)
    draw_polyline(PackedVector2Array([a, mid, b]), INK, 1.5, true)

func _box(rect: Rect2, dashed: bool, salt: int) -> void:
    var points: Array[Vector2] = [rect.position, Vector2(rect.end.x, rect.position.y), rect.end, Vector2(rect.position.x, rect.end.y), rect.position]
    for index: int in range(4):
        if dashed:
            var distance: float = points[index].distance_to(points[index + 1])
            var along: float = 0.0
            while along < distance:
                _line(points[index].lerp(points[index + 1], along / distance), points[index].lerp(points[index + 1], minf(along + 5.0, distance) / distance), salt + index)
                along += 9.0
        else:
            _line(points[index], points[index + 1], salt + index)

func _draw() -> void:
    var wash: Color = Color(0.40, 0.49, 0.47, 0.15)
    draw_rect(Rect2(0, 0, 96, 64), wash)
    _box(Rect2(0, 0, 96, 64), true, 1)
    _box(Rect2(120, 0, 96, 64), true, 8)
    _line(Vector2(128, 8), Vector2(208, 8), 21)
    _line(Vector2(128, 8), Vector2(128, 56), 22)
    _line(Vector2(208, 8), Vector2(208, 56), 23)
    _box(Rect2(240, 0, 64, 64), false, 30)
    draw_rect(Rect2(330, 20, 60, 8), wash)
    _box(Rect2(330, 20, 96, 8), false, 42)
