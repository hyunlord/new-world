"""Repair imported seam geometry and discard verified subpixel remesh debris."""
import bmesh
import bpy
from mathutils import Vector


def landmark(mesh: bpy.types.Object, quadruped: bool) -> Vector:
    """Return crown or front-trunk withers surface point; front is Blender -Y."""
    points = [v.co for v in mesh.data.vertices]
    if quadruped:
        low = Vector(tuple(min(p[i] for p in points) for i in range(3)))
        high = Vector(tuple(max(p[i] for p in points) for i in range(3)))
        y = low.y+(high.y-low.y)*.4
        points = [p for p in points if abs(p.y-y) < (high.y-low.y)*.04 and abs(p.x-(low.x+high.x)/2) < (high.x-low.x)*mesh.get("withers_width_fraction", .4)/2]
    assert points, "No height-reference surface points"
    return max(points, key=lambda point: point.z).copy()


def weld(mesh: bpy.types.Object) -> None:
    """Merge coincident geometry while preserving per-loop UV coordinates."""
    topology = bmesh.new()
    topology.from_mesh(mesh.data)
    bmesh.ops.remove_doubles(topology, verts=list(topology.verts), dist=1e-5)
    bmesh.ops.delete(topology, geom=[v for v in topology.verts if not v.link_faces], context="VERTS")
    topology.normal_update()
    topology.to_mesh(mesh.data)
    topology.free()


def debris(mesh: bpy.types.Object) -> None:
    """Remove tiny disconnected ≤3-vertex remnants, below3% of body height."""
    topology = bmesh.new()
    topology.from_mesh(mesh.data)
    height = max(v.co.z for v in topology.verts)-min(v.co.z for v in topology.verts)
    pending = set(topology.verts)
    discard = []
    while pending:
        component = {pending.pop()}
        queue = list(component)
        while queue:
            for edge in queue.pop().link_edges:
                for vertex in edge.verts:
                    if vertex in pending:
                        pending.remove(vertex)
                        component.add(vertex)
                        queue.append(vertex)
        extent = max(max(v.co[i] for v in component)-min(v.co[i] for v in component) for i in range(3))
        if len(component) <= 3 and extent < .03*height:
            discard.extend(component)
    mesh["debris_vertices_removed"] = len(discard)
    bmesh.ops.delete(topology, geom=discard, context="VERTS")
    topology.to_mesh(mesh.data)
    topology.free()
