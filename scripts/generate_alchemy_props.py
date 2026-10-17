import bpy
import bmesh
import math
import os
from mathutils import Vector, Matrix, Euler

# Reset scene
for obj in bpy.data.objects:
    bpy.data.objects.remove(obj, do_unlink=True)
for mesh in bpy.data.meshes:
    bpy.data.meshes.remove(mesh)
for mat in bpy.data.materials:
    bpy.data.materials.remove(mat)

def create_mat(name, color, roughness=0.2, metallic=0.0, emission=None, alpha=1.0):
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = color
        bsdf.inputs["Roughness"].default_value = roughness
        bsdf.inputs["Metallic"].default_value = metallic
        if emission:
            bsdf.inputs["Emission Color"].default_value = emission
            bsdf.inputs["Emission Strength"].default_value = 2.5
        if alpha < 1.0:
            bsdf.inputs["Alpha"].default_value = alpha
            mat.blend_method = 'BLEND'
    return mat

mat_glass = create_mat("M_Glass", (0.85, 0.95, 1.0, 0.4), roughness=0.1, alpha=0.5)
mat_cork = create_mat("M_Cork", (0.6, 0.45, 0.3, 1.0), roughness=0.9)
mat_potion_blue = create_mat("M_PotionBlue", (0.05, 0.75, 1.0, 0.85), roughness=0.1, emission=(0.1, 0.8, 1.0, 1.0))
mat_potion_red = create_mat("M_PotionRed", (0.95, 0.15, 0.15, 0.85), roughness=0.1, emission=(1.0, 0.2, 0.2, 1.0))
mat_potion_green = create_mat("M_PotionGreen", (0.15, 0.95, 0.35, 0.85), roughness=0.1, emission=(0.2, 1.0, 0.4, 1.0))
mat_potion_gold = create_mat("M_PotionGold", (1.0, 0.8, 0.1, 0.85), roughness=0.1, emission=(1.0, 0.85, 0.2, 1.0))

mat_brass = create_mat("M_PropBrass", (0.85, 0.65, 0.25, 1.0), roughness=0.3, metallic=0.85)
mat_iron = create_mat("M_ChestIron", (0.2, 0.2, 0.22, 1.0), roughness=0.5, metallic=0.7)
mat_wood = create_mat("M_ChestWood", (0.35, 0.22, 0.12, 1.0), roughness=0.7)
mat_stone = create_mat("M_MortarStone", (0.3, 0.3, 0.32, 1.0), roughness=0.8)

mat_mush_cap = create_mat("M_MushroomCap", (0.05, 0.85, 0.95, 1.0), roughness=0.4, emission=(0.1, 0.9, 1.0, 1.0))
mat_mush_stem = create_mat("M_MushroomStem", (0.85, 0.92, 0.95, 1.0), roughness=0.6)
mat_mush_spore = create_mat("M_SporeYellow", (1.0, 0.95, 0.2, 1.0), roughness=0.3, emission=(1.0, 0.9, 0.3, 1.0))

# 1. Round Potion Flask (Cyan Stamina Elixir)
def build_potion_round(loc, name, liquid_mat):
    root = bpy.data.objects.new(name, None)
    bpy.context.collection.objects.link(root)
    root.location = loc
    
    # Outer Glass Bulb
    bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=12, radius=0.12, location=loc)
    bulb = bpy.context.active_object
    bulb.name = f"{name}_GlassBulb"
    bulb.data.materials.append(mat_glass)
    bulb.parent = root
    
    # Inner Liquid Bulb
    bpy.ops.mesh.primitive_uv_sphere_add(segments=14, ring_count=10, radius=0.105, location=loc)
    liquid = bpy.context.active_object
    liquid.name = f"{name}_Liquid"
    liquid.data.materials.append(liquid_mat)
    liquid.parent = root
    
    # Glass Neck
    bpy.ops.mesh.primitive_cylinder_add(radius=0.04, depth=0.10, location=(loc[0], loc[1], loc[2] + 0.14))
    neck = bpy.context.active_object
    neck.name = f"{name}_Neck"
    neck.data.materials.append(mat_glass)
    neck.parent = root
    
    # Brass Collar
    bpy.ops.mesh.primitive_torus_add(major_radius=0.045, minor_radius=0.01, location=(loc[0], loc[1], loc[2] + 0.17))
    collar = bpy.context.active_object
    collar.name = f"{name}_Collar"
    collar.data.materials.append(mat_brass)
    collar.parent = root
    
    # Cork Stopper
    bpy.ops.mesh.primitive_cylinder_add(radius=0.038, depth=0.06, location=(loc[0], loc[1], loc[2] + 0.20))
    cork = bpy.context.active_object
    cork.name = f"{name}_Cork"
    cork.data.materials.append(mat_cork)
    cork.parent = root

build_potion_round((0, 0, 0), "Potion_Cyan", mat_potion_blue)
build_potion_round((0.5, 0, 0), "Potion_Health", mat_potion_red)
build_potion_round((1.0, 0, 0), "Potion_Antidote", mat_potion_green)

