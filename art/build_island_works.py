"""Build the island landmarks and their portraits with blender -b -P art/build_island_works.py."""

import bpy
import math
from mathutils import Vector
from pathlib import Path
from math import pi, sin, cos
root = Path(__file__).resolve().parents[1] / 'assets'
colors = {'slate': '4B575D', 'stone': 'A99782', 'pale': 'E1CDA7', 'ivory': 'F1E0B6', 'wood': '885734', 'oak': 'AF794A', 'dark': '594337', 'roof': 'A44B38', 'brick': 'CA7850', 'jade': '416E69', 'teal': '76B7A6', 'navy': '405975', 'blue': '5E8CA8', 'brass': 'D6A553', 'gold': 'FEE1A0', 'moss': '7E8851', 'water': '60AFBB'}

def linear(x):
    return ((x + 0.055) / 1.055) ** 2.4 if x > 0.04045 else x / 12.92

def mat(name):
    if name in bpy.data.materials:
        return bpy.data.materials[name]
    material = bpy.data.materials.new(name)
    rgb = [linear(int(colors[name][i:i + 2], 16) / 255) for i in (0, 2, 4)] + [1]
    material.diffuse_color = rgb
    material.use_nodes = True
    shader = material.node_tree.nodes.get('Principled BSDF')
    shader.inputs['Base Color'].default_value = rgb
    shader.inputs['Roughness'].default_value = 0.7
    if name == 'brass':
        shader.inputs['Metallic'].default_value = 0.36
    return material

def finish(name, color):
    obj = bpy.context.object
    obj.name = name
    obj.data.materials.append(mat(color))
    return obj

def cube(name, position, dimensions, color, bevel=0.035, rot=0):
    bpy.ops.mesh.primitive_cube_add(size=1, location=position)
    obj = finish(name, color)
    obj.dimensions = dimensions
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    obj.rotation_euler.z = rot
    if bevel:
        bevel_modifier = obj.modifiers.new('Hewn edges', 'BEVEL')
        bevel_modifier.width = min(bevel, min(dimensions) * 0.23)
        bevel_modifier.segments = 1
        bpy.ops.object.modifier_apply(modifier=bevel_modifier.name)
    return obj

def cone(name, position, lower_radius, upper_radius, h, color, sides=12, angle=0):
    bpy.ops.mesh.primitive_cone_add(vertices=sides, radius1=lower_radius, radius2=upper_radius, depth=h, location=position, rotation=(0, 0, angle))
    return finish(name, color)

def beam(name, a, b, width, color, depth=None):
    a, b = (Vector(a), Vector(b))
    obj = cube(name, (a + b) / 2, (width, depth or width, (b - a).length), color, 0.018)
    obj.rotation_mode = 'QUATERNION'
    obj.rotation_quaternion = (b - a).to_track_quat('Z', 'Y')
    return obj

def barrel(x, y, z, r=0.18):
    cone('Cask', (x, y, z + 0.19), r, r, 0.38, 'oak')
    for h in (0.07, 0.3):
        cone('Hoop', (x, y, z + h), r + 0.008, r + 0.008, 0.035, 'slate')

def wall(x, y, width, depth, h, color='pale'):
    cube('Coursed masonry', (x, y, 0.17 + h * 0.5), (width, depth, h), color, 0.055)
    for height in (0.18, h + 0.16):
        cube('Cornice', (x, y, height), (width + 0.1, depth + 0.09, 0.1), 'stone')

def gable(x, y, z, width, depth, h, a, b):
    for sign in (-1, 1):
        obj = cube('Roof slope', (x + sign * width / 4, y, z + h / 2), ((width * width / 4 + h * h) ** 0.5, depth, 0.13), a, 0.03)
        obj.rotation_euler.y = sign * math.atan2(h, width / 2)
        for k in range(3):
            offset = (k + 0.35) / 3 * width / 2
            obj = cube('Tile course', (x + sign * offset, y, z + h * (1 - offset / (width / 2)) + 0.065), (0.06, depth + 0.04, 0.06), b, 0.009)
    cube('Ridge cap', (x, y, z + h + 0.06), (0.16, depth + 0.06, 0.12), b)

def base():
    cube('Lower foundation', (0, 0, 0.055), (2.45, 1.93, 0.11), 'slate')
    cube('Stone apron', (0, 0, 0.13), (2.3, 1.8, 0.14), 'stone')

