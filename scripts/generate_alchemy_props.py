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
mat_wood_dark = create_mat("M_WorkbenchWood", (0.22, 0.14, 0.08, 1.0), roughness=0.75)
mat_wood_chest = create_mat("M_ChestWood", (0.35, 0.22, 0.12, 1.0), roughness=0.7)
mat_parchment = create_mat("M_Parchment", (0.88, 0.82, 0.68, 1.0), roughness=0.85)
mat_stone = create_mat("M_MortarStone", (0.30, 0.30, 0.32, 1.0), roughness=0.8)

# Root Container for complete Workbench workstation
root_bench = bpy.data.objects.new("Alchemist_Workstation", None)
bpy.context.collection.objects.link(root_bench)

# ------------------------------------------------------------------------------
# 1. RUSTIC WOODEN ALCHEMY WORKBENCH (Mesa de Alquimia com Pés e Tampo)
# ------------------------------------------------------------------------------
def build_workbench():
    # 4 Thick Legs
    leg_coords = [
        (-0.72, -0.30),
        (-0.72, 0.30),
        (0.72, -0.30),
        (0.72, 0.30),
    ]
    for i, (lx, ly) in enumerate(leg_coords):
        bpy.ops.mesh.primitive_cube_add(size=1.0, location=(lx, ly, 0.41))
        leg = bpy.context.active_object
        leg.name = f"Workbench_Leg_{i}"
        leg.scale = (0.10, 0.10, 0.82)
        bpy.ops.object.transform_apply(scale=True)
        leg.data.materials.append(mat_wood_dark)
        leg.parent = root_bench

    # Lower shelf stretcher braces
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(0.0, 0.0, 0.18))
    stretcher = bpy.context.active_object
    stretcher.name = "Workbench_Stretcher"
    stretcher.scale = (1.44, 0.50, 0.04)
    bpy.ops.object.transform_apply(scale=True)
    stretcher.data.materials.append(mat_wood_dark)
    stretcher.parent = root_bench

    # Main Heavy Tabletop Planks (Top surface at Z = 0.86m)
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(0.0, 0.0, 0.82))
    top = bpy.context.active_object
    top.name = "Workbench_Tabletop"
    top.scale = (1.70, 0.78, 0.08)
    bpy.ops.object.transform_apply(scale=True)
    top.data.materials.append(mat_wood_dark)
    top.parent = root_bench

    # Back Risers & Upper Potion Rack (Shelf at Z = 1.15m)
    for rx in [-0.70, 0.70]:
        bpy.ops.mesh.primitive_cube_add(size=1.0, location=(rx, 0.28, 1.00))
        riser = bpy.context.active_object
        riser.scale = (0.06, 0.16, 0.30)
        bpy.ops.object.transform_apply(scale=True)
        riser.data.materials.append(mat_wood_dark)
        riser.parent = root_bench

    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(0.0, 0.28, 1.15))
    shelf = bpy.context.active_object
    shelf.name = "Workbench_PotionShelf"
    shelf.scale = (1.56, 0.22, 0.04)
    bpy.ops.object.transform_apply(scale=True)
    shelf.data.materials.append(mat_wood_dark)
    shelf.parent = root_bench

build_workbench()

