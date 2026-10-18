"""
Masterwork Rustic Mossy Wooden Bench for Deep Alchemy
Executes inside Blender 4.5 headless mode.
Generates assets/models/alchemy_props.glb containing the rustic dark-wood bench
with thick green moss accumulation on the backrest top and seat surface,
exactly as specified in the reference concept artwork.
"""
import bpy
import bmesh
import math
import os
import random

# Reset scene
for obj in list(bpy.data.objects):
    bpy.data.objects.remove(obj, do_unlink=True)
for mesh in list(bpy.data.meshes):
    bpy.data.meshes.remove(mesh)
for mat in list(bpy.data.materials):
    bpy.data.materials.remove(mat)

def create_mat(name, color, roughness=0.8, metallic=0.0, emission=None, alpha=1.0):
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    bsdf = next((n for n in nodes if n.type == 'BSDF_PRINCIPLED'), None)
    if bsdf:
        if "Base Color" in bsdf.inputs:
            bsdf.inputs["Base Color"].default_value = (*color[:3], 1.0)
        if "Roughness" in bsdf.inputs:
            bsdf.inputs["Roughness"].default_value = roughness
        if "Metallic" in bsdf.inputs:
            bsdf.inputs["Metallic"].default_value = metallic
        if "Alpha" in bsdf.inputs:
            bsdf.inputs["Alpha"].default_value = alpha
        if alpha < 1.0:
            mat.blend_method = 'BLEND'
        if emission:
            if "Emission Color" in bsdf.inputs:
                bsdf.inputs["Emission Color"].default_value = (*emission[:3], 1.0)
            elif "Emission" in bsdf.inputs:
                bsdf.inputs["Emission"].default_value = (*emission[:3], 1.0)
            if "Emission Strength" in bsdf.inputs:
                bsdf.inputs["Emission Strength"].default_value = 1.8
    return mat

# Materials matching Cel-Shaded Concept Artwork
mat_wood_dark = create_mat("M_Rustic_DarkWood", (0.24, 0.16, 0.10), roughness=0.85)     # Aged dark timber
mat_wood_light = create_mat("M_Rustic_WoodPlank", (0.30, 0.20, 0.13), roughness=0.80)   # Seat planks
mat_moss = create_mat("M_Cavern_Moss", (0.16, 0.44, 0.22), roughness=0.90)              # Thick green velvety moss
mat_vine = create_mat("M_Creeping_Vine", (0.22, 0.55, 0.28), roughness=0.75)            # Climbing foliage
mat_iron_rivet = create_mat("M_RivetIron", (0.18, 0.18, 0.20), roughness=0.45, metallic=0.75)

root_bench = bpy.data.objects.new("Rustic_Mossy_Bench", None)
bpy.context.collection.objects.link(root_bench)

def make_chamfered_box(name, sx, sy, sz, loc, mat, parent=root_bench):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = (sx, sy, sz)
    bpy.ops.object.transform_apply(scale=True)
    obj.data.materials.append(mat)
    obj.parent = parent
    return obj

# ------------------------------------------------------------------------------
# 1. BENCH TIMBER FRAME (Pés, Travessas e Suportes de Madeira Escura)
# ------------------------------------------------------------------------------
# Bench dimensions: length = 1.55m, depth = 0.58m, seat_height = 0.48m, back_height = 0.98m
# 4 Sturdy Legs
leg_radius = 0.048
leg_h = 0.46
leg_locs = [
    (-0.68, -0.22, leg_h * 0.5),
    (-0.68,  0.22, leg_h * 0.5),
    ( 0.68, -0.22, leg_h * 0.5),
    ( 0.68,  0.22, leg_h * 0.5),
]
for idx, (lx, ly, lz) in enumerate(leg_locs):
    bpy.ops.mesh.primitive_cylinder_add(vertices=12, radius=leg_radius, depth=leg_h, location=(lx, ly, lz))
    leg = bpy.context.active_object
    leg.name = f"Bench_Leg_{idx}"
    leg.data.materials.append(mat_wood_dark)
    leg.parent = root_bench

