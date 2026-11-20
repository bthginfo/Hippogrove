"""Build the original village models with blender -b -P art/build_villages.py."""

from dataclasses import dataclass
from math import cos, pi, sin, tau
from pathlib import Path
from random import Random

import bpy
from mathutils import Vector


OUTPUT = Path(__file__).resolve().parents[1] / "assets" / "models" / "villages"


@dataclass(frozen=True)
class Style:
    file: str
    wall: int
    stone: tuple[int, int]
    timber: int
    trim: int
    roof: tuple[int, int]
    bloom: int
    pitch: float


STYLES = (
    Style("valley", 0xE4CDA0, (0xBEB39C, 0xDBCEB5), 0x75513B, 0x557961,
          (0xA25138, 0xD08453), 0xF3DC87, 0.89),
    Style("fjord", 0x8B6551, (0x8B9697, 0xC2CCCB), 0x533C36, 0x983F39,
          (0x3D536A, 0x708BA1), 0xCFDBE4, 1.07),
    Style("terraces", 0xE9D7AF, (0x899386, 0xBAC1A6), 0x914939, 0xC35C42,
          (0x325D57, 0x6B9780), 0xECB1A4, 0.78),
    Style("lake", 0xC49460, (0xAA795D, 0xC9946F), 0x795333, 0x385F65,
          (0x9E7139, 0xDBB66B), 0xF1AB57, 0.82),
    Style("aegean", 0xF0DFC3, (0xC5AEA0, 0xF5E8D0), 0x8A5A43, 0x3B7390,
          (0xBC623B, 0xE39A63), 0xDB6192, 0.74),
)


def rgb(color):
    return tuple(((color >> bit) & 255) / 255 for bit in (16, 8, 0))


def blend(first, second, amount):
    a, b = rgb(first), rgb(second)
    return tuple(x + (y - x) * amount for x, y in zip(a, b))


class Model:
    def __init__(self, seed):
        self.random = Random(seed)
        self.vertices = []
        self.faces = []
        self.colors = []

    def add(self, vertices, faces, color):
        start = len(self.vertices)
        self.vertices.extend(vertices)
        for face in faces:
            self.faces.append(tuple(start + index for index in face))
            self.colors.append(rgb(color) if isinstance(color, int) else color)

    def rings(self, rings, color, caps=True):
        sides = len(rings[0])
        faces = []
        for row in range(len(rings) - 1):
            for side in range(sides):
                a = row * sides + side
                b = row * sides + (side + 1) % sides
                faces.append((a, b, b + sides, a + sides))
        if caps:
            end = (len(rings) - 1) * sides
            faces.extend((tuple(reversed(range(sides))), tuple(range(end, end + sides))))
        self.add([vertex for ring in rings for vertex in ring], faces, color)

    def box(self, center, size, color, bevel=0, angle=0):
        x, y, z = (dimension * 0.5 for dimension in size)
        radius = min(bevel, x * 0.35, y * 0.35, z * 0.45)
        levels = (
            ((-z, radius), (-z + radius, 0), (z - radius, 0), (z, radius))
            if radius else ((-z, 0), (z, 0))
        )
        c, s = cos(angle), sin(angle)
        rings = []
        for height, inset in levels:
            w, d = x - inset, y - inset
            outline = ((-w, -d), (w, -d), (w, d), (-w, d))
            rings.append([(center[0] + u * c - v * s, center[1] + u * s + v * c,
                           center[2] + height) for u, v in outline])
        self.rings(rings, color)

    def tube(self, start, end, radii, color, sides=8):
        a, b = Vector(start), Vector(end)
        axis = (b - a).normalized()
        tangent = axis.cross(Vector((0, 0, 1) if abs(axis.z) < 0.9 else (0, 1, 0))).normalized()
        bitangent = axis.cross(tangent)
        rings = [[tuple(center + radius *
                        (cos(tau * index / sides) * tangent + sin(tau * index / sides) * bitangent))
                  for index in range(sides)] for center, radius in zip((a, b), radii)]
        self.rings(rings, color)

    def build(self, scene, material):
        surface = bpy.data.meshes.new(scene.name)
        surface.from_pydata(self.vertices, [], self.faces)
        surface.update()
        colors = surface.color_attributes.new(name="Pigment", type="BYTE_COLOR", domain="CORNER")
        for face, color in zip(surface.polygons, self.colors):
            for index in face.loop_indices:
                z = surface.vertices[surface.loops[index].vertex_index].co.z
                # Leave a little ambient shading where walls and trim meet the ground.
                bounce = 0.84 + 0.16 * min(1.0, max(0.0, z / 1.12))
                colors.data[index].color_srgb = (*(min(1., c * bounce) for c in color), 1.)
        surface.materials.append(material)
        scene.collection.objects.link(bpy.data.objects.new(scene.name, surface))