# ------------------------------------------------------------------------------
# 2. POTION FLASKS PLACED PROPERLY ON THE UPPER WORKBENCH SHELF (Z = 1.17m)
# ------------------------------------------------------------------------------
def build_potion_round(loc, name, liquid_mat):
    root = bpy.data.objects.new(name, None)
    bpy.context.collection.objects.link(root)
    root.location = loc
    root.parent = root_bench
    
    # Outer Glass Bulb (Radius 0.08m, centered at Z = 0.08m above shelf)
    bpy.ops.mesh.primitive_uv_sphere_add(segments=14, ring_count=10, radius=0.075, location=(loc[0], loc[1], loc[2] + 0.075))
    bulb = bpy.context.active_object
    bulb.name = f"{name}_GlassBulb"
    bulb.data.materials.append(mat_glass)
    bulb.parent = root
    
    # Inner Liquid Bulb
    bpy.ops.mesh.primitive_uv_sphere_add(segments=12, ring_count=8, radius=0.068, location=(loc[0], loc[1], loc[2] + 0.072))
    liquid = bpy.context.active_object
    liquid.name = f"{name}_Liquid"
    liquid.data.materials.append(liquid_mat)
    liquid.parent = root
    
    # Glass Neck
    bpy.ops.mesh.primitive_cylinder_add(radius=0.026, depth=0.07, location=(loc[0], loc[1], loc[2] + 0.16))
    neck = bpy.context.active_object
    neck.name = f"{name}_Neck"
    neck.data.materials.append(mat_glass)
    neck.parent = root
    
    # Brass Collar
    bpy.ops.mesh.primitive_torus_add(major_radius=0.030, minor_radius=0.007, location=(loc[0], loc[1], loc[2] + 0.18))
    collar = bpy.context.active_object
    collar.name = f"{name}_Collar"
    collar.data.materials.append(mat_brass)
    collar.parent = root
    
    # Cork Stopper
    bpy.ops.mesh.primitive_cylinder_add(radius=0.025, depth=0.045, location=(loc[0], loc[1], loc[2] + 0.20))
    cork = bpy.context.active_object
    cork.name = f"{name}_Cork"
    cork.data.materials.append(mat_cork)
    cork.parent = root

def build_potion_cone(loc, name, liquid_mat):
    root = bpy.data.objects.new(name, None)
    bpy.context.collection.objects.link(root)
    root.location = loc
    root.parent = root_bench
    
    bpy.ops.mesh.primitive_cone_add(vertices=14, radius1=0.08, radius2=0.028, depth=0.15, location=(loc[0], loc[1], loc[2] + 0.075))
    body = bpy.context.active_object
    body.name = f"{name}_Body"
    body.data.materials.append(mat_glass)
    body.parent = root
    
    bpy.ops.mesh.primitive_cone_add(vertices=12, radius1=0.07, radius2=0.025, depth=0.11, location=(loc[0], loc[1], loc[2] + 0.055))
    liq = bpy.context.active_object
    liq.name = f"{name}_Liquid"
    liq.data.materials.append(liquid_mat)
    liq.parent = root
    
    bpy.ops.mesh.primitive_cylinder_add(radius=0.024, depth=0.04, location=(loc[0], loc[1], loc[2] + 0.165))
    cork = bpy.context.active_object
    cork.name = f"{name}_Cork"
    cork.data.materials.append(mat_cork)
    cork.parent = root

# Placed neatly on upper shelf (Z = 1.17)
shelf_z = 1.17
build_potion_round((-0.45, 0.28, shelf_z), "Potion_Cyan", mat_potion_blue)
build_potion_round((-0.15, 0.28, shelf_z), "Potion_Health", mat_potion_red)
build_potion_round((0.15, 0.28, shelf_z), "Potion_Antidote", mat_potion_green)
build_potion_cone((0.45, 0.28, shelf_z), "Potion_Sunfire", mat_potion_gold)

# ------------------------------------------------------------------------------
# 3. PROPS ON THE MAIN TABLETOP (Z = 0.86m)
# ------------------------------------------------------------------------------
# Stone Mortar & Pestle on left side of tabletop
def build_mortar_pestle(loc):
    root = bpy.data.objects.new("Mortar_Pestle", None)
    bpy.context.collection.objects.link(root)
    root.location = loc
    root.parent = root_bench
    
    # Bowl resting on table
    bpy.ops.mesh.primitive_cylinder_add(vertices=16, radius=0.14, depth=0.12, location=(loc[0], loc[1], loc[2] + 0.06))
    bowl = bpy.context.active_object
    bowl.name = "Mortar_Bowl"
    bowl.data.materials.append(mat_stone)
    bowl.parent = root
    
    # Pestle angled in bowl
    bpy.ops.mesh.primitive_cylinder_add(radius=0.028, depth=0.22, location=(loc[0] + 0.04, loc[1], loc[2] + 0.12), rotation=(0.25, 0.28, 0))
    pestle = bpy.context.active_object
    pestle.name = "Mortar_Pestle"
    pestle.data.materials.append(mat_stone)
    pestle.parent = root

