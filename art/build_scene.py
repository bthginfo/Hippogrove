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


def rounded(name, position, scale, surface, segments=16, rings=10):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segments, ring_count=rings, location=position)
    obj = bpy.context.object
    obj.name = name
    obj.scale = scale
    obj.data.materials.append(surface)
    return obj


rounded("Runder Körper", (0, 0.03, 0.79), (0.41, 0.31, 0.44), hide)
rounded("Heller Bauch", (0, -0.135, 0.77), (0.32, 0.21, 0.32), belly)
rounded("Kopf", (0, -0.07, 1.25), (0.43, 0.35, 0.35), hide)
rounded("Schnauze", (0, -0.357, 1.12), (0.40, 0.277, 0.218), cheek)
rounded("Halstuch", (0, -0.008, 0.992), (0.376, 0.315, 0.105), scarf)
rounded("Halstuchknoten", (-0.23, -0.266, 0.969), (0.11, 0.081, 0.087), scarf, 10, 6)
tail = rounded("Halstuchzipfel", (-0.300, -0.265, 0.83), (0.073, 0.042, 0.155), crease, 10, 6)
tail.rotation_euler.y = 0.48
for side in (-1, 1):
    rounded(f"Ohr {side}", (side * 0.286, -0.031, 1.535), (0.11, 0.07, 0.123), hide, 12, 7)
    rounded(f"Ohrmuschel {side}", (side * 0.291, -0.079, 1.544), (0.064, 0.029, 0.078), ear, 10, 6)
    rounded(f"Auge {side}", (side * 0.205, -0.331, 1.359), (0.060, 0.044, 0.072), eye, 12, 8)
    rounded(f"Augenlicht {side}", (side * 0.205 - 0.017, -0.371, 1.385), (0.014, 0.012, 0.016), shine, 8, 6)
    rounded(f"Nüster {side}", (side * 0.19, -0.575, 1.203), (0.047, 0.024, 0.032), snoot, 10, 6)
    rounded(f"Bäckchen {side}", (side * 0.31, -0.489, 1.075), (0.068, 0.027, 0.036), ear, 10, 6)
rounded("Lächeln", (0, -0.596, 1.052), (0.116, 0.016, 0.014), snoot, 12, 6)

model_dir = ROOT / "assets" / "models"
model_dir.mkdir(parents=True, exist_ok=True)
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT / "art" / "Hippogrove.blend"))
bpy.ops.export_scene.gltf(filepath=str(model_dir / "hippo.glb"), export_format="GLB", export_yup=True)