def facade(model, depth, turn=0):
    c, s = cos(turn), sin(turn)

    def piece(x, z, w, h, color, relief=0.04, thickness=0.065, bevel=0):
        y = -depth - relief
        model.box((x * c - y * s, x * s + y * c, z), (w, thickness, h), color, bevel, turn)

    return piece


def foundation(model, style, width, depth):
    model.box((0, 0, 0.115), (width + 0.10, depth + 0.10, 0.23), style.stone[0], 0.029)
    for turn in (0, pi / 2, pi, -pi / 2):
        w, d = (width, depth) if abs(cos(turn)) > 0.5 else (depth, width)
        piece = facade(model, d / 2, turn)
        for row in range(2):
            for index in range(7):
                left = max(-w / 2 - 0.02, -w / 2 + (index - 0.5 * row) * w / 6)
                right = min(w / 2 + 0.02, -w / 2 + (index + 1 - 0.5 * row) * w / 6)
                if right - left > 0.07:
                    tone = blend(*style.stone, model.random.uniform(0.16, 0.78))
                    piece((left + right) * 0.5, 0.062 + row * 0.112, right - left - 0.017,
                          0.096, tone, relief=0.045)


def walls(model, style, width, depth, eave):
    model.box((0, 0, (eave + 0.20) / 2), (width, depth, eave - 0.20), style.wall, 0.025)
    for turn in (0, pi / 2, pi, -pi / 2):
        w, d = (width, depth) if abs(cos(turn)) > 0.5 else (depth, width)
        piece = facade(model, d / 2, turn)
        if style.file == "fjord":
            for index in range(9):
                z = 0.30 + (eave - 0.32) * index / 9
                color = blend(style.wall, style.timber, model.random.uniform(0.05, 0.34))
                piece(0, z, w - 0.04, (eave - 0.32) / 9 - 0.012, color, relief=0.018)
        if style.file in ("lake", "aegean"):
            for side in (-1, 1):
                for row in range(5):
                    stone_width = 0.29 if row % 2 else 0.40
                    tone = blend(*style.stone, model.random.uniform(0.24, 0.92))
                    piece(side * (w * 0.49 - stone_width * 0.48), 0.37 + row * (eave - 0.30) / 5,
                          stone_width, 0.24, tone, relief=0.022, bevel=0.021)
        else:
            for side in (-1, 1):
                piece(side * w * 0.46, (eave + 0.22) / 2, 0.14, eave - 0.22, style.timber,
                      bevel=0.019)
            piece(0, 0.49, w - 0.06, 0.09, style.timber)
        piece(0, eave - 0.058, w + 0.09, 0.11, style.timber, relief=0.039)


def tile(model, x0, x1, y0, y1, height, crown, color):
    n = 4 if crown > 0.046 else 2
    vertices = []
    for x in (x0, x1):
        for index in range(n + 1):
            t = index / n
            y = y0 + t * (y1 - y0)
            vertices.append((x, y, height(x) + 0.042 + crown * sin(pi * t)))
    stride = n + 1
    faces = [(i, i + stride, i + stride + 1, i + 1) for i in range(n)]
    top_edge_z = height(x0) + 0.018
    vertices.extend(((x0, y0, top_edge_z), (x0, y1, top_edge_z)))
    faces.append(tuple(range(stride)) + (len(vertices) - 1, len(vertices) - 2))
    model.add(vertices, faces, color)