build_mortar_pestle((-0.48, -0.06, 0.86))

# Open Alchemy Parchment in the middle of the tabletop
def build_alchemy_parchment(loc):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(loc[0], loc[1], loc[2] + 0.005))
    doc = bpy.context.active_object
    doc.name = "Alchemy_Grimoire_Page"
    doc.scale = (0.34, 0.24, 0.008)
    bpy.ops.object.transform_apply(scale=True)
    doc.data.materials.append(mat_parchment)
    doc.parent = root_bench

build_alchemy_parchment((0.02, -0.08, 0.86))

# Brass alembic retort vessel on right side of tabletop
def build_brass_alembic(loc):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=14, ring_count=10, radius=0.10, location=(loc[0], loc[1], loc[2] + 0.10))
    flask = bpy.context.active_object
    flask.name = "Brass_Retort_Flask"
    flask.data.materials.append(mat_brass)
    flask.parent = root_bench
    
    bpy.ops.mesh.primitive_cylinder_add(radius=0.022, depth=0.16, location=(loc[0] + 0.04, loc[1], loc[2] + 0.16), rotation=(0.0, 0.55, 0.0))
    spout = bpy.context.active_object
    spout.name = "Brass_Retort_Spout"
    spout.data.materials.append(mat_brass)
    spout.parent = root_bench

build_brass_alembic((0.46, -0.04, 0.86))

# ------------------------------------------------------------------------------
# 4. ALCHEMIST TREASURE CHEST (Resting solidly on floor beside table at Z = 0)
# ------------------------------------------------------------------------------
def build_alchemist_chest(loc):
    root = bpy.data.objects.new("Alchemist_Chest", None)
    bpy.context.collection.objects.link(root)
    root.location = loc
    root.parent = root_bench
    
    # Chest Base resting on floor
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(loc[0], loc[1], loc[2] + 0.25))
    base = bpy.context.active_object
    base.name = "Chest_Base"
    base.scale = (0.70, 0.44, 0.45)
    bpy.ops.object.transform_apply(scale=True)
    base.data.materials.append(mat_wood_chest)
    base.parent = root
    
    # Curved Lid
    bpy.ops.mesh.primitive_cylinder_add(vertices=16, radius=0.22, depth=0.70, location=(loc[0], loc[1], loc[2] + 0.48), rotation=(0, math.radians(90), 0))
    lid = bpy.context.active_object
    lid.name = "Chest_Lid"
    lid.data.materials.append(mat_wood_chest)
    lid.parent = root
    
    # Iron Bands
    for x_off in [-0.22, 0.22]:
        bpy.ops.mesh.primitive_cube_add(size=1.0, location=(loc[0] + x_off, loc[1], loc[2] + 0.25))
        band = bpy.context.active_object
        band.scale = (0.05, 0.46, 0.46)
        bpy.ops.object.transform_apply(scale=True)
        band.data.materials.append(mat_iron)
        band.parent = root
        
    # Lock Plate
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(loc[0], loc[1] + 0.23, loc[2] + 0.38))
    lock = bpy.context.active_object
    lock.scale = (0.08, 0.03, 0.10)
    bpy.ops.object.transform_apply(scale=True)
    lock.data.materials.append(mat_brass)
    lock.parent = root

# Placed to the left of the workbench on the floor
build_alchemist_chest((-1.35, 0.0, 0.0))

# Smooth shading on all meshes
for obj in bpy.data.objects:
    if obj.type == 'MESH':
        for poly in obj.data.polygons:
            poly.use_smooth = True

target_path = os.path.abspath("assets/models/alchemy_props.glb")
os.makedirs(os.path.dirname(target_path), exist_ok=True)

bpy.ops.export_scene.gltf(
    filepath=target_path,
    export_format='GLB',
    use_selection=False,
    export_apply=False,
    export_yup=True
)

print(f">>> [BLENDER 4.5] Successfully exported Masterwork Alchemy Workstation to: {target_path}")
