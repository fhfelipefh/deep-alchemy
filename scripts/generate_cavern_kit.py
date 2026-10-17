import bpy
import bmesh
import math
import os
import random
from mathutils import Vector, Matrix, Euler

# Reset scene
for obj in bpy.data.objects:
    bpy.data.objects.remove(obj, do_unlink=True)
for mesh in bpy.data.meshes:
    bpy.data.meshes.remove(mesh)
for mat in bpy.data.materials:
    bpy.data.materials.remove(mat)

def create_mat(name, color, roughness=0.8, metallic=0.0, emission=None):
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = color
        bsdf.inputs["Roughness"].default_value = roughness
        bsdf.inputs["Metallic"].default_value = metallic
        if emission:
            bsdf.inputs["Emission Color"].default_value = emission
            bsdf.inputs["Emission Strength"].default_value = 3.0
    return mat

mat_rock = create_mat("M_CavernRock", (0.16, 0.17, 0.20, 1.0), roughness=0.85)
mat_rock_moss = create_mat("M_RockMoss", (0.12, 0.22, 0.15, 1.0), roughness=0.9)
mat_limestone = create_mat("M_Limestone", (0.75, 0.70, 0.60, 1.0), roughness=0.6)
mat_wood = create_mat("M_BridgeWood", (0.28, 0.18, 0.10, 1.0), roughness=0.75)
mat_rope = create_mat("M_Rope", (0.45, 0.40, 0.30, 1.0), roughness=0.9)
mat_crystal = create_mat("M_CrystalCyan", (0.1, 0.9, 1.0, 1.0), roughness=0.2, emission=(0.1, 0.9, 1.0, 1.0))
mat_waterfall_crest = create_mat("M_WaterFoam", (0.85, 0.95, 1.0, 1.0), roughness=0.1, emission=(0.4, 0.8, 1.0, 1.0))

# 1. Massive Cavern Column / Pillar (Rich faceting like artworks)
def build_massive_pillar(loc):
    bpy.ops.mesh.primitive_cylinder_add(vertices=12, radius=1.8, depth=14.0, location=loc)
    pillar = bpy.context.active_object
    pillar.name = "Cavern_Pillar_Massive"
    pillar.data.materials.append(mat_rock)
    
    # Subdivide and displace vertices to give organic rocky striations
    bm = bmesh.new()
    bm.from_mesh(pillar.data)
    random.seed(42)
    for v in bm.verts:
        # taper in the middle like a natural column
        h = v.co.z
        factor = 1.0 + 0.35 * math.cos(h * 0.4)
        noise = (random.random() - 0.5) * 0.4
        v.co.x = v.co.x * factor + noise
        v.co.y = v.co.y * factor + noise
    bm.to_mesh(pillar.data)
    bm.free()
    
    # Add embedded glowing crystals
    for i in range(4):
        ang = i * 1.57 + 0.3
        cx = loc[0] + math.cos(ang) * 1.6
        cy = loc[1] + math.sin(ang) * 1.6
        cz = loc[2] - 2.0 + i * 1.2
        bpy.ops.mesh.primitive_cylinder_add(vertices=6, radius=0.18, depth=0.8, location=(cx, cy, cz), rotation=(0.4, 0.6 * i, ang))
        cryst = bpy.context.active_object
        cryst.name = f"Crystal_Cluster_{i}"
        cryst.data.materials.append(mat_crystal)
        cryst.parent = pillar

build_massive_pillar((0, 0, 7.0))

# 2. Modular Cavern Wall (4m wide, 8m tall)
def build_wall_segment(loc, name):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    wall = bpy.context.active_object
    wall.name = name
    wall.scale = (4.0, 1.5, 8.0)
    bpy.ops.object.transform_apply(scale=True)
    wall.data.materials.append(mat_rock)
    
    bm = bmesh.new()
    bm.from_mesh(wall.data)
    bmesh.ops.subdivide_edges(bm, edges=bm.edges, cuts=2, use_grid_fill=True)
    random.seed(99)
    for v in bm.verts:
        if abs(v.co.y) > 0.6:
            v.co.y += (random.random() - 0.5) * 0.6
            v.co.x += (random.random() - 0.5) * 0.3
    bm.to_mesh(wall.data)
    bm.free()