# 2. Conical / Erlenmeyer Flask (Golden Sun Elixir)
def build_potion_cone(loc, name, liquid_mat):
    root = bpy.data.objects.new(name, None)
    bpy.context.collection.objects.link(root)
    root.location = loc
    
    bpy.ops.mesh.primitive_cone_add(vertices=16, radius1=0.13, radius2=0.04, depth=0.22, location=(loc[0], loc[1], loc[2] + 0.11))
    body = bpy.context.active_object
    body.name = f"{name}_Body"
    body.data.materials.append(mat_glass)
    body.parent = root
    
    bpy.ops.mesh.primitive_cone_add(vertices=14, radius1=0.11, radius2=0.035, depth=0.16, location=(loc[0], loc[1], loc[2] + 0.08))
    liq = bpy.context.active_object
    liq.name = f"{name}_Liquid"
    liq.data.materials.append(liquid_mat)
    liq.parent = root
    
    bpy.ops.mesh.primitive_cylinder_add(radius=0.035, depth=0.05, location=(loc[0], loc[1], loc[2] + 0.24))
    cork = bpy.context.active_object
    cork.name = f"{name}_Cork"
    cork.data.materials.append(mat_cork)
    cork.parent = root

build_potion_cone((1.5, 0, 0), "Potion_Sunfire", mat_potion_gold)

# 3. Bioluminescent Cavern Mushroom
def build_glowing_mushroom(loc, name):
    root = bpy.data.objects.new(name, None)
    bpy.context.collection.objects.link(root)
    root.location = loc
    
    # Stem
    bpy.ops.mesh.primitive_cylinder_add(vertices=12, radius=0.08, depth=0.6, location=(loc[0], loc[1], loc[2] + 0.3))
    stem = bpy.context.active_object
    stem.name = f"{name}_Stem"
    stem.data.materials.append(mat_mush_stem)
    stem.parent = root
    
    # Cap (Hemisphere / flattened sphere)
    bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=12, radius=0.35, location=(loc[0], loc[1], loc[2] + 0.6))
    cap = bpy.context.active_object
    cap.scale = (1.0, 1.0, 0.45)
    bpy.ops.object.transform_apply(scale=True)
    cap.name = f"{name}_Cap"
    cap.data.materials.append(mat_mush_cap)
    cap.parent = root
    
    # Spores on cap
    for i in range(5):
        ang = i * 1.25
        r = 0.20
        sx = loc[0] + math.cos(ang) * r
        sy = loc[1] + math.sin(ang) * r
        sz = loc[2] + 0.72
        bpy.ops.mesh.primitive_uv_sphere_add(segments=8, ring_count=6, radius=0.03, location=(sx, sy, sz))
        spore = bpy.context.active_object
        spore.name = f"{name}_Spore_{i}"
        spore.data.materials.append(mat_mush_spore)
        spore.parent = root

build_glowing_mushroom((2.2, 0, 0), "Mushroom_Cyan")

# 4. Alchemist Treasure Chest (Elder Scrolls style)
def build_alchemist_chest(loc):
    root = bpy.data.objects.new("Alchemist_Chest", None)
    bpy.context.collection.objects.link(root)
    root.location = loc
    
    # Chest Base
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(loc[0], loc[1], loc[2] + 0.3))
    base = bpy.context.active_object
    base.name = "Chest_Base"
    base.scale = (0.8, 0.5, 0.5)
    bpy.ops.object.transform_apply(scale=True)
    base.data.materials.append(mat_wood)
    base.parent = root
    
    # Lid
    bpy.ops.mesh.primitive_cylinder_add(vertices=16, radius=0.25, depth=0.8, location=(loc[0], loc[1], loc[2] + 0.55), rotation=(0, math.radians(90), 0))
    lid = bpy.context.active_object
    lid.name = "Chest_Lid"
    lid.data.materials.append(mat_wood)
    lid.parent = root
    
    # Iron Bands
    for x_off in [-0.25, 0.25]:
        bpy.ops.mesh.primitive_cube_add(size=1.0, location=(loc[0] + x_off, loc[1], loc[2] + 0.3))
        band = bpy.context.active_object
        band.scale = (0.06, 0.52, 0.52)
        bpy.ops.object.transform_apply(scale=True)
        band.data.materials.append(mat_iron)
        band.parent = root
        
    # Lock Plate
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(loc[0], loc[1] + 0.26, loc[2] + 0.45))
    lock = bpy.context.active_object
    lock.scale = (0.1, 0.04, 0.12)
    bpy.ops.object.transform_apply(scale=True)
    lock.data.materials.append(mat_brass)
    lock.parent = root

build_alchemist_chest((3.5, 0, 0))

# 5. Stone Mortar & Pestle
def build_mortar_pestle(loc):
    root = bpy.data.objects.new("Mortar_Pestle", None)
    bpy.context.collection.objects.link(root)
    root.location = loc
    
    # Bowl
    bpy.ops.mesh.primitive_cylinder_add(vertices=16, radius=0.18, depth=0.16, location=(loc[0], loc[1], loc[2] + 0.08))
    bowl = bpy.context.active_object
    bowl.name = "Mortar_Bowl"
    bowl.data.materials.append(mat_stone)
    bowl.parent = root
    
    # Pestle
    bpy.ops.mesh.primitive_cylinder_add(radius=0.035, depth=0.28, location=(loc[0] + 0.05, loc[1], loc[2] + 0.16), rotation=(0.2, 0.3, 0))
    pestle = bpy.context.active_object
    pestle.name = "Mortar_Pestle"
    pestle.data.materials.append(mat_stone)
    pestle.parent = root

build_mortar_pestle((4.8, 0, 0))

# Smooth shading
for obj in bpy.data.objects:
    if obj.type == 'MESH':
        for poly in obj.data.polygons:
            poly.use_smooth = True

target_path = r"C:\Users\fhgam\Documents\GitHub\gamelab\assets\models\alchemy_props.glb"
bpy.ops.export_scene.gltf(
    filepath=target_path,
    export_format='GLB',
    use_selection=False,
    export_apply=False,
    export_yup=True
)

print("SUCCESS: Alchemy props exported to", target_path)
