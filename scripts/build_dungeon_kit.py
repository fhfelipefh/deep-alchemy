"""
Masterwork Procedural Dungeon 3D Modular Asset Kit Generator for Deep Alchemy
Generates modular cel-shaded cavern pieces for the Dual Grid procedural paths.
Compatible with Blender 4.5 LTS headless execution.
"""

import bpy
import bmesh
import math
import os
import random

# Clear existing objects and collections
bpy.ops.wm.read_factory_settings(use_empty=True)

# -----------------------------------------------------------------------------
# Material Factory (Cel-Shaded PBR Palette with HDR Emission for Bloom)
# -----------------------------------------------------------------------------
def create_pbr_material(name, base_color, roughness=0.8, metallic=0.0, emission=None, emission_strength=1.0):
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    bsdf = nodes.get("Principled BSDF")
    if bsdf:
        if "Base Color" in bsdf.inputs:
            bsdf.inputs["Base Color"].default_value = (*base_color[:3], 1.0)
        if "Roughness" in bsdf.inputs:
            bsdf.inputs["Roughness"].default_value = roughness
        if "Metallic" in bsdf.inputs:
            bsdf.inputs["Metallic"].default_value = metallic
        if emission:
            if "Emission Color" in bsdf.inputs:
                bsdf.inputs["Emission Color"].default_value = (*emission[:3], 1.0)
            elif "Emission" in bsdf.inputs:
                bsdf.inputs["Emission"].default_value = (*emission[:3], 1.0)
            if "Emission Strength" in bsdf.inputs:
                bsdf.inputs["Emission Strength"].default_value = emission_strength
    return mat

mat_stone = create_pbr_material("M_Dungeon_Stone", (0.18, 0.24, 0.30), roughness=0.85)
mat_floor = create_pbr_material("M_Dungeon_Floor", (0.38, 0.28, 0.20), roughness=0.88)
mat_moss = create_pbr_material("M_Dungeon_Moss", (0.16, 0.45, 0.26), roughness=0.80)
mat_iron = create_pbr_material("M_Dungeon_Iron", (0.12, 0.13, 0.15), roughness=0.35, metallic=0.85)
mat_gold_rune = create_pbr_material("M_Gold_Rune", (1.0, 0.78, 0.25), roughness=0.25, emission=(1.0, 0.78, 0.25), emission_strength=4.2)
mat_cyan_crystal = create_pbr_material("M_Cyan_Crystal", (0.20, 0.95, 1.0), roughness=0.15, emission=(0.20, 0.95, 1.0), emission_strength=4.8)
mat_ember = create_pbr_material("M_Brazier_Ember", (1.0, 0.45, 0.10), roughness=0.20, emission=(1.0, 0.45, 0.10), emission_strength=5.0)

# Helper to finalize and smooth mesh
def finalize_mesh(bm, obj, mesh, materials):
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    for poly in mesh.polygons:
        poly.use_smooth = True
    mesh.update()
    for m in materials:
        obj.data.materials.append(m)

# -----------------------------------------------------------------------------
# 1. Straight Corridor Section (4m x 4m) with Arched Vault Ribs
# -----------------------------------------------------------------------------
def build_corridor_straight():
    mesh = bpy.data.meshes.new("Tile_Corridor_Straight_Mesh")
    obj = bpy.data.objects.new("Tile_Corridor_Straight", mesh)
    bpy.context.collection.objects.link(obj)
    bm = bmesh.new()

    tile_size = 4.0
    h_w = tile_size * 0.5
    h_vault = 3.6

    # 1. Flagstone floor with subtle crown curvature
    u_segs = 8
    v_segs = 8
    floor_grid = []
    for iv in range(v_segs + 1):
        v_f = (iv / v_segs) - 0.5
        row = []
        for iu in range(u_segs + 1):
            u_f = (iu / u_segs) - 0.5
            x = u_f * tile_size
            y = v_f * tile_size
            z = math.cos(u_f * math.pi) * 0.06
            vert = bm.verts.new((x, y, z))
            row.append(vert)
        floor_grid.append(row)

    bm.verts.ensure_lookup_table()
    for iv in range(v_segs):
        for iu in range(u_segs):
            face = bm.faces.new((floor_grid[iv][iu], floor_grid[iv][iu+1], floor_grid[iv+1][iu+1], floor_grid[iv+1][iu]))
            face.material_index = 0 # mat_floor

    # 2. Left Wall Rock with Arched Rib
    for side in [-1.0, 1.0]:
        wall_segs = 8
        wall_strip = []
        for iz in range(wall_segs + 1):
            t = iz / wall_segs
            w_z = t * h_vault
            # Inward arching curve
            w_x = (h_w - (math.sin(t * math.pi * 0.5) ** 1.6) * 0.65) * side
            row = []
            for iv in range(v_segs + 1):
                v_f = (iv / v_segs) - 0.5
                w_y = v_f * tile_size
                # Organic rock facet displacement
                disp = math.sin(w_z * 2.0 + w_y * 1.5) * 0.08
                vert = bm.verts.new((w_x + disp * side, w_y, w_z))
                row.append(vert)
            wall_strip.append(row)

        for iz in range(wall_segs):
            for iv in range(v_segs):
                face = bm.faces.new((wall_strip[iz][iv], wall_strip[iz+1][iv], wall_strip[iz+1][iv+1], wall_strip[iz][iv+1]))
                face.material_index = 1 if iz < 3 else 2 # moss at bottom, stone above

    finalize_mesh(bm, obj, mesh, [mat_floor, mat_moss, mat_stone])