def wheel(x, y, z, r, kind='wood'):
    """The named axle is also the rotation pivot used by the game."""
    hub = bpy.data.objects.new('WerkRad', None)
    bpy.context.collection.objects.link(hub)
    hub.location = (x, y, z)
    parts = []
    for side in [-0.11, 0.11]:
        bpy.ops.mesh.primitive_torus_add(major_segments=16, minor_segments=4, location=(x + side, y, z), rotation=(0, pi / 2, 0), major_radius=r, minor_radius=0.044)
        parts.append(finish('Bound rim', 'brass' if kind == 'brass' else 'dark'))
    for k in range(10):
        a = 2 * pi * k / 10
        v = Vector((0, cos(a), sin(a)))
        p = Vector((x, y, z))
        parts.append(beam('Wheel spoke', p, p + v * r, 0.042, 'oak'))
        obj = cube('Paddle', p + v * (r - 0.02), (0.36, 0.135, 0.08), 'oak')
        obj.rotation_euler.x = a
        parts.append(obj)
    for obj in parts:
        location = obj.location.copy()
        obj.parent = hub
        obj.location = location - Vector((x, y, z))

def plant(x, y):
    for k in range(5):
        a = k * 2.39
        beam('Reed', (x, y, 0.18), (x + 0.12 * cos(a), y + 0.12 * sin(a), 0.6 + 0.07 * (k % 3)), 0.044, 'moss')

def create(theme):
    base()
    if theme == 'valley':
        wall(-0.26, 0.14, 1.08, 1.05, 1.02)
        gable(-0.26, 0.14, 1.2, 1.45, 1.4, 0.52, 'roof', 'brick')
        for x in (-0.72, 0.18):
            cube('Oak upright', (x, -0.409, 0.7), (0.1, 0.08, 0.98), 'dark')
        beam('Diagonal brace', (-0.67, -0.46, 0.28), (0.16, -0.46, 1.1), 0.08, 'wood')
        cube('Millrace', (0.76, 0.02, 0.205), (0.56, 1.63, 0.055), 'water')
        wheel(0.69, 0.02, 0.86, 0.64)
        barrel(-0.86, -0.58, 0.2, 0.14)
    if theme == 'fjord':
        for k in range(4):
            cone('Beacon course', (0, 0.04, 0.36 + k * 0.36), 0.7 - k * 0.09, 0.64 - k * 0.09, 0.37, 'slate' if k % 2 == 0 else 'stone', 8, pi / 8)
        for x in (-0.36, 0.36):
            for y in (-0.31, 0.39):
                beam('Fire basket post', (x, y, 1.4), (x * 1.22, y * 1.22, 2.2), 0.085, 'dark')
        cone('Hearth', (0, 0.04, 1.56), 0.42, 0.54, 0.17, 'brass', 8)
        cone('Amber beacon', (0, 0.04, 1.86), 0.3, 0.04, 0.56, 'gold', 7)
        for k in range(3):
            cube('Landing stair', (0, -0.6 - k * 0.11, 0.26 - 0.025 * k), (0.79, 0.23, 0.13), 'stone')
        for side in (-1, 1):
            beam('Carved horn', (side * 0.45, 0.12, 2.1), (side * 0.67, 0.12, 2.38), 0.1, 'oak')
    if theme == 'terraces':
        for x in (-0.66, 0.66):
            for y in (-0.42, 0.42):
                cube('Vermilion post', (x, y, 0.84), (0.14, 0.14, 1.3), 'roof')
                cone('Post foot', (x, y, 0.23), 0.16, 0.16, 0.11, 'stone', 8)
        for side in (1, -1):
            beam('Bracket', (-0.82, side * 0.45, 1.35), (0.82, side * 0.45, 1.35), 0.14, 'wood')
        for z, r, rr in [(1.55, 1.12, 0.69), (1.79, 0.74, 0.28)]:
            cone('Jade pavilion roof', (0, 0, z), r, rr, 0.23, 'jade', 4, pi / 4)
            cone('Glazed eaves', (0, 0, z - 0.105), r + 0.025, r, 0.052, 'teal', 4, pi / 4)
        cone('Roof finial', (0, 0, 2.02), 0.09, 0.0, 0.3, 'brass')
        cube('Rice channel', (0, 0, 0.3), (1.8, 0.32, 0.1), 'water')
        wheel(0.42, 0, 0.78, 0.45, 'brass')
    if theme == 'lake':
        for k in range(3):
            cone('Well course', (-0.4, 0.04, 0.29 + k * 0.15), 0.52, 0.54, 0.16, 'pale' if k % 2 == 0 else 'stone', 12)
        cone('Well water', (-0.4, 0.04, 0.6), 0.44, 0.44, 0.05, 'water', 12)
        for y in (-0.22, 0.24):
            beam('Shaduf fork', (0.29, y, 0.22), (0.27, y * 0.6, 1.69), 0.14, 'dark')
        beam('Shaduf lever', (-0.73, 0, 1.23), (0.96, 0, 2.05), 0.11, 'oak')
        beam('Bucket line', (-0.65, 0.01, 1.27), (-0.65, 0.01, 0.72), 0.029, 'wood')
        cone('Copper pail', (-0.65, 0.01, 0.74), 0.115, 0.15, 0.21, 'brass')
        cone('Counterweight', (0.9, 0, 1.87), 0.19, 0.13, 0.35, 'stone', 7)
        plant(0.69, 0.6)
        plant(0.88, 0.48)
    if theme == 'aegean':
        for k in range(3):
            cone('Lighthouse ashlar', (0, 0.05, 0.47 + k * 0.46), 0.64 - k * 0.1, 0.56 - k * 0.1, 0.46, 'ivory' if k != 1 else 'pale', 8, pi / 8)
        cone('Blue gallery', (0, 0.05, 1.6), 0.48, 0.54, 0.12, 'navy', 8)
        for k in range(6):
            a = k * pi / 3
            cone('Lantern column', (0.37 * cos(a), 0.05 + 0.37 * sin(a), 1.87), 0.048, 0.044, 0.47, 'ivory', 8)
        cone('Lamp', (0, 0.05, 1.84), 0.2, 0.11, 0.31, 'gold')
        cone('Copper cupola', (0, 0.05, 2.17), 0.56, 0.14, 0.25, 'jade', 8)
        cone('Finial', (0, 0.05, 2.38), 0.06, 0, 0.22, 'brass')
        barrel(-0.82, -0.36, 0.2)
        cube('Quay ledge', (0.78, -0.25, 0.26), (0.42, 0.75, 0.16), 'pale')