def gable(model, style, depth, eave, store):
    span = 1.45
    end = depth / 2 + 0.23
    pitch = style.pitch * (0.82 if store else 1)
    curved = style.file == "terraces"

    def height(x):
        t = abs(x / span)
        return eave + pitch * (1 - t) - (0.17 * sin(pi * t) if curved else 0)

    for turn in (-1, 1):
        face_y = turn * depth / 2
        peak = [(-span + 0.18, face_y, eave - 0.055), (span - 0.18, face_y, eave - 0.055),
                (0, face_y, height(0) - 0.075)]
        model.add(peak, ((0, 1, 2) if turn < 0 else (2, 1, 0),), style.wall)
    for half in (-1, 1):
        for row in range(5):
            first, last = sorted((half * span * row / 5, half * span * (row + 1) / 5))
            for y in (-end, end):
                model.tube((first, y, height(first) - 0.015), (last, y, height(last) - 0.015),
                           (0.064, 0.064), style.timber, sides=6)
            model.add([(first, -end, height(first)), (last, -end, height(last)),
                       (last, end, height(last)), (first, end, height(first))],
                      ((0, 1, 2, 3),), style.roof[0])
            count = 9
            step = 2 * end / count
            for index in range(count + 1):
                y0 = max(-end, -end + (index - 0.46 * (row % 2)) * step)
                y1 = min(end, y0 + step - 0.012) if index else min(end, -end + (1 - 0.46 * (row % 2)) * step)
                if y1 - y0 < 0.075:
                    continue
                tone = blend(*style.roof, model.random.uniform(0.23, 0.89))
                crown = 0.061 if style.file in ("aegean", "terraces") else 0.028
                tile(model, first + 0.007, last - 0.007, y0, y1, height, crown, tone)
    for index in range(9):
        y = -end + index * (end * 2 / 8)
        model.tube((0, y - 0.145, height(0) + 0.048), (0, y + 0.145, height(0) + 0.048),
                   (0.077, 0.077), style.roof[1], sides=7)
    if style.file == "fjord":
        for side in (-1, 1):
            for index in range(4):
                x = side * (0.38 + index * 0.10)
                y = -0.78 + index * 0.49 + 0.12 * side
                model.box((x, y, height(x) + 0.05), (0.23, 0.39, 0.09), 0xDEE4DF, 0.028)
    if not store and style.file != "terraces":
        x = span * 0.33
        z = height(x)
        model.box((x, 0.32, z + 0.22), (0.28, 0.36, 0.59), style.stone[0], 0.024)
        for row in range(3):
            model.box((x, 0.32, z + row * 0.16), (0.29, 0.365, 0.03), style.stone[1])
        model.box((x, 0.32, z + 0.52), (0.37, 0.45, 0.12), style.stone[1], 0.023)
        model.box((x, 0.32, z + 0.583), (0.19, 0.23, 0.011), 0x4B4239)


def thatch(model, style):
    sides = 24
    levels = ((1.53, 1.42), (1.60, 1.42), (1.76, 1.23),
              (1.98, 0.89), (2.21, 0.54), (2.42, 0.18), (2.49, 0.07))
    rings = [
        [(cos(tau * i / sides) * r * (0.97 + 0.055 * sin(i * 1.8)),
          sin(tau * i / sides) * r * (0.97 + 0.055 * sin(i * 1.8)),
          z + 0.018 * sin(i * 2.4 + row * 0.46)) for i in range(sides)]
        for row, (z, r) in enumerate(levels)
    ]
    for lower, upper in zip(rings, rings[1:]):
        for i in range(sides):
            j = (i + 1) % sides
            tone = blend(*style.roof, model.random.uniform(0.12, 0.88))
            model.add((lower[i], lower[j], upper[j], upper[i]), ((0, 1, 2, 3),), tone)
    model.tube((0, 0, 2.40), (0, 0, 2.56), (0.13, 0.06), style.roof[0])


def window(piece, style, x, z, shutters=True):
    piece(x, z, 0.46, 0.46, style.stone[1], thickness=0.11, bevel=0.017)
    piece(x, z + 0.015, 0.325, 0.33, 0x483A33, 0.105, 0.045)
    for side in (-1, 1):
        piece(x + side * 0.080, z + 0.036, 0.132, 0.264, 0xE5B77A, 0.131, 0.035)
        if shutters:
            piece(x + side * 0.254, z + 0.02, 0.143, 0.38, style.trim, 0.114, 0.055, 0.011)
            for slat in range(4):
                piece(x + side * 0.254, z - 0.10 + slat * 0.074, 0.126, 0.017,
                      blend(style.trim, style.stone[1], 0.24), 0.147, 0.025)
    piece(x, z + 0.02, 0.35, 0.030, style.timber, 0.154, 0.025)
    piece(x, z - 0.207, 0.57, 0.071, style.stone[1], 0.112, 0.218, 0.014)