build_corridor_straight()

# -----------------------------------------------------------------------------
# 2. Corner Outer Bend (Smooth curved 90-degree cavern elbow)
# -----------------------------------------------------------------------------
def build_corridor_corner():
    mesh = bpy.data.meshes.new("Tile_Corridor_Corner_Mesh")
    obj = bpy.data.objects.new("Tile_Corridor_Corner", mesh)
    bpy.context.collection.objects.link(obj)
    bm = bmesh.new()

    tile_size = 4.0
    h_w = tile_size * 0.5
    h_vault = 3.6

    # Curved floor quarter turn
    rad_steps = 10
    ring_steps = 6
    r_inner = 0.6
    r_outer = tile_size * 0.95

    rings = []
    for ir in range(ring_steps + 1):
        r_f = ir / ring_steps
        curr_r = r_inner + r_f * (r_outer - r_inner)
        row = []
        for ia in range(rad_steps + 1):
            angle = (ia / rad_steps) * math.pi * 0.5
            # Center of turn at corner (-h_w, -h_w)
            x = -h_w + math.cos(angle) * curr_r
            y = -h_w + math.sin(angle) * curr_r
            z = 0.02
            vert = bm.verts.new((x, y, z))
            row.append(vert)
        rings.append(row)

    bm.verts.ensure_lookup_table()
    for ir in range(ring_steps):
        for ia in range(rad_steps):
            face = bm.faces.new((rings[ir][ia], rings[ir][ia+1], rings[ir+1][ia+1], rings[ir+1][ia]))
            face.material_index = 0

    # Outer wall curved sweep
    outer_wall = []
    wall_z_steps = 6
    for iz in range(wall_z_steps + 1):
        t = iz / wall_z_steps
        w_z = t * h_vault
        overhang = math.sin(t * math.pi * 0.5) * 0.55
        curr_r = r_outer - overhang
        row = []
        for ia in range(rad_steps + 1):
            angle = (ia / rad_steps) * math.pi * 0.5
            x = -h_w + math.cos(angle) * curr_r
            y = -h_w + math.sin(angle) * curr_r
            vert = bm.verts.new((x, y, w_z))
            row.append(vert)
        outer_wall.append(row)

    for iz in range(wall_z_steps):
        for ia in range(rad_steps):
            face = bm.faces.new((outer_wall[iz][ia], outer_wall[iz][ia+1], outer_wall[iz+1][ia+1], outer_wall[iz+1][ia]))
            face.material_index = 1 # mat_stone

    finalize_mesh(bm, obj, mesh, [mat_floor, mat_stone, mat_moss])

build_corridor_corner()