def aim(obj, target):
    obj.rotation_euler = (Vector(target) - obj.location).to_track_quat('-Z', 'Y').to_euler()
for directory in ('models/works', 'ui/works'):
    (root / directory).mkdir(parents=True, exist_ok=True)

# Batch static geometry by material, retaining the wheel as a separate rotating group.
for name in ('valley', 'fjord', 'terraces', 'lake', 'aegean'):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    create(name)
    groups = {}
    for obj in list(bpy.context.scene.objects):
        if obj.type == 'MESH':
            groups.setdefault((obj.parent, obj.data.materials[0].name), []).append(obj)
    for (parent, material), objects in groups.items():
        bpy.ops.object.select_all(action='DESELECT')
        for obj in objects:
            obj.select_set(True)
        bpy.context.view_layer.objects.active = objects[0]
        if len(objects) > 1:
            bpy.ops.object.join()
        objects[0].name = name + '-' + material
    bpy.ops.export_scene.gltf(filepath=str(root / 'models/works' / f'{name}.glb'), export_format='GLB', export_animations=False)
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.use_denoising = False
    scene.cycles.samples = 20
    scene.render.film_transparent = True
    scene.world = bpy.data.worlds.new('Ambient')
    scene.world.use_nodes = True
    scene.world.node_tree.nodes['Background'].inputs[0].default_value = (0.69, 0.76, 0.85, 1)
    scene.world.node_tree.nodes['Background'].inputs[1].default_value = 0.8
    bpy.ops.object.light_add(type='AREA', location=(-3, -4, 6))
    obj = bpy.context.object
    obj.data.energy = 430
    obj.data.shape = 'DISK'
    obj.data.size = 4
    aim(obj, (0, 0, 0.7))
    bpy.ops.object.camera_add(location=(4.4, -7, 4.5))
    obj = bpy.context.object
    aim(obj, (0, 0, 1.0))
    obj.data.type = 'ORTHO'
    obj.data.ortho_scale = 3.45
    scene.camera = obj
    scene.render.resolution_x = 256
    scene.render.resolution_y = 228
    scene.render.resolution_percentage = 100
    scene.render.image_settings.color_mode = 'RGBA'
    scene.render.filepath = str(root / 'ui/works' / f'{name}.png')
    bpy.ops.render.render(write_still=True)
