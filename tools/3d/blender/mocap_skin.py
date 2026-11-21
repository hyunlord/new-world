"""Offline garment binding and surface relaxation for the mocap VAT bake."""

from itertools import product
from typing import Final, Sequence

import numpy as np
from numpy.typing import NDArray

from mocap_deform import MAX_BODY_STRETCH, _double_areas, _edge_lengths, body_edge_geometry, joint_geometry

FloatArray = NDArray[np.float64]
IndexArray = NDArray[np.intp]
Mask = NDArray[np.bool_]

_STRETCH: Final = 1.35
_AREA: Final = .35
_RINGS: Final = 6
_SWEEPS: Final = 60
_SINGULAR_MAX: Final = 1.22
_FOOT_HEIGHT: Final = .10
PLANTED_SOLE_M: Final = .04


def stabilize_garment_weights(points: FloatArray, rows: Sequence[dict[str, float]]) -> list[dict[str, float]]:
    """Bind the shared hem to the pelvis and feather weight toward each thigh."""
    bottom = points[:, 2].min()
    height = points[:, 2].max() - bottom
    x = points[:, 0] / height
    z = (points[:, 2] - bottom) / height

    def ramp(values: FloatArray, low: float, high: float) -> FloatArray:
        t = np.clip((values - low) / (high - low), 0., 1.)
        return t * t * (3. - 2. * t)

    waist = ramp(z, .34, .44)
    hem = ramp(z, .20, .29) * (1. - ramp(np.abs(x), .03, .19))
    pelvis = waist + (1. - waist) * hem
    shared = ramp(z, .20, .29)
    left_share = ramp(x, -.065, .065)
    corrected = []
    for row, bind, crossfade, share in zip(rows, pelvis, shared, left_share):
        weights = row.copy()
        for segment in ("upper_leg", "lower_leg"):
            right, left = f"{segment}_R", f"{segment}_L"
            right_weight, left_weight = row.get(right, 0.), row.get(left, 0.)
            total = right_weight + left_weight
            if not total or not (bind or crossfade):
                continue
            weights["hips"] = weights.get("hips", 0.) + bind * total
            weights[right] = (1. - bind) * ((1. - crossfade) * right_weight + crossfade * (1. - share) * total)
            weights[left] = (1. - bind) * ((1. - crossfade) * left_weight + crossfade * share * total)
        corrected.append(weights)
    return corrected


def _scatter(vertices: IndexArray, vectors: FloatArray, count: int) -> FloatArray:
    """Sum the three coordinate contributions at each stencil vertex."""
    return np.column_stack([np.bincount(vertices, component, minlength=count)
                            for component in vectors.reshape(-1, 3).T])


def _face_stencils(rest: FloatArray, faces: IndexArray) -> tuple[FloatArray, FloatArray]:
    """Express each face's two tangents as an area-weighted linear operator."""
    a, b, c = np.moveaxis(rest[faces], 1, 0)
    ab, ac = b - a, c - a
    base = np.linalg.norm(ab, axis=1)
    along = np.sum(ab * ac, axis=1) / base
    height = np.linalg.norm(ac - ab * (along / base)[:, None], axis=1)
    scale = np.sqrt(base * height)
    tangent = np.column_stack((-1. / base, 1. / base, np.zeros_like(base)))
    other = -along / (base * height)
    bitangent = np.column_stack((-other - 1. / height, other, 1. / height))
    return np.stack((tangent, bitangent), axis=1) * scale[:, None, None], scale