# -----------------------------------------------------------------------------
# 3. Solid Cavern Pillar (4m x 4m Solid Quad with Chiseled Runes)
# -----------------------------------------------------------------------------
def build_solid_pillar():
    mesh = bpy.data.meshes.new("Tile_Solid_Pillar_Mesh")
    obj = bpy.data.objects.new("Tile_Solid_Pillar", mesh)
    bpy.context.collection.objects.link(obj)
    bm = bmesh.new()

    tile_size = 4.0
    h_w = tile_size * 0.46
    h_pillar = 4.2

    # Cylindrical organic rock pillar with faceted strata
    rings = 8
    radials = 16
    ring_verts = []
    for ir in range(rings + 1):
        t = ir / rings
        z = t * h_pillar
        # Flaring at top and bottom
        flare = ((1.0 - t) ** 3.0) * 0.35 + (t ** 3.0) * 0.30
        r = (h_w + flare)
        row = []
        for ia in range(radials):
            angle = (ia / radials) * math.pi * 2.0
            disp = math.sin(angle * 4.0 + t * 3.0) * 0.12
            x = math.cos(angle) * (r + disp)
            y = math.sin(angle) * (r + disp)
            vert = bm.verts.new((x, y, z))
            row.append(vert)
        ring_verts.append(row)

    bm.verts.ensure_lookup_table()
    for ir in range(rings):
        for ia in range(radials):
            ia_next = (ia + 1) % radials
            face = bm.faces.new((ring_verts[ir][ia], ring_verts[ir][ia_next], ring_verts[ir+1][ia_next], ring_verts[ir+1][ia]))
            face.material_index = 0 if ir >= 2 else 1 # moss base, stone above

    finalize_mesh(bm, obj, mesh, [mat_stone, mat_moss])

build_solid_pillar()

# -----------------------------------------------------------------------------
# 4. Alchemical Shrine Pedestal (Macro Chamber Feature)
# -----------------------------------------------------------------------------
def build_alchemical_shrine():
    mesh = bpy.data.meshes.new("Macro_Alchemical_Shrine_Mesh")
    obj = bpy.data.objects.new("Macro_Alchemical_Shrine", mesh)
    bpy.context.collection.objects.link(obj)
    bm = bmesh.new()

    # Stepped circular dais
    steps = 3
    base_r = 2.4
    h_step = 0.22
    for s in range(steps):
        r = base_r - s * 0.45
        z_bot = s * h_step
        z_top = (s + 1) * h_step
        radials = 24

        ring_bot = [bm.verts.new((math.cos(i * math.pi * 2 / radials) * r, math.sin(i * math.pi * 2 / radials) * r, z_bot)) for i in range(radials)]
        ring_top = [bm.verts.new((math.cos(i * math.pi * 2 / radials) * r, math.sin(i * math.pi * 2 / radials) * r, z_top)) for i in range(radials)]

        for i in range(radials):
            i_next = (i + 1) % radials
            f = bm.faces.new((ring_bot[i], ring_bot[i_next], ring_top[i_next], ring_top[i]))
            f.material_index = 0

        # Top step rune cap
        if s == steps - 1:
            center_v = bm.verts.new((0.0, 0.0, z_top))
            for i in range(radials):
                i_next = (i + 1) % radials
                f = bm.faces.new((ring_top[i], ring_top[i_next], center_v))
                f.material_index = 1 # mat_gold_rune

    # Floating central alchemical crucible sphere
    bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=12, radius=0.45, location=(0.0, 0.0, 1.45))
    crucible = bpy.context.active_object
    crucible.name = "Shrine_Crucible"
    crucible.data.materials.append(mat_gold_rune)

    finalize_mesh(bm, obj, mesh, [mat_stone, mat_gold_rune])

build_alchemical_shrine()

# -----------------------------------------------------------------------------
# 5. Bioluminescent Crystal Spire Formation (Macro Geode Feature)
# -----------------------------------------------------------------------------
def build_crystal_spire():
    mesh = bpy.data.meshes.new("Macro_Crystal_Cluster_Mesh")
    obj = bpy.data.objects.new("Macro_Crystal_Cluster", mesh)
    bpy.context.collection.objects.link(obj)
    bm = bmesh.new()

    spire_configs = [
        (0.0, 0.0, 0.0, 2.8, 0.38, 0.0, 0.0),
        (-0.45, 0.35, 0.0, 2.2, 0.28, -0.15, 0.12),
        (0.50, -0.25, 0.0, 1.9, 0.25, 0.18, -0.10),
        (-0.25, -0.45, 0.0, 1.6, 0.22, -0.10, -0.15),
        (0.35, 0.40, 0.0, 1.4, 0.20, 0.12, 0.16),
    ]

    for sx, sy, sz, h, r, lean_x, lean_y in spire_configs:
        hex_sides = 6
        h_body = h * 0.78
        tip_z = h

        # Hexagonal prism body
        base_verts = []
        mid_verts = []
        for i in range(hex_sides):
            angle = (i / hex_sides) * math.pi * 2.0
            bx = sx + math.cos(angle) * r
            by = sy + math.sin(angle) * r
            mx = sx + math.cos(angle) * r * 0.85 + lean_x * h_body
            my = sy + math.sin(angle) * r * 0.85 + lean_y * h_body
            base_verts.append(bm.verts.new((bx, by, sz)))
            mid_verts.append(bm.verts.new((mx, my, sz + h_body)))

        tip_vert = bm.verts.new((sx + lean_x * tip_z, sy + lean_y * tip_z, sz + tip_z))

        for i in range(hex_sides):
            i_next = (i + 1) % hex_sides
            # Body quad
            f_body = bm.faces.new((base_verts[i], base_verts[i_next], mid_verts[i_next], mid_verts[i]))
            f_body.material_index = 0
            # Tip pyramid triangle
            f_tip = bm.faces.new((mid_verts[i], mid_verts[i_next], tip_vert))
            f_tip.material_index = 0

    finalize_mesh(bm, obj, mesh, [mat_cyan_crystal])