# Lower Longitudinal Stretchers (travessas inferiores)
make_chamfered_box("Stretcher_Front", 1.36, 0.05, 0.06, (0.0, -0.22, 0.15), mat_wood_dark)
make_chamfered_box("Stretcher_Back", 1.36, 0.05, 0.06, (0.0, 0.22, 0.15), mat_wood_dark)
make_chamfered_box("Stretcher_Left", 0.05, 0.44, 0.06, (-0.68, 0.0, 0.15), mat_wood_dark)
make_chamfered_box("Stretcher_Right", 0.05, 0.44, 0.06, (0.68, 0.0, 0.15), mat_wood_dark)

# Upper Seat Support Frame
make_chamfered_box("Seat_Bearer_Left", 0.07, 0.48, 0.06, (-0.68, 0.0, 0.44), mat_wood_dark)
make_chamfered_box("Seat_Bearer_Mid", 0.06, 0.48, 0.06, (0.0, 0.0, 0.44), mat_wood_dark)
make_chamfered_box("Seat_Bearer_Right", 0.07, 0.48, 0.06, (0.68, 0.0, 0.44), mat_wood_dark)

# ------------------------------------------------------------------------------
# 2. SEAT PLANKS (Tábuas Rústicas de Madeira Escura)
# ------------------------------------------------------------------------------
# 3 Parallel Timber Planks along length with small gap between them
plank_w = 0.15
plank_thick = 0.045
plank_len = 1.58
seat_z = 0.49

for ip, py in enumerate([-0.17, 0.0, 0.17]):
    plank = make_chamfered_box(f"Seat_Plank_{ip}", plank_len, plank_w, plank_thick, (0.0, py, seat_z), mat_wood_light)
    # Iron rivets at ends
    for rx in [-0.68, 0.68]:
        bpy.ops.mesh.primitive_cylinder_add(vertices=8, radius=0.012, depth=0.01, location=(rx, py, seat_z + 0.025))
        rivet = bpy.context.active_object
        rivet.name = f"Seat_Rivet_{ip}_{rx}"
        rivet.data.materials.append(mat_iron_rivet)
        rivet.parent = root_bench

# ------------------------------------------------------------------------------
# 3. BACKREST STRUCTURE (Encosto Angulado de Madeira)
# ------------------------------------------------------------------------------
# 3 Upright Backrest Posts angled backward slightly (tilt ~ 7 degrees)
tilt_angle = math.radians(7.0)
post_h = 0.56
post_z = seat_z + 0.28
for px in [-0.68, 0.0, 0.68]:
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(px, 0.25, post_z))
    post = bpy.context.active_object
    post.name = f"Backrest_Post_{px}"
    post.scale = (0.065, 0.055, post_h)
    post.rotation_euler = (tilt_angle, 0.0, 0.0)
    bpy.ops.object.transform_apply(scale=True, rotation=True)
    post.data.materials.append(mat_wood_dark)
    post.parent = root_bench

# Horizontal Backrest Rails (2 wide timber planks across back)
rail_y_mid = 0.25 + math.sin(tilt_angle) * 0.18
rail_z_mid = seat_z + 0.22
make_chamfered_box("Backrest_Rail_Lower", 1.52, 0.04, 0.14, (0.0, rail_y_mid, rail_z_mid), mat_wood_light)

rail_y_top = 0.25 + math.sin(tilt_angle) * 0.45
rail_z_top = seat_z + 0.48
make_chamfered_box("Backrest_Rail_Upper", 1.56, 0.05, 0.14, (0.0, rail_y_top, rail_z_top), mat_wood_dark)