class _JointSkin:
    """Factor the shared least-squares system for all poses of one mesh."""

    def __init__(self, rest: FloatArray, triangles: IndexArray, weights: FloatArray, fixed: Mask) -> None:
        edges, lengths, self.joint_edges, self.joint_lengths, self.faces, self.areas = joint_geometry(
            rest, triangles, weights)
        self.fixed = fixed
        self.count = len(rest)
        self.mobility = np.zeros(self.count)
        band = np.zeros(self.count, dtype=bool)
        band[self.joint_edges] = True
        band[self.faces] = True
        self.mobility[band] = 1.
        for ring in range(1, _RINGS + 1):
            band[edges[band[edges].any(axis=1)]] = True
            np.maximum(self.mobility, band * (1. - ring / (_RINGS + 1.)), out=self.mobility)
        self.mobility[fixed] = 0.
        self.move = np.flatnonzero(self.mobility > 0.)
        if not len(self.move):
            return

        selected = (self.mobility[edges].max(axis=1) > 0.) & (lengths > 1.e-5)
        self.edges = edges[selected]
        edge_keys = edges[:, 0] * self.count + edges[:, 1]
        joint_keys = self.joint_edges[:, 0] * self.count + self.joint_edges[:, 1]
        self.core = np.isin(edge_keys[selected], joint_keys)
        self.edge_lower = .62 * lengths[selected]
        self.edge_upper = _SINGULAR_MAX * lengths[selected]
        faces = self.faces[self.mobility[self.faces].max(axis=1) > 0.]
        stencils, scale = _face_stencils(rest, faces)
        self.scale = scale[:, None]
        self.vertices = np.vstack((self.edges[:, [0, 1, 1]], np.repeat(faces, 2, axis=0)))
        edge_stencils = np.broadcast_to((-1., 1., 0.), (len(self.edges), 3))
        self.stencils = np.vstack((edge_stencils, stencils.reshape(-1, 3)))
        self.spring = .014 / (.3 + self.mobility[self.move, None]) ** 2

        columns = np.full(self.count, -1, dtype=np.intp)
        columns[self.move] = np.arange(len(self.move))
        row_columns = columns[self.vertices]
        gram = np.diag(self.spring[:, 0])
        for i, j in product(range(3), repeat=2):
            a, b = row_columns[:, i], row_columns[:, j]
            keep = (a >= 0) & (b >= 0)
            np.add.at(gram, (a[keep], b[keep]), self.stencils[keep, i] * self.stencils[keep, j])
        self.inverse = np.linalg.inv(gram)
        self.flat_vertices = self.vertices.ravel()

    def _apply(self, points: FloatArray) -> FloatArray:
        return np.einsum("ij,ijk->ik", self.stencils, points[self.vertices])

    def pose(self, raw: FloatArray, initial: FloatArray) -> FloatArray:
        """Alternate local edge/face projections with the coupled position solve."""
        if not len(self.move):
            return raw.copy()
        count = len(self.edges)
        raw_lengths = _edge_lengths(raw, self.edges)
        lower = np.where(self.core, self.edge_lower, np.minimum(self.edge_lower, raw_lengths))
        upper = np.where(self.core, self.edge_upper, np.maximum(self.edge_upper, raw_lengths))
        fixed_points = raw.copy()
        fixed_points[self.move] = 0.
        offset = self._apply(fixed_points)
        rhs = self.spring * raw[self.move]
        current = initial.copy()
        current[self.fixed] = raw[self.fixed]
        values = self._apply(current)
        dual = np.zeros_like(values)
        for _ in range(_SWEEPS):
            targets = values + dual
            edge_targets = targets[:count]
            edge_lengths = np.linalg.norm(edge_targets, axis=1)
            edge_targets *= (np.clip(edge_lengths, lower, upper) / np.maximum(edge_lengths, 1.e-12))[:, None]
            u, singular, vh = np.linalg.svd(targets[count:].reshape(-1, 2, 3), full_matrices=False)
            singular = np.clip(singular, .67 * self.scale, _SINGULAR_MAX * self.scale)
            targets[count:] = ((u * singular[:, None, :]) @ vh).reshape(-1, 3)
            weighted = ((targets - dual - offset)[:, None, :] * self.stencils[:, :, None])
            current[self.move] = self.inverse @ (rhs + _scatter(
                self.flat_vertices, weighted, self.count)[self.move])
            values = self._apply(current)
            dual += values - targets
        return current

    def deficits(self, pose: FloatArray, stretch: float, area: float) -> tuple[FloatArray, FloatArray]:
        return (np.maximum(0., _edge_lengths(pose, self.joint_edges) / self.joint_lengths - stretch),
                np.maximum(0., area - _double_areas(pose, self.faces) / self.areas))

    def finish(self, pose: FloatArray) -> FloatArray:
        """Remove residual length and area violations after the global solve."""
        edges, faces = self.joint_edges, self.faces
        i, j = edges.T
        wi, wj = self.mobility[i, None], self.mobility[j, None]
        face_weights = self.mobility[faces, None]
        caps = np.where(self.joint_lengths < .008, 1.22, 1.28) * self.joint_lengths
        area_floors = .40 * self.areas
        for _ in range(24):
            delta = pose[j] - pose[i]
            lengths = np.linalg.norm(delta, axis=1)
            long = lengths > caps
            if long.any():
                step = (lengths[long] - caps[long]) / lengths[long]
                force = delta[long] * step[:, None] / np.maximum(wi[long] + wj[long], 1.e-10)
                ends = edges[long].ravel()
                forces = np.stack((wi[long] * force, -wj[long] * force), axis=1)
                degree = np.maximum(1, np.bincount(ends, minlength=self.count))
                pose += .94 * _scatter(ends, forces, self.count) / degree[:, None]
            a, b, c = np.moveaxis(pose[faces], 1, 0)
            ab, ac = b - a, c - a
            normals = np.cross(ab, ac)
            areas = np.linalg.norm(normals, axis=1)
            thin = areas < area_floors
            if thin.any():
                normal = normals[thin] / np.maximum(areas[thin, None], 1.e-10)
                gb, gc = np.cross(ac[thin], normal), np.cross(normal, ab[thin])
                gradients = np.stack((-gb - gc, gb, gc), axis=1)
                weight = face_weights[thin]
                stiffness = np.maximum(1.e-10, np.sum(weight * gradients ** 2, axis=(1, 2)))
                change = .85 * (area_floors[thin] - areas[thin]) / stiffness
                corners = faces[thin].ravel()
                degree = np.maximum(1, np.bincount(corners, minlength=self.count))
                pose += _scatter(corners, weight * gradients * change[:, None, None],
                                 self.count) / degree[:, None]
            if not long.any() and not thin.any():
                break
        return pose

    def _grid_step(self, packed: FloatArray, full: FloatArray, vertex: int, radius: int) -> bool:
        """Pick the nearest useful half-float lattice candidate for one vertex."""
        edges = (self.joint_edges == vertex).any(axis=1)
        faces = (self.faces == vertex).any(axis=1)
        codes = packed[vertex].astype(np.float16).view(np.int16)
        offsets = np.array(list(product(range(-radius, radius + 1), repeat=3)), dtype=np.int16)
        candidates = (codes + offsets).view(np.float16).astype(np.float64)
        candidates = candidates[np.isfinite(candidates).all(axis=1)]
        candidates = np.vstack((packed[vertex], candidates[np.linalg.norm(candidates - full[vertex], axis=1) <= .003]))
        e, f = self.joint_edges[edges], self.faces[faces]
        e_pos = np.where((e == vertex)[None, :, :, None], candidates[:, None, None], packed[e])
        f_pos = np.where((f == vertex)[None, :, :, None], candidates[:, None, None], packed[f])
        length = np.linalg.norm(e_pos[:, :, 1] - e_pos[:, :, 0], axis=2)
        area = np.linalg.norm(np.cross(f_pos[:, :, 1] - f_pos[:, :, 0], f_pos[:, :, 2] - f_pos[:, :, 0]), axis=2)
        stretch_error = np.maximum(0., length / self.joint_lengths[edges] - 1.345)
        area_error = np.maximum(0., .355 - area / self.areas[faces])
        score = (np.sum(stretch_error ** 2, axis=1) + np.sum(area_error ** 2, axis=1)
                 + 1.e-4 * np.sum((candidates - full[vertex]) ** 2, axis=1))
        best = int(score.argmin())
        if score[best] >= score[0] - 1.e-12:
            return False
        packed[vertex] = candidates[best]
        return True

    def repair_encoding(self, full: FloatArray) -> FloatArray:
        """Repair guard violations introduced by absolute float16 positions."""
        packed = full.astype(np.float16).astype(np.float64)
        initial = packed.copy()
        for sweep in range(20):
            stretch, area = self.deficits(packed, 1.345, .355)
            vertices = np.unique(np.r_[self.joint_edges[stretch > 0.].ravel(), self.faces[area > 0.].ravel()])
            if not len(vertices):
                break
            ordered = vertices[np.argsort(np.min(np.abs(full[vertices]), axis=1), kind="stable")]
            radius = 1 if sweep < 10 else 2
            moved = False
            for vertex in ordered[~self.fixed[ordered]]:
                moved |= self._grid_step(packed, full, int(vertex), radius)
            if not moved and radius == 2:
                break
        chosen = (packed != initial).any(axis=1)
        # Merging corrected half-floats into full precision can add a deficit
        # along their boundary. Include adjacent half-floats only as needed.
        for _ in range(20):
            result = np.where(chosen[:, None], packed, full)
            stretch, area = self.deficits(result, _STRETCH, _AREA)
            if not stretch.any() and not area.any():
                break
            chosen[self.joint_edges[stretch > 0.]] = True
            chosen[self.faces[area > 0.]] = True
            chosen[self.fixed] = False
        return result