build_wall_segment((6.0, 0, 4.0), "Cavern_Wall_Modular")

# 3. Thermal Rim Pool (Limestone terraced pool)
def build_thermal_rim_pool(loc):
    bpy.ops.mesh.primitive_cylinder_add(vertices=16, radius=2.5, depth=0.7, location=loc)
    pool = bpy.context.active_object
    pool.name = "Thermal_Rim_Pool"
    pool.data.materials.append(mat_limestone)
    
    # Inner basin
    inner_loc = (loc[0], loc[1], loc[2] + 0.15)
    bpy.ops.mesh.primitive_cylinder_add(vertices=16, radius=2.1, depth=0.45, location=inner_loc)
    basin = bpy.context.active_object
    basin.name = "Thermal_Pool_Basin"
    basin.data.materials.append(mat_rock_moss)
    basin.parent = pool

build_thermal_rim_pool((-6.0, 0, 0.35))

# 4. Suspension Footbridge (Planks and rope railing)
def build_footbridge(loc):
    bridge_empty = bpy.data.objects.new("Suspension_Bridge", None)
    bpy.context.collection.objects.link(bridge_empty)
    bridge_empty.location = loc
    
    plank_count = 10
    plank_length = 0.9
    for i in range(plank_count):
        y = (i - plank_count / 2) * 0.6
        z_droop = -0.15 * math.sin((i / plank_count) * math.pi)
        bpy.ops.mesh.primitive_cube_add(size=1.0, location=(loc[0], loc[1] + y, loc[2] + z_droop))
        plank = bpy.context.active_object
        plank.name = f"Bridge_Plank_{i}"
        plank.scale = (plank_length * 2.0, 0.45, 0.08)
        bpy.ops.object.transform_apply(scale=True)
        plank.data.materials.append(mat_wood)
        plank.parent = bridge_empty
        
    # Ropes left & right
    for side in [-plank_length, plank_length]:
        bpy.ops.mesh.primitive_cylinder_add(radius=0.04, depth=plank_count * 0.6, location=(loc[0] + side, loc[1], loc[2] + 0.8), rotation=(math.radians(90), 0, 0))
        rope = bpy.context.active_object
        rope.name = f"Bridge_Rope_{'L' if side < 0 else 'R'}"
        rope.data.materials.append(mat_rope)
        rope.parent = bridge_empty

build_footbridge((0, 8.0, 1.5))

# 5. Grand Waterfall Drop Crest / Rock Shelf
def build_waterfall_cliff(loc):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    cliff = bpy.context.active_object
    cliff.name = "Waterfall_Cliff_Shelf"
    cliff.scale = (12.0, 4.0, 10.0)
    bpy.ops.object.transform_apply(scale=True)
    cliff.data.materials.append(mat_rock)
    
    # Water chute crest
    bpy.ops.mesh.primitive_cylinder_add(vertices=12, radius=2.5, depth=4.0, location=(loc[0], loc[1], loc[2] + 4.9), rotation=(0, math.radians(90), 0))
    crest = bpy.context.active_object
    crest.name = "Waterfall_Crest_Foam"
    crest.data.materials.append(mat_waterfall_crest)
    crest.parent = cliff

build_waterfall_cliff((0, -12.0, 5.0))

# Smooth shading
for obj in bpy.data.objects:
    if obj.type == 'MESH':
        for poly in obj.data.polygons:
            poly.use_smooth = True

target_path = r"C:\Users\fhgam\Documents\GitHub\gamelab\assets\models\cavern_kit.glb"
bpy.ops.export_scene.gltf(
    filepath=target_path,
    export_format='GLB',
    use_selection=False,
    export_apply=False,
    export_yup=True
)

print("SUCCESS: Cavern kit exported to", target_path)
