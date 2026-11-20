"""Build the original Hippogrove companion. Run with blender -b -P art/build_scene.py."""
import math
from pathlib import Path

import bpy

ROOT = Path(__file__).resolve().parents[1]
bpy.ops.wm.read_factory_settings(use_empty=True)


def material(name, color):
    result = bpy.data.materials.new(name)
    result.diffuse_color = (*color, 1)
    result.use_nodes = True
    principled = result.node_tree.nodes.get("Principled BSDF")
    principled.inputs["Base Color"].default_value = result.diffuse_color
    principled.inputs["Roughness"].default_value = 0.82
    return result


hide = material("Lavendel", (0.41, 0.33, 0.52))
cheek = material("Rosa Schnauze", (0.70, 0.44, 0.51))
belly = material("Bauch", (0.51, 0.43, 0.58))
ear = material("Ohr innen", (0.66, 0.38, 0.46))
eye = material("Auge", (0.031, 0.021, 0.037))
shine = material("Lichtreflex", (0.89, 0.88, 0.74))
snoot = material("Nüstern", (0.32, 0.17, 0.24))
scarf = material("Honigfarbenes Halstuch", (0.84, 0.44, 0.095))
crease = material("Halstuchfalte", (0.64, 0.29, 0.052))
linen = material("Ungebleichtes Leinen", (0.78, 0.67, 0.48))
fold = material("Leinen im Schatten", (0.65, 0.51, 0.33))
hem = material("Gewebte Borte", (0.37, 0.26, 0.13))
bronze = material("Bronzene Spange", (0.78, 0.50, 0.16))


def rounded(name, position, scale, surface, segments=16, rings=10):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segments, ring_count=rings, location=position)
    obj = bpy.context.object
    obj.name = name
    obj.scale = scale
    obj.data.materials.append(surface)
    return obj


def tunic():
    rings = ((0.53, 0.32, 0.265), (0.56, 0.348, 0.29), (0.62, 0.367, 0.316),
             (0.79, 0.420, 0.332), (0.91, 0.405, 0.318), (0.985, 0.355, 0.277))
    sides = 22
    positions = []
    faces = []
    for row, (height, width, depth) in enumerate(rings):
        for side in range(sides):
            angle = math.tau * side / sides
            ripple = 1 + 0.024 * math.sin(angle * 7 + row * 0.65)
            positions.append((width * math.cos(angle) * ripple,
                              0.013 + depth * math.sin(angle) * ripple,
                              height + 0.021 * math.cos(angle)))
    for row in range(len(rings) - 1):
        for side in range(sides):
            a, b = row * sides + side, row * sides + (side + 1) % sides
            faces.append((a, b, b + sides, a + sides))
    mesh = bpy.data.meshes.new("Gefaltetes Leinen")
    mesh.from_pydata(positions, [], faces)
    obj = bpy.data.objects.new(mesh.name, mesh)
    bpy.context.collection.objects.link(obj)
    for surface in (linen, fold, hem):
        mesh.materials.append(surface)
    for index, face in enumerate(mesh.polygons):
        row, side = divmod(index, sides)
        face.material_index = 2 if row == 0 else int((side + row) % 7 in (2, 3))


rounded("Runder Körper", (0, 0.03, 0.79), (0.41, 0.31, 0.44), hide)
tunic()
rounded("Kopf", (0, -0.07, 1.25), (0.43, 0.35, 0.35), hide)
rounded("Schnauze", (0, -0.357, 1.12), (0.40, 0.277, 0.218), cheek)
rounded("Unterlippe", (0, -0.388, 0.965), (0.304, 0.18, 0.097), belly)
rounded("Halstuch", (0, -0.008, 0.992), (0.379, 0.32, 0.089), scarf)
rounded("Halstuchknoten", (-0.23, -0.266, 0.969), (0.11, 0.081, 0.087), scarf, 10, 6)
tail = rounded("Halstuchzipfel", (-0.300, -0.265, 0.83), (0.073, 0.042, 0.155), crease, 10, 6)
tail.rotation_euler.y = 0.48
rounded("Bronzene Fibel", (-0.225, -0.338, 0.969), (0.043, 0.021, 0.046), bronze, 12, 6)
for side in (-1, 1):
    rounded(f"Ohr {side}", (side * 0.286, -0.031, 1.535), (0.11, 0.07, 0.123), hide, 12, 7)
    rounded(f"Ohrmuschel {side}", (side * 0.291, -0.079, 1.544), (0.064, 0.029, 0.078), ear, 10, 6)
    rounded(f"Auge {side}", (side * 0.205, -0.331, 1.359), (0.060, 0.044, 0.072), eye, 12, 8)
    rounded(f"Augenlicht {side}", (side * 0.205 - 0.017, -0.371, 1.385), (0.014, 0.012, 0.016), shine, 8, 6)
    rounded(f"Nüster {side}", (side * 0.19, -0.575, 1.203), (0.047, 0.024, 0.032), snoot, 10, 6)
    rounded(f"Bäckchen {side}", (side * 0.31, -0.489, 1.075), (0.068, 0.027, 0.036), ear, 10, 6)
    brow = rounded(f"Augenbraue {side}", (side * 0.205, -0.304, 1.463),
                   (0.070, 0.026, 0.016), snoot, 10, 6)
    brow.rotation_euler.y = side * 0.19
    rounded(f"Milchzahn {side}", (side * 0.098, -0.587, 1.01),
            (0.031, 0.023, 0.034), shine, 10, 6)
rounded("Lächeln", (0, -0.596, 1.052), (0.116, 0.016, 0.014), snoot, 12, 6)

model_dir = ROOT / "assets" / "models"
model_dir.mkdir(parents=True, exist_ok=True)
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT / "art" / "Hippogrove.blend"))
bpy.ops.export_scene.gltf(filepath=str(model_dir / "hippo.glb"), export_format="GLB", export_yup=True)