def relax_joint_skin(
    rest: FloatArray, frames: FloatArray, triangles: IndexArray, arm_weights: FloatArray,
    grips: Sequence[int] | IndexArray, *, loop: bool,
) -> FloatArray:
    """Relax posed joint surfaces in world metres, keeping palms and feet fixed.

    The caller verifies the original guard independently on the returned poses
    and on the encoded VAT. Loop samples use circular temporal initialization.
    """
    fixed = rest[:, 2] <= _FOOT_HEIGHT
    fixed[np.asarray(grips, dtype=np.intp)] = True
    solver = _JointSkin(rest, triangles, arm_weights, fixed)
    if not len(solver.move):
        return frames.copy()
    duplicate_endpoint = loop and len(frames) > 1 and np.array_equal(frames[0], frames[-1])
    raw = frames[:-1] if duplicate_endpoint else frames
    independent = np.stack([solver.pose(pose, pose) for pose in raw])
    delta = independent - raw
    previous, next_ = np.roll(delta, 1, axis=0), np.roll(delta, -1, axis=0)
    if not loop:
        previous[0], next_[-1] = delta[0], delta[-1]
    warm = raw + .25 * previous + .50 * delta + .25 * next_
    poses = np.stack([solver.repair_encoding(solver.finish(solver.pose(pose, initial)))
                      for pose, initial in zip(raw, warm)])
    if duplicate_endpoint:
        poses = np.concatenate((poses, poses[:1]), axis=0)
    # Leave signed zeros and all other original palm/foot bits untouched.
    poses[:, fixed] = frames[:, fixed]
    return poses