def pot(model, x, y, style, z=0):
    model.tube((x, y, z + 0.025), (x, y, z + 0.29), (0.102, 0.16), 0xB97351)
    model.tube((x, y, z + 0.262), (x, y, z + 0.309), (0.174, 0.169), 0xCD9266)
    model.tube((x, y, z + 0.308), (x, y, z + 0.314), (0.135, 0.135), 0x735640)
    for i in range(7):
        angle = i * 2.4
        dx, dy = cos(angle), sin(angle)
        height = model.random.uniform(0.14, 0.32)
        tip = (x + dx * height * 0.66, y + dy * height * 0.66, z + 0.32 + height)
        model.tube((x, y, z + 0.32), tip, (0.022, 0.011), 0x648053, sides=5)
        if i % 2 == 0:
            model.tube((tip[0], tip[1], tip[2] - 0.018), (tip[0], tip[1], tip[2] + 0.047),
                       (0.037, 0.057), style.bloom, sides=6)


def climbing_vine(model, style):
    points = ((0.80, -1.35, 0.31), (0.99, -1.31, 0.58), (1.02, -1.17, 0.89),
              (1.18, -1.17, 1.15), (1.20, -1.21, 1.45), (1.06, -1.18, 1.65))
    for index, (a, b) in enumerate(zip(points, points[1:])):
        radius = 0.042 - index * 0.005
        model.tube(a, b, (radius, radius * 0.83), style.timber, sides=5)
        for side in (-1, 1):
            x, y, z = b
            tip = (x + side * 0.138, y - 0.096, z + 0.079)
            model.tube(b, tip, (0.018, 0.009), 0x596D47, sides=5)
            color = blend(style.bloom, 0xF1ADBA, model.random.uniform(0.08, 0.65))
            model.tube((tip[0], tip[1], tip[2] - 0.028), tip,
                       (0.034, 0.112), 0x668653, sides=6)
            model.tube(tip, (tip[0], tip[1], tip[2] + 0.069), (0.093, 0.052), color, sides=6)


def lantern(model, x, y, z, cord):
    model.tube((x, y, z + 0.22), (x, y, z + 0.22 + cord),
               (0.018, 0.018), 0x8D5A3D, sides=5)
    model.tube((x, y, z), (x, y, z + 0.12), (0.071, 0.105), 0xDB7B4E)
    model.tube((x, y, z + 0.12), (x, y, z + 0.24), (0.105, 0.071), 0xF2A85C)
    for height in (0.01, 0.23):
        model.tube((x, y, z + height - 0.012), (x, y, z + height + 0.012),
                   (0.075, 0.075), 0xA6523D, sides=7)


def cottage(style, seed):
    model = Model(seed)
    width, depth, eave = 2.36, 2.16, 1.86
    if style.file == "lake":
        model.tube((0, 0, 0), (0, 0, 0.22), (1.15, 1.17), style.stone[0], sides=16)
        model.tube((0, 0, 0.18), (0, 0, 1.63), (1.12, 1.08), style.wall, sides=16)
        for z in (0.33, 0.47, 1.41, 1.49):
            model.tube((0, 0, z), (0, 0, z + 0.033), (1.126, 1.126),
                       0xD9B67F if z > 1 else 0xB78055, sides=16)
        thatch(model, style)
    else:
        foundation(model, style, width, depth)
        walls(model, style, width, depth, eave)
        gable(model, style, depth, eave, store=False)
    front = facade(model, depth / 2)
    front(0, 0.85, 0.75, 1.17, style.stone[1], relief=0.035, thickness=0.16, bevel=0.023)
    front(0, 0.84, 0.554, 1.04, 0x453B32, relief=0.124)
    for plank in range(4):
        color = blend(style.trim, style.stone[1], 0.08 + plank * 0.043)
        front(-0.198 + plank * 0.13, 0.84, 0.123, 1.01, color, relief=0.153)
    for z in (0.52, 1.15):
        front(0, z, 0.53, 0.076, style.trim, 0.198, 0.026)
    front(0.19, 0.86, 0.076, 0.060, 0xD9B573, 0.202, 0.064, 0.008)
    if style.file == "lake":
        for angle in (-0.76, 0.77, 1.63):
            window(facade(model, 1.064, angle), style, 0, 1.17, shutters=False)
    else:
        for x in (-0.78, 0.78):
            window(front, style, x, 1.17)
        window(facade(model, width / 2, pi / 2), style, -0.24, 1.21)
    for level in range(2):
        model.box((0, -depth / 2 - 0.31 + 0.13 * level, 0.062 + 0.122 * level),
                  (1.04 - level * 0.10, 0.38, 0.124), style.stone[1], 0.024)
    pot(model, 0.80, -depth / 2 - 0.27, style)
    if style.file in ("valley", "fjord"):
        for side in (-1, 1):
            model.tube((side * 0.87, -depth / 2 - 0.024, 1.96),
                       (side * 0.21, -depth / 2 - 0.024, 2.48),
                       (0.041, 0.041), style.timber, sides=5)
        front(0, 2.16, 0.168, 0.23, style.trim, relief=0.031, bevel=0.014)
    elif style.file == "terraces":
        lantern(model, 1.05, -depth / 2 - 0.188, 1.44, 0.24)
    elif style.file == "aegean":
        climbing_vine(model, style)
    return model