build_crystal_spire()

# -----------------------------------------------------------------------------
# 6. Wrought-Iron Tripod Brazier with Glowing Embers (Clutter Prop)
# -----------------------------------------------------------------------------
def build_brazier_prop():
    mesh = bpy.data.meshes.new("Prop_Rune_Brazier_Mesh")
    obj = bpy.data.objects.new("Prop_Rune_Brazier", mesh)
    bpy.context.collection.objects.link(obj)
    bm = bmesh.new()

    # 3 Arched iron legs
    legs = 3
    h_brazier = 1.15
    for l in range(legs):
        angle = (l / legs) * math.pi * 2.0
        r_bottom = 0.55
        r_top = 0.35
        # 4 segments per leg
        segs = 4
        leg_verts = []
        for s in range(segs + 1):
            t = s / segs
            z = t * h_brazier
            r = r_bottom + (r_top - r_bottom) * t + math.sin(t * math.pi) * 0.12
            x = math.cos(angle) * r
            y = math.sin(angle) * r
            leg_verts.append(bm.verts.new((x, y, z)))

        for s in range(segs):
            # Thick wire rod
            v1 = leg_verts[s]
            v2 = leg_verts[s+1]
            # Create small square cross-section
            p_x = -math.sin(angle) * 0.03
            p_y = math.cos(angle) * 0.03
            v1_r = bm.verts.new((v1.co.x + p_x, v1.co.y + p_y, v1.co.z))
            v2_r = bm.verts.new((v2.co.x + p_x, v2.co.y + p_y, v2.co.z))
            f = bm.faces.new((v1, v2, v2_r, v1_r))
            f.material_index = 0

    # Iron Fire Bowl
    bowl_radials = 16
    bowl_r = 0.42
    bowl_z = h_brazier
    bowl_depth = 0.18

    rim_verts = [bm.verts.new((math.cos(i * math.pi * 2 / bowl_radials) * bowl_r, math.sin(i * math.pi * 2 / bowl_radials) * bowl_r, bowl_z)) for i in range(bowl_radials)]
    bowl_center = bm.verts.new((0.0, 0.0, bowl_z - bowl_depth))

    for i in range(bowl_radials):
        i_next = (i + 1) % bowl_radials
        f = bm.faces.new((rim_verts[i], rim_verts[i_next], bowl_center))
        f.material_index = 0

    # Glowing Embers inside bowl
    ember_center = bm.verts.new((0.0, 0.0, bowl_z - 0.02))
    ember_rim = [bm.verts.new((math.cos(i * math.pi * 2 / bowl_radials) * (bowl_r * 0.85), math.sin(i * math.pi * 2 / bowl_radials) * (bowl_r * 0.85), bowl_z - 0.04)) for i in range(bowl_radials)]
    for i in range(bowl_radials):
        i_next = (i + 1) % bowl_radials
        f = bm.faces.new((ember_rim[i_next], ember_rim[i], ember_center))
        f.material_index = 1 # mat_ember

    finalize_mesh(bm, obj, mesh, [mat_iron, mat_ember])

build_brazier_prop()

# -----------------------------------------------------------------------------
# 7. EXPORT COMPLETE MODULAR DUNGEON GLB
# -----------------------------------------------------------------------------
output_path = os.path.abspath("assets/models/dungeon_kit.glb")
os.makedirs(os.path.dirname(output_path), exist_ok=True)

bpy.ops.export_scene.gltf(
    filepath=output_path,
    export_format='GLB',
    use_selection=False,
    export_apply=True,
    export_materials='EXPORT',
)

print(f">>> [BLENDER 4.5] Successfully exported Masterwork Dungeon Kit to: {output_path}")