# ------------------------------------------------------------------------------
# 4. THICK VOLUMETRIC GREEN MOSS ACCUMULATION (Diretriz 2: Encosto e Assento)
# ------------------------------------------------------------------------------
def build_moss_layer(name, length, width, thickness, loc, rot=(0.0, 0.0, 0.0), density=18):
    """Generates an organic, puffy, irregular moss cushion with rounded clumps."""
    bm = bmesh.new()
    random.seed(42)
    
    clump_count = density
    for ic in range(clump_count):
        # Position along the plank
        t_len = (ic / (clump_count - 1) - 0.5) * length * 0.94
        t_wid = (random.random() - 0.5) * width * 0.75
        c_thick = thickness * (0.65 + random.random() * 0.6)
        c_rad = (width * 0.45) * (0.8 + random.random() * 0.45)
        
        # Build organic dome clump
        rings = 5
        radials = 8
        clump_verts = []
        for ir in range(rings + 1):
            t_ring = ir / rings
            rz = math.cos(t_ring * math.pi * 0.5) * c_thick
            rr = math.sin(t_ring * math.pi * 0.5) * c_rad
            
            row = []
            for ia in range(radials):
                ang = 2.0 * math.pi * ia / radials
                noise_r = rr * (1.0 + math.sin(ang * 3.0 + ic) * 0.15)
                vx = t_len + math.cos(ang) * noise_r
                vy = t_wid + math.sin(ang) * noise_r
                vz = rz
                row.append(bm.verts.new((vx, vy, vz)))
            clump_verts.append(row)
            
        for ir in range(rings):
            for ia in range(radials):
                ia_next = (ia + 1) % radials
                bm.faces.new((clump_verts[ir][ia], clump_verts[ir][ia_next], clump_verts[ir + 1][ia_next], clump_verts[ir + 1][ia]))
                
        # Top apex
        top_v = bm.verts.new((t_len, t_wid, c_thick))
        for ia in range(radials):
            ia_next = (ia + 1) % radials
            bm.faces.new((clump_verts[rings][ia], clump_verts[rings][ia_next], top_v))

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    
    obj.location = loc
    obj.rotation_euler = rot
    obj.data.materials.append(mat_moss)
    obj.parent = root_bench
    return obj

# A. Thick Moss across the Top Ridge of the Backrest (Borda Superior do Encosto)
build_moss_layer(
    name="Backrest_Top_Moss_Layer",
    length=1.52,
    width=0.08,
    thickness=0.065,
    loc=(0.0, rail_y_top + 0.015, rail_z_top + 0.075),
    rot=(tilt_angle, 0.0, 0.0),
    density=22
)

# B. Organic Moss Patches across Seat Surface (Superfície do Assento)
# Left corner moss cluster
build_moss_layer(
    name="Seat_Moss_Left_Cluster",
    length=0.48,
    width=0.32,
    thickness=0.045,
    loc=(-0.52, 0.08, seat_z + 0.025),
    density=12
)

# Right corner moss cluster
build_moss_layer(
    name="Seat_Moss_Right_Cluster",
    length=0.44,
    width=0.30,
    thickness=0.045,
    loc=(0.52, 0.06, seat_z + 0.025),
    density=11
)

# Central back seam moss patch
build_moss_layer(
    name="Seat_Moss_Center_Seam",
    length=0.72,
    width=0.14,
    thickness=0.038,
    loc=(0.0, 0.16, seat_z + 0.025),
    density=14
)

# ------------------------------------------------------------------------------
# 5. CREEPING VINE FOLIAGE (Vinhas Subindo pelos Pés e Encosto)
# ------------------------------------------------------------------------------
def build_creeping_leaves(name, loc, parent=root_bench):
    bm = bmesh.new()
    leaf_count = 6
    for il in range(leaf_count):
        ang = (il / leaf_count) * math.pi * 2.0
        lx = math.cos(ang) * 0.08
        ly = math.sin(ang) * 0.08
        lz = (il * 0.05)
        # 3-vert diamond leaf
        v0 = bm.verts.new((lx, ly, lz))
        v1 = bm.verts.new((lx + 0.05 * math.cos(ang), ly + 0.05 * math.sin(ang), lz + 0.02))
        v2 = bm.verts.new((lx + 0.02 * math.cos(ang + 0.5), ly + 0.02 * math.sin(ang + 0.5), lz - 0.01))
        bm.faces.new((v0, v1, v2))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    obj.location = loc
    obj.data.materials.append(mat_vine)
    obj.parent = parent
    return obj

build_creeping_leaves("Vine_Back_Left", (-0.68, 0.22, 0.25))
build_creeping_leaves("Vine_Back_Right", (0.68, 0.22, 0.25))

# ------------------------------------------------------------------------------
# 6. EXPORT COMPLETE GLB MODEL
# ------------------------------------------------------------------------------
output_path = os.path.abspath("assets/models/alchemy_props.glb")
os.makedirs(os.path.dirname(output_path), exist_ok=True)

bpy.ops.export_scene.gltf(
    filepath=output_path,
    export_format='GLB',
    use_selection=False,
    export_apply=True,
    export_materials='EXPORT',
)

print(f">>> [BLENDER 4.5] Successfully exported Rustic Mossy Bench to: {output_path}")