def relax_body_skin(
    rest: FloatArray, frames: FloatArray, triangles: IndexArray, grips: Sequence[int] | IndexArray,
    *, loop: bool, arm_weights: FloatArray | None = None,
) -> FloatArray:
    """Relax long edges outside the planted soles, palms and repaired arm joints."""
    edges, lengths = body_edge_geometry(rest, triangles)
    # The boot collar blends into the shin. Only the contact surface stays pinned.
    fixed = rest[:, 2] <= PLANTED_SOLE_M
    fixed[np.asarray(grips, dtype=np.intp)] = True
    if arm_weights is not None:
        _, _, joint_edges, _, joint_faces, _ = joint_geometry(rest, triangles, arm_weights)
        fixed[joint_edges] = True
        fixed[joint_faces] = True

    duplicate = loop and len(frames) > 1 and np.array_equal(frames[0], frames[-1])
    poses = frames.copy()
    movable = ~fixed[edges]
    projectable = movable.any(axis=1)
    edges, lengths, movable = edges[projectable], lengths[projectable], movable[projectable]
    # Edges in one color have disjoint ends, allowing in-place projection without write conflicts.
    used = [0] * len(rest)
    colors: list[list[int]] = []
    for edge, (a, b) in enumerate(edges):
        neighbor_colors = used[a] | used[b]
        free = (neighbor_colors + 1) & ~neighbor_colors
        color = free.bit_length() - 1
        if color == len(colors):
            colors.append([])
        colors[color].append(edge)
        used[a] |= free
        used[b] |= free

    freedom = movable / movable.sum(axis=1, keepdims=True)
    limits = (MAX_BODY_STRETCH - .35) * lengths  # Reserve room for float16 quantization.
    groups = [(edges[indices].T, limits[indices], freedom[indices].T) for indices in colors]
    for pose in poses[:-1] if duplicate else poses:
        for _ in range(192):
            changed = False
            for (a, b), caps, (weight_a, weight_b) in groups:
                links = pose[b] - pose[a]
                distance = np.linalg.norm(links, axis=1)
                long = distance > caps + 1.e-6
                if long.any():
                    move = links[long] * (1. - caps[long] / distance[long])[:, None]
                    pose[a[long]] += weight_a[long, None] * move
                    pose[b[long]] -= weight_b[long, None] * move
                    changed = True
            if not changed:
                break
    if duplicate:
        poses[-1] = poses[0]
    poses[:, fixed] = frames[:, fixed]
    return poses