def warehouse(style, seed):
    model = Model(seed)
    width, depth, eave = 2.36, 2.16, 1.63
    foundation(model, style, width, depth)
    walls(model, style, width, depth, eave)
    gable(model, style, depth, eave, store=True)
    front = facade(model, depth / 2)
    front(0, 0.86, 1.67, 1.23, 0x46362D)
    for side in (-1, 1):
        front(side * 0.85, 0.86, 0.13, 1.32, style.timber, 0.105, 0.16, 0.013)
        front(side * 0.65, 1.15, 0.31, 0.07, style.trim, 0.15)
    front(0, 1.49, 1.87, 0.153, style.timber, 0.105, 0.17, 0.016)
    front(0, 1.56, 0.47, 0.12, style.trim, 0.199, 0.054, 0.011)
    model.box((0, -depth / 2 - 0.23, 0.13), (1.93, 0.63, 0.26), style.timber, 0.026)
    for index in range(4):
        for row in range(3):
            if index + row < 5:
                x = -0.70 + index * 0.22 + row * 0.043
                z = 0.36 + row * 0.18
                model.tube((x, -1.09, z), (x, -1.53 - 0.02 * row, z),
                           (0.108, 0.105), style.timber, sides=7)
                model.tube((x, -1.53 - 0.02 * row, z), (x, -1.546 - 0.02 * row, z),
                           (0.083, 0.083), 0xCA9B69, sides=7)
    for x, y, z, size in ((0.69, -1.38, 0.43, 0.35), (0.55, -1.34, 0.71, 0.28)):
        model.box((x, y, z), (size, size, size), 0xBC9363, 0.017, angle=0.06)
        model.box((x, y - size / 2, z - size * 0.28), (size, 0.036, 0.065), style.timber)
        model.box((x, y - size / 2, z + size * 0.28), (size, 0.036, 0.065), style.timber)
    if style.file in ("lake", "aegean"):
        flag = 0xEBC174 if style.file == "aegean" else 0xCC9260
        for stripe in range(7):
            x0, x1 = -0.91 + stripe * 0.26, -0.65 + stripe * 0.26
            tone = blend(flag, 0xF3DDAA, 0.23 * (stripe % 2))
            model.add([(x0, -1.59, 1.41), (x1, -1.59, 1.41),
                       (x1, -1.12, 1.53), (x0, -1.12, 1.53)],
                      ((0, 1, 2, 3), (3, 2, 1, 0)), tone)
            model.add([(x0, -1.59, 1.41), (x0, -1.60, 1.34),
                       (x1, -1.60, 1.34), (x1, -1.59, 1.41)],
                      ((0, 1, 2, 3),), tone)
    elif style.file == "terraces":
        lantern(model, 0.88, -1.36, 1.24, 0.05)
    return model


def export(style, index):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    house = bpy.context.scene
    house.name = "House"
    store = bpy.data.scenes.new("Store")
    material = bpy.data.materials.new("Bemalte Oberflächen")
    material.use_nodes = True
    nodes = material.node_tree.nodes
    shader = nodes.get("Principled BSDF")
    shader.inputs["Roughness"].default_value = 0.88
    pigments = nodes.new("ShaderNodeVertexColor")
    pigments.layer_name = "Pigment"
    material.node_tree.links.new(pigments.outputs["Color"], shader.inputs["Base Color"])
    cottage(style, index * 37).build(house, material)
    warehouse(style, index * 37 + 17).build(store, material)
    bpy.ops.export_scene.gltf(
        filepath=str(OUTPUT / f"{style.file}.glb"), export_format="GLB",
        use_active_scene=False, export_yup=True, export_animations=False,
    )


if __name__ == "__main__":
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for index, style in enumerate(STYLES):
        export(style, index)
