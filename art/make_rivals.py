"""Sculpt the five Hippogrove rivals. Run with blender -b -P art/make_rivals.py."""

import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[1]
MODELS = ROOT / "assets/models/rivals"


def reset():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    for data in list(bpy.data.materials):
        bpy.data.materials.remove(data)


def paint(name, color, roughness=.81, metal=0., glow=0.):
    def channel(c):
        c /= 255.
        return ((c + .055) / 1.055) ** 2.4 if c > .04045 else c / 12.92
    rgba = tuple(channel(int(color[n:n + 2], 16)) for n in (0, 2, 4)) + (1.,)
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = rgba
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metal
    if glow:
        bsdf.inputs["Emission Color"].default_value = rgba
        bsdf.inputs["Emission Strength"].default_value = glow
    mat.diffuse_color = rgba
    return mat


def space(p):
    return (p[0], -p[2], p[1])


def surface(name, vertices, faces, material, smooth=True):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([space(p) for p in vertices], [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    mesh.materials.append(material)
    for polygon in mesh.polygons:
        polygon.use_smooth = smooth
    return obj


def loft(name, points, material, sides=12):
    """A tapered volume with elliptical sections around a three-dimensional spine."""
    vertices, faces = [], []
    for i, (center, across, depth) in enumerate(points):
        prev = Vector(points[max(i - 1, 0)][0])
        after = Vector(points[min(i + 1, len(points) - 1)][0])
        direction = (after - prev).normalized()
        cross = Vector((1., 0., 0.))
        if abs(direction.dot(cross)) > .92:
            cross = Vector((0., 1., 0.))
        u = (cross - direction.dot(cross) * direction).normalized()
        v = direction.cross(u)
        for k in range(sides):
            angle = math.tau * k / sides
            vertices.append(Vector(center) + math.cos(angle) * across * u + math.sin(angle) * depth * v)
    for i in range(len(points) - 1):
        for k in range(sides):
            a, b = i * sides + k, i * sides + (k + 1) % sides
            faces.append((a, b, b + sides, a + sides))
    faces.extend((tuple(reversed(range(sides))), tuple(range(len(vertices) - sides, len(vertices)))))
    return surface(name, vertices, faces, material)


def oval(name, at, size, material, roll=0.):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=10, location=space(at))
    obj = bpy.context.object
    obj.name = name
    obj.scale = space(size)
    obj.scale.y = abs(obj.scale.y)
    obj.rotation_euler[1] = roll
    obj.data.materials.append(material)
    for polygon in obj.data.polygons:
        polygon.use_smooth = True
    return obj


def blade(name, start, end, width, thickness, material, curve=0.):
    s, e = Vector(start), Vector(end)
    bend = Vector((0., curve, 0.))
    return loft(name, [
        (s, .01, .01),
        (s.lerp(e, .28) + .7 * bend, width * .89, thickness),
        (s.lerp(e, .57) + bend, width, thickness * .82),
        (s.lerp(e, .82) + .6 * bend, width * .63, thickness * .53),
        (e, .004, .004),
    ], material, 8)


def thread(name, centers, radii, material, sides=8):
    return loft(name, [(c, r, r) for c, r in zip(centers, radii)], material, sides)


def bright_eyes(at, spread, rim, pupil, spark):
    x, y, z = at
    for side in (-1, 1):
        oval("Brow socket", (x + side * spread, y, z), (.126, .083, .071), rim, -.28 * side)
        oval("Eye", (x + side * spread * 1.02, y - .004, z + .052), (.069, .048, .048), pupil)
        oval("Glint", (x + side * spread * 1.02 - .014, y + .015, z + .092), (.015, .015, .010), spark)


def quadruped_leg(side, z, shoulder, fur, foot, hoof=False):
    q = side
    forward = 1 if z > 0. else -1
    x = .33 if z > 0. else .35
    end = z + .13 * forward + (.08 if side == 1 else -.04)
    loft("Shaped leg", [
        ((q * x * .83, shoulder, z), .14, .17),
        ((q * x, .66 * shoulder, z - .10 * forward), .115, .13),
        ((q * x * 1.08, .40 * shoulder, z + .07 * forward), .067 if hoof else .102, .09),
        ((q * x * 1.16, .18, end), .065 if hoof else .095, .081),
        ((q * x * 1.18, .095, end + .04), .064 if hoof else .11, .088),
    ], fur)
    oval("Hoof" if hoof else "Paw", (q * x * 1.19, .102, end + .04),
         (.094 if hoof else .135, .098, .158 if hoof else .195), foot)


def deer():
    bark = paint("Chestnut coat", "995d3b")
    rich = paint("Brow and socks", "51392e")
    cream = paint("Fawn markings", "e2b77e")
    velvet = paint("Antler velvet", "c8a67b")
    moss = paint("Forest moss", "597f4e")
    fern = paint("Fern tips", "a6bd64")
    eye = paint("Polished amber", "291b17", .38)
    glint = paint("Pale honey", "fdf0af")
    loft("High shoulders and tucked flanks", [
        ((0., 1.08, -1.16), .04, .06), ((0., 1.12, -.87), .35, .44),
        ((0., 1.14, -.45), .43, .50), ((0., 1.25, .08), .43, .49),
        ((0., 1.36, .55), .36, .45), ((0., 1.38, .82), .22, .30),
        ((0., 1.46, .95), .09, .10)], bark, 18)
    loft("Raised neck", [((0., 1.27, .46), .29, .36), ((0., 1.61, .68), .30, .32),
        ((0., 1.93, .84), .23, .28), ((0., 2.24, 1.0), .19, .23),
        ((0., 2.39, 1.06), .17, .17)], bark, 16)
    blade("Throat blaze", (0., 1.43, .78), (0., 2.25, 1.22), .17, .045, cream)
    loft("Deer face", [((0., 2.32, .90), .14, .16), ((0., 2.34, 1.15), .235, .23),
        ((0., 2.24, 1.40), .185, .15), ((0., 2.13, 1.60), .11, .093),
        ((0., 2.13, 1.69), .07, .054)], bark, 14)
    oval("Nose", (0., 2.145, 1.667), (.103, .064, .074), rich)
    bright_eyes((0., 2.36, 1.29), .176, cream, eye, glint)
    blade("Rump and short tail", (0., 1.34, -1.01), (0., 1.38, -1.48), .123, .104, cream, .17)
    for s in (-1, 1):
        quadruped_leg(s, -.66, 1.15, bark, rich, hoof=True)
        quadruped_leg(s, .50, 1.25, bark, rich, hoof=True)
        blade("Outer ear", (s*.17, 2.48, 1.05), (s*.55, 2.67, .92), .111, .070, bark)
        blade("Ear velvet", (s*.19, 2.51, 1.079), (s*.51, 2.65, .985), .066, .035, cream)
        beam = [(s*.14, 2.49, 1.02), (s*.25, 2.71, .95), (s*.41, 2.94, .79),
                (s*.61, 3.11, .62), (s*.70, 3.40, .47), (s*.84, 3.50, .36)]
        thread("Antler beam", beam, [.077, .073, .062, .050, .029, .006], velvet)
        for j, offsets in [(1, (.16, .31)), (2, (.22, .36)), (3, (.11, .36))]:
            x, y, z = beam[j]
            wide, rise = offsets
            thread("Brow tine", [(x, y, z), (x + s * wide, y + rise*.62, z + .18),
                                (x + s * wide*.85, y + rise, z + .25)], [.05, .033, .006], velvet)
        for j in range(6):
            z = -.88 + .164*j
            blade("Moss blanket", (s*.16, 1.60 + j*.018, z),
                  (s*(.37 + (j % 2)*.11), 1.48 + j*.033, z+.23), .090, .041,
                  fern if j%3 == 0 else moss, .055)
        for j in range(3):
            x, y, z = beam[1+j]
            blade("Antler fern", (x, y-.035, z), (x+s*.19, y+.08, z-.18), .047, .018, moss, .025)


def wolf():
    frost = paint("Slate winter coat", "526785")
    mane = paint("Blue guard fur", "30445e")
    throat = paint("Pearl undercoat", "b6c6d5")
    mask = paint("Midnight nose and claws", "202537")
    rune = paint("Icy turquoise inlay", "83ded6", .49, .12, .14)
    spark = paint("Glacier glint", "eff7ed")
    loft("Powerful back and haunch", [
        ((0., .98, -1.36), .06, .065), ((0., 1.06, -1.10), .31, .38),
        ((0., 1.08, -.72), .43, .46), ((0., 1.14, -.25), .415, .43),
        ((0., 1.24, .34), .48, .56), ((0., 1.27, .71), .445, .58),
        ((0., 1.24, .99), .267, .35), ((0., 1.27, 1.13), .13, .13)], frost, 18)
    loft("Broad forehead and long jaw", [
        ((0., 1.44, .57), .18, .25), ((0., 1.51, .88), .43, .37),
        ((0., 1.50, 1.14), .385, .32), ((0., 1.36, 1.38), .226, .18),
        ((0., 1.32, 1.70), .158, .11), ((0., 1.35, 1.84), .10, .085)], frost, 16)
    oval("Pale jaw", (0., 1.234, 1.51), (.202, .139, .345), throat)
    oval("Nose", (0., 1.351, 1.833), (.146, .072, .080), mask)
    bright_eyes((0., 1.563, 1.303), .264, mane, rune, spark)
    thread("Curled winter tail", [(0., 1.04, -1.10), (.13, .83, -1.49),
                                 (.24, .69, -1.91), (.42, .84, -2.25), (.49, 1.05, -2.40)],
           [.16, .22, .21, .126, .009], mane, 14)
    for side in (-1, 1):
        quadruped_leg(side, .63, 1.19, frost, mask)
        quadruped_leg(side, -.77, 1.08, frost, mask)
        blade("High pointed ear", (side*.254, 1.61, .86), (side*.42, 2.102, .735), .163, .098, mane, .07)
        blade("Inner ear", (side*.263, 1.78, .955), (side*.393, 2.034, .81), .084, .041, throat)
        for j in range(7):
            x, y, z = side*(.28 + .025*j), 1.69 - .125*j, .855 - .062*j
            blade("Layered ruff", (x,y,z), (x+side*(.20-.012*j), y-.22, z-.66+.027*j),
                  .132, .075, throat if j>2 else mane, .014)
        for j in range(4):
            z = .12 - j*.27
            blade("Guard fur ridge", (side*.20, 1.64-j*.035, z), (side*.44, 1.32, z-.33),
                  .114, .046, mane, .055)
        p = side*.477
        thread("Rune inlay", [(p,1.29,.31),(p*1.02,1.54,.22),(p,1.36,.04),
                             (p*.97,1.20,.21)], [.027,.027,.026,.021], rune)
        thread("Rune crossbar", [(p,1.36,.36),(p,1.39,.04)], [.021,.022], rune)
    thread("Forehead frost line", [(0.,1.79,1.065),(0.,1.72,1.26),(0.,1.61,1.43)], [.042,.040,.012], rune)


def dragon():
    jade = paint("Verdigris scales", "388782")
    belly = paint("Celandine belly", "dfc592")
    fin = paint("Lake blue mane", "24566e")
    horn = paint("Old gold antlers", "b88450", .63, .16)
    plume = paint("Silk whiskers", "e1e7cf")
    eye = paint("Dragon amber", "381d22")
    cloud = paint("Pale mountain cloud", "9cbbbd")
    path = []
    for i in range(28):
        t = i / 27
        x = .60 * math.sin(t * math.tau * 1.32) * (1.1 - .68*t)
        y = .32 + 1.46*t + .22*math.sin(t*math.tau + .08)
        z = -2.25 + 3.60*t
        radius = .035 + .39*t
        path.append(((x,y,z), radius*1.06, radius))
    loft("Continuous jade spine",path,jade,18)
    loft("Long temple belly",[(Vector(p)-Vector((0.,r*.64,0.)),w*.83,r*.50) for p,w,r in path[3:]],belly,14)
    for i,(p,w,r) in enumerate(path[4:-1:2]):
        x,y,z=p
        blade("Dorsal crest", (x,y+r*.74,z+.15), (x+.025,y+r+.20+(i%3)*.032,z-.22),
              .046+i*.004,.078,fin,.028)
        for s in [-1,1]:
            blade("Brushed scale", (x+s*w*.84,y,z+.055), (x+s*w*.95,y-.025,z-.19),
                  .040,.018,fin)
    x,y,z = path[-1][0]
    loft("Long dragon muzzle",[
        ((x,y,z-.23),.20,.19), ((x,y+.16,z+.04),.40,.38),
        ((x,y+.08,z+.33),.39,.28), ((x,y-.015,z+.64),.32,.156),
        ((x,y,z+.84),.26,.138), ((x,y,z+.94),.10,.07)],jade,16)
    oval("Lower lip", (x,y-.11,z+.63), (.32,.092,.37), belly)
    bright_eyes((x,y+.24,z+.483),.268,belly,eye,plume)
    for s in [-1,1]:
        oval("Nostril", (x+s*.195,y+.069,z+.86),(.075,.044,.044),fin)
        p=(x+s*.22,y+.34,z+.04)
        thread("Swept horn", [p,(x+s*.30,y+.63,z-.06),(x+s*.46,y+.67,z-.33),
                             (x+s*.54,y+.85,z-.43)], [.10,.087,.052,.009], horn)
        thread("Horn tine", [(x+s*.30,y+.62,z-.09),(x+s*.41,y+.77,z+.04),
                              (x+s*.45,y+.85,z+.09)], [.054,.034,.006], horn)
        thread("Flowing moustache",[(x+s*.20,y-.03,z+.79),(x+s*.62,y+.03,z+.86),
              (x+s*.86,y+.16,z+.58),(x+s*.95,y+.30,z+.15),(x+s*.82,y+.41,z-.15)],
              [.045,.041,.030,.017,.005],plume)
        for j in range(4):
            blade("Silk frill",(x+s*.20,y+.21-j*.135,z+.19),
                  (x+s*(.65-j*.062),y+.35-j*.16,z-.40),.105,.066,fin if j%2 else jade,.10)
        for i in [12,22]:
            (u,v,w),_,_=path[i]
            h=(u+s*.49,v-.25,w+.20)
            thread("Clawed leg",[(u+s*.19,v-.05,w), (u+s*.44,v-.13,w-.22),h],
                  [.13,.09,.076], jade)
            for j in range(3):
                blade("Dragon claw",h,(h[0]+s*(.07*j),h[1]-.09,h[2]+.21-j*.025),.035,.042,belly)
    for j in range(4):
        oval("Cloud at feet",(-.47+j*.22,.16+.055*math.sin(j),-1.32+j*.055),(.38,.137,.28),cloud)


def serpent():
    skin = paint("Papyrus copper", "a86c47")
    dark = paint("Lapis scales", "225e63")
    sand = paint("Pale belly plates", "e5bd76")
    body = []
    for i in range(44):
        t = i / 43
        a = t * math.tau * 1.31 + .86
        r = 1.08 * (1. - .76*t) + .16
        lift = max(0., (t - .61) / .39)
        radius = .033 + .33*t
        body.append(((r*math.cos(a), .16+lift**1.8*1.76, -.38+r*math.sin(a)), radius*1.05,radius))
    x,y,z=body[-1][0]
    body.extend([((x,y+.24,z+.05),.34,.27),((x,y+.43,z+.18),.29,.24)])
    loft("Coil and rising neck",body,dark,18)
    neck = body[32:]
    loft("Banded underside",[(Vector(p)+Vector((0.,0.,r*.59)),w*.69,r*.48) for p,w,r in neck],sand,14)
    for p,w,r in neck[1:-2:2]:
        px,py,pz=p
        blade("Belly scute",(px-w*.58,py,pz+r*.87),(px+w*.64,py+.009,pz+r*.86),.025,.026,skin)
    for side in [-1,1]:
        # The hood grows out of the upper neck and folds back along the spine.
        vertices, faces = [], []
        for j in range(11):
            t=j/10
            spread=.17 + .54*math.sin(math.pi*t)**.87
            for k in range(6):
                u=k/5
                vertices.append((x+side*(.14+u*spread), y-.63+t*1.07,
                                 z-.09-.23*u*u+.09*math.sin(t*math.pi)))
        for j in range(10):
            for k in range(5):
                a=j*6+k
                order=(a,a+1,a+7,a+6)
                faces.append(order if side>0 else tuple(reversed(order)))
        hood=surface("Sculpted hood",vertices,faces,skin)
        m=hood.modifiers.new("Rounded edge","SOLIDIFY");m.thickness=.07
        for j in range(5):
            t=.18+j*.145
            spread=.17+.54*math.sin(math.pi*t)**.87
            blade("Inlaid hood stripe",(x+side*.31,y-.63+t*1.07,z+.01),
                  (x+side*(.13+spread),y-.63+t*1.07+.08,z-.25),.045,.049,
                  sand if j in (1,3) else dark)
    loft("Flat regal face", [((x,y+.37,z+.06),.22,.21), ((x,y+.40,z+.33),.37,.22),
        ((x,y+.31,z+.61),.30,.155), ((x,y+.26,z+.77),.13,.084)],dark,18)
    oval("Golden cheekline",(x,y+.205,z+.553),(.267,.073,.247),sand)
    bright_eyes((x,y+.466,z+.547), .218, sand, skin, paint("Sun glint", "fff4c6"))
    for side in [-1,1]:
        oval("Small nostril",(x+side*.104,y+.313,z+.745),(.041,.030,.038),skin)
    for j,(p,w,r) in enumerate(body[4:30:2]):
        px,py,pz=p
        oval("Coil dapples",(px,py+r*.89,pz),(.06+j*.006,.034,.11),skin, .08)


def griffin():
    ochre = paint("Lion coat", "b57840")
    under = paint("Warm underfur", "e3ac69")
    ivory = paint("Ivory pinions", "efd7a1")
    night = paint("Blue glazed tips", "376272")
    clay = paint("Vermilion wing stripes", "b45941")
    gold = paint("Brass bird beak", "d99a42",.55,.12)
    eye = paint("Glazed obsidian", "312125",.4)
    loft("Lion's back", [((0.,1.12,-1.30),.06,.12),((0.,1.13,-1.02),.33,.46),
        ((0.,1.20,-.66),.44,.53),((0.,1.23,-.08),.46,.54),
        ((0.,1.42,.50),.53,.57),((0.,1.43,.85),.42,.48),((0.,1.44,1.08),.16,.22)],ochre,18)
    loft("Eagle breast and head", [((0.,.92,.68),.26,.23),((0.,1.42,.68),.48,.47),
        ((0.,1.72,.79),.42,.36),((0.,2.02,.95),.36,.33),((0.,2.18,1.13),.255,.22),
        ((0.,2.21,1.31),.15,.13)],ivory,18)
    loft("Chiseled hooked beak", [((0.,2.09,1.19),.215,.147),((0.,2.11,1.46),.23,.171),
        ((0.,2.075,1.68),.16,.129),((0.,1.977,1.785),.088,.087),
        ((0.,1.869,1.71),.016,.019)],gold,12)
    oval("Lower beak",(0.,1.957,1.506),(.15,.063,.18),night)
    bright_eyes((0.,2.225,1.337),.192,clay,eye,ivory)
    thread("Lion tail",[(0.,1.24,-1.17),(-.18,1.09,-1.48),(-.40,1.12,-1.88),
       (-.54,1.32,-2.04),(-.54,1.57,-2.02),(-.35,1.71,-1.86)], [.135,.105,.081,.066,.052,.025],ochre,10)
    blade("Tail tuft",(-.43,1.65,-1.97),(-.20,1.83,-1.74),.116,.122,clay,.061)
    for s in [-1,1]:
        quadruped_leg(s,-.78,1.19,ochre,under)
        quadruped_leg(s,.66,1.31,ivory,gold)
        for j in range(3):
            blade("Talon",(s*.40+(j-1)*.065,.098,.81),(s*.40+(j-1)*.072,.04,1.079),
                  .033,.036,night,.042)
        for j in range(5):
            blade("Layered eagle cheek",(s*.23,2.14-j*.14,.99),(s*(.42+j*.032),2.12-j*.144,.48),
                  .105,.089,ivory if j%3 else under,.043)
        for j in range(3):
            blade("Crest feather",(s*.10,2.22,.98-j*.085),(s*(.18+j*.063),2.79-j*.11,.49-j*.078),
                  .086,.068,night if j==2 else ivory,.066)
        branch = [(s*.36,1.69,.48),(s*.72,2.09,.34),(s*1.12,2.32,.01),
                  (s*1.53,2.39,-.38),(s*2.01,2.31,-.70)]
        loft("Covered wing shoulder",[(p,w,w*.8) for p,w in zip(branch,[.24,.29,.25,.19,.048])],
             under,12)
        for j in range(8):
            t=j/7
            start=(s*(.56+1.24*t),1.86+.49*t,.30-.91*t)
            end=(s*(1.04+1.34*t),1.83+.79*t,-1.13-.58*t+(.44*t*t))
            blade("Long fan primary",start,end,.159,.060,night if j in [0,2,5] else ivory,.092)
            blade("Red coverts", Vector(start).lerp(Vector(end),.13),Vector(start).lerp(Vector(end),.57),
                  .120,.068,clay if j%2==0 else under,.110)
            blade("Ivory root coverts", (start[0],start[1]+.11,start[2]+.12),
                  (start[0]+s*.21,start[1]+.093,start[2]-.39), .112,.073,ivory,.070)


def bake(name, model, export=True):
    reset()
    model()
    # Join by palette to keep draw calls small on mobile. Apply the hood thickness before joining.
    for mat in bpy.data.materials:
        pieces = [obj for obj in bpy.context.collection.objects if obj.type == "MESH" and obj.data.materials[0] == mat]
        if not pieces:
            continue
        for obj in pieces:
            bpy.context.view_layer.objects.active = obj
            for modifier in list(obj.modifiers):
                bpy.ops.object.modifier_apply(modifier=modifier.name)
        bpy.ops.object.select_all(action="DESELECT")
        for obj in pieces:
            obj.select_set(True)
        bpy.context.view_layer.objects.active = pieces[0]
        if len(pieces) > 1:
            bpy.ops.object.join()
        bpy.context.object.name = mat.name
    if export:
        bpy.ops.export_scene.gltf(filepath=str(MODELS / (name + ".glb")), export_format="GLB",
                                  export_cameras=False, export_lights=False)
    print(name, "faces", sum(len(o.data.polygons) for o in bpy.context.collection.objects if o.type=="MESH"))


if __name__ == "__main__":
    MODELS.mkdir(parents=True,exist_ok=True)
    for key, model in [("valley",deer),("fjord",wolf),("terraces",dragon),("lake",serpent),("aegean",griffin)]:
        bake(key,model)
