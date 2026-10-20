import bpy
import bmesh
import math
import random
import os
from mathutils import Vector, Matrix, Euler

for obj in list(bpy.data.objects):
    bpy.data.objects.remove(obj, do_unlink=True)
for mesh in list(bpy.data.meshes):
    bpy.data.meshes.remove(mesh)
for mat in list(bpy.data.materials):
    bpy.data.materials.remove(mat)

def create_pbr_material(name, base_color, roughness=0.8, metallic=0.0, emission=None, emission_strength=1.0, alpha=1.0, use_vertex_color=False):
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    bsdf = next((n for n in nodes if n.type == 'BSDF_PRINCIPLED'), None)
    if bsdf:
        if "Base Color" in bsdf.inputs:
            bsdf.inputs["Base Color"].default_value = (*base_color[:3], 1.0)
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
                bsdf.inputs["Emission Strength"].default_value = emission_strength
        if use_vertex_color:
            attr_node = nodes.new(type="ShaderNodeAttribute")
            attr_node.attribute_name = "Color"
            mat.node_tree.links.new(attr_node.outputs["Color"], bsdf.inputs["Base Color"])
    return mat

mat_rock_shadow = create_pbr_material("M_Rock_Shadow", (0.11, 0.15, 0.19), roughness=0.90)
mat_rock_lit = create_pbr_material("M_Rock_Lit", (0.22, 0.30, 0.35), roughness=0.80)
mat_cavern_moss = create_pbr_material("M_Cavern_Moss", (0.10, 0.46, 0.16), roughness=0.86)
mat_cavern_vine = create_pbr_material("M_Cavern_Vine", (0.16, 0.50, 0.22), roughness=0.75)
mat_ground_path = create_pbr_material("M_Ground_Path", (0.18, 0.10, 0.04), roughness=0.95)
mat_ground_grass = create_pbr_material("M_Ground_Grass", (0.08, 0.28, 0.13), roughness=0.90)
mat_fine_rim = create_pbr_material("M_Fine_Stone_Rim", (0.22, 0.24, 0.26), roughness=0.75)
mat_river_pebble = create_pbr_material("M_River_Pebble", (0.32, 0.34, 0.36), roughness=0.65)
mat_angular_rock = create_pbr_material("M_Angular_Rock", (0.20, 0.23, 0.26), roughness=0.88)

mat_wet_submerged_rock = create_pbr_material("M_Wet_Submerged_Rock", (0.04, 0.08, 0.12), roughness=0.22, metallic=0.04)
mat_submerged_crater = create_pbr_material("M_Submerged_Crater", (0.02, 0.05, 0.08), roughness=0.30)
mat_leaf_autumn = create_pbr_material("M_Leaf_Autumn", (0.52, 0.30, 0.08), roughness=0.72)
mat_leaf_olive = create_pbr_material("M_Leaf_Olive", (0.24, 0.38, 0.14), roughness=0.70)
mat_dirt_patch = create_pbr_material("M_Dirt_Patch", (0.13, 0.08, 0.03), roughness=0.92)
mat_terrain_splat = create_pbr_material("M_Terrain_Splat", (1.0, 1.0, 1.0), roughness=0.88, use_vertex_color=True)

mat_rock_highlight = create_pbr_material("M_Rock_Highlight", (0.28, 0.38, 0.45), roughness=0.75)
mat_stone_slab = create_pbr_material("M_Stone_Slab", (0.22, 0.26, 0.30), roughness=0.82)

mat_shroom_pale_cap = create_pbr_material("M_Shroom_Pale_Cap", (0.05, 0.88, 0.86), roughness=0.20, emission=(0.06, 0.95, 0.92), emission_strength=2.6)
mat_shroom_dots = create_pbr_material("M_Shroom_Dots", (0.85, 0.98, 1.0), roughness=0.20, emission=(0.80, 0.98, 1.0), emission_strength=2.2)
mat_shroom_gills = create_pbr_material("M_Shroom_Gills", (0.05, 0.72, 0.78), roughness=0.30, emission=(0.08, 0.84, 0.88), emission_strength=2.2)
mat_shroom_stem = create_pbr_material("M_Shroom_Stem", (0.72, 0.84, 0.80), roughness=0.65, emission=(0.06, 0.22, 0.22), emission_strength=0.6)
mat_broad_leaf = create_pbr_material("M_Broad_Leaf", (0.14, 0.48, 0.20), roughness=0.65)
mat_glowing_tip = create_pbr_material("M_Glowing_Tip", (0.20, 0.95, 0.92), roughness=0.20, emission=(0.20, 0.98, 0.95), emission_strength=3.0)

mat_shroom_vibrant_cap = create_pbr_material("M_Shroom_Vibrant_Cap", (0.04, 0.88, 0.90), roughness=0.18, emission=(0.06, 0.96, 0.94), emission_strength=2.6)
mat_tall_grass = create_pbr_material("M_Tall_Grass", (0.16, 0.50, 0.22), roughness=0.72)
mat_glow_bulb = create_pbr_material("M_Glow_Bulb", (0.15, 0.95, 0.90), roughness=0.15, emission=(0.15, 0.98, 0.92), emission_strength=3.0)
mat_dandelion_stem = create_pbr_material("M_Dandelion_Stem", (0.20, 0.52, 0.16), roughness=0.75)
mat_dandelion_leaf = create_pbr_material("M_Dandelion_Leaf", (0.18, 0.48, 0.14), roughness=0.72)
mat_dandelion_flower = create_pbr_material("M_Dandelion_Flower", (1.0, 0.85, 0.12), roughness=0.62, emission=(0.96, 0.76, 0.12), emission_strength=0.82)
mat_dandelion_seed = create_pbr_material("M_Dandelion_Seed", (0.95, 0.98, 1.0), roughness=0.75, emission=(0.35, 0.48, 0.55), emission_strength=0.35)
mat_moss_wet = create_pbr_material("M_Wet_Moss", (0.06, 0.30, 0.12), roughness=0.92)
mat_moss_lime = create_pbr_material("M_Lime_Moss", (0.22, 0.62, 0.18), roughness=0.84)
mat_moss_fringes = create_pbr_material("M_Moss_Fringes", (0.11, 0.40, 0.16), roughness=0.88)

mat_water_foam_billow = create_pbr_material("M_Water_Foam_Billow", (0.92, 0.98, 1.0), roughness=0.20, emission=(0.60, 0.90, 1.0), emission_strength=1.5)
mat_water_ripple_ring = create_pbr_material("M_Water_Ripple_Ring", (0.40, 0.92, 0.98), roughness=0.15, emission=(0.35, 0.90, 0.98), emission_strength=1.8, alpha=0.85)

mat_ceiling_lichen = create_pbr_material("M_Ceiling_Lichen", (0.20, 0.92, 0.88), roughness=0.15, emission=(0.18, 0.95, 0.90), emission_strength=2.8)

def continuous_noise_3d(x, y, z):

    n1 = math.sin(x * 1.35 + y * 1.82 + z * 0.91)
    n2 = math.cos(x * 2.71 - y * 1.43 + z * 2.15)
    n3 = math.sin(x * 4.95 + y * 3.78 - z * 3.12)
    n4 = math.cos(x * 9.15 - y * 8.21 + z * 7.43)
    return (n1 * 0.48 + n2 * 0.28 + n3 * 0.16 + n4 * 0.08)

def get_terrain_surface_z(x, y):
    lake_center_x = 0.0
    lake_center_y = 8.5
    lake_radius = 5.2
    dist_lake = math.sqrt((x - lake_center_x)**2 + (y - lake_center_y)**2)

    if dist_lake < lake_radius:
        u = dist_lake / lake_radius
        funnel_depth = math.pow(max(0.0, 1.0 - u), 1.75) * 3.10 + (1.0 - u) * 0.35
        wall_noise = continuous_noise_3d(x * 0.55, y * 0.55, -funnel_depth * 0.4) * 0.14
        return 0.04 - funnel_depth + wall_noise
    elif dist_lake < lake_radius + 1.8:
        rim_blend = (dist_lake - lake_radius) / 1.8
        return 0.04 + rim_blend * 0.03
    elif y < 5.2 and abs(x) < 2.4:
        path_dent = (1.0 - min(1.0, abs(x) / 2.4)) * 0.015
        return 0.04 - path_dent
    else:
        dist_side = max(0.0, abs(x) - 2.4)
        bank_h = min(1.85, dist_side * 0.35)
        wave_relief = (
            math.sin(x * 0.45 + y * 0.35) * 0.08 +
            math.cos(x * 0.72) * 0.04 +
            continuous_noise_3d(x * 0.4, y * 0.4, 0.0) * 0.06
        )
        return 0.05 + bank_h + wave_relief

def get_cavern_ceiling_z(x, y):

    wall_dist = 9.2 + max(0.0, y - 2.0) * 0.35
    sin_ang = max(-0.99, min(0.99, x / max(0.01, wall_dist)))
    angle = math.asin(sin_ang)
    is_over_waterfall = (abs(angle) < 0.35 and y > 7.0)

    if y >= 12.5:

        curr_z = 18.2 - abs(math.sin(angle)) * 4.0
    elif is_over_waterfall:

        y_blend = min(1.0, max(0.0, (y - 7.0) / 4.8))
        peak_z = 18.5 - (abs(angle) / 0.35) * 1.5
        curr_z = 10.5 + y_blend * (peak_z - 10.5)
    else:

        arch_dip = math.cos(angle * 1.15) * 1.8
        base_z = 6.2 + arch_dip + max(0.0, y) * 0.28
        curr_z = max(4.5, base_z)

    n_disp = continuous_noise_3d(angle * 3.5, y * 0.45, curr_z * 0.35) * 0.38
    return curr_z + n_disp

def build_visible_cavern_roof_and_arch():

    bm = bmesh.new()
    segments_u = 48
    segments_v = 36

    grid = []
    for iv in range(segments_v + 1):
        v_frac = iv / segments_v
        curr_y = -4.5 + v_frac * 18.0

        row = []
        for iu in range(segments_u + 1):
            u_frac = iu / segments_u
            angle = -math.pi * 0.48 + u_frac * math.pi * 0.96
            wall_dist = 9.2 + max(0.0, curr_y - 2.0) * 0.35
            curr_x = math.sin(angle) * wall_dist
            base_z = get_cavern_ceiling_z(curr_x, curr_y)

            facet_snap = math.floor((curr_x * 0.45 + curr_y * 0.35) * 2.0) * 0.18
            n_disp = continuous_noise_3d(curr_x * 0.35, curr_y * 0.35, base_z * 0.25) * 0.32
            curr_z = base_z + facet_snap + n_disp

            v = bm.verts.new((curr_x, curr_y, curr_z))
            row.append(v)
        grid.append(row)

    bm.verts.ensure_lookup_table()
    for iv in range(segments_v):
        for iu in range(segments_u):
            v1 = grid[iv][iu]
            v2 = grid[iv][iu + 1]
            v3 = grid[iv + 1][iu + 1]
            v4 = grid[iv + 1][iu]
            f = bm.faces.new((v1, v2, v3, v4))

            center = f.calc_center_median()
            norm = f.normal
            if center.z > 14.0 and abs(center.x) < 3.8:
                f.material_index = 1
            elif norm.z > 0.50 and center.z < 8.0:
                f.material_index = 2
            else:
                f.material_index = 0

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new("Cavern_Visible_Arch_Mesh")
    obj = bpy.data.objects.new("Cavern_Visible_Arch", mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()

    for p in mesh.polygons:
        p.use_smooth = False
    mesh.update()

    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)
    return obj

build_visible_cavern_roof_and_arch()

def spawn_organic_stalactite(x, y, z_top, height=2.8, radius=0.42, tilt=(0.0, 0.0), name="OrganicStalactite"):

    bm = bmesh.new()
    rings = 14
    radials = 7

    ring_verts = []
    for ir in range(rings + 1):
        t = ir / rings

        step_ring = math.floor(t * 3.5) / 3.5
        stepped_drop = 1.0 - (t - step_ring) * 0.22
        flare = ((1.0 - t) ** 3.0) * 0.55 * radius
        exp_taper = ((1.0 - t) ** 1.30)
        curr_r = max(0.016, (radius * exp_taper * stepped_drop) + flare)

        curr_x = tilt[0] * (t ** 1.25) + math.sin(t * 5.2) * 0.025
        curr_y = tilt[1] * (t ** 1.25) + math.cos(t * 5.2) * 0.025
        curr_z = -t * height

        row = []
        for ia in range(radials):
            ang = 2.0 * math.pi * ia / radials

            r_facet = curr_r * (1.0 + 0.12 * math.cos(ang * 3.0) + 0.05 * math.sin(t * 8.0))
            vx = curr_x + math.cos(ang) * r_facet
            vy = curr_y + math.sin(ang) * r_facet
            vz = curr_z
            vert = bm.verts.new((vx, vy, vz))
            row.append(vert)
        ring_verts.append(row)

    bm.verts.ensure_lookup_table()

    for ir in range(rings):
        t = (ir + 0.5) / rings
        for ia in range(radials):
            ia_next = (ia + 1) % radials
            v1 = ring_verts[ir][ia]
            v2 = ring_verts[ir][ia_next]
            v3 = ring_verts[ir + 1][ia_next]
            v4 = ring_verts[ir + 1][ia]
            face = bm.faces.new((v1, v2, v3, v4))

            if t < 0.16:
                face.material_index = 2
            elif math.cos(ia * 2.0 * math.pi / radials) > 0.10:
                face.material_index = 1
            else:
                face.material_index = 0

    tip_center = bm.verts.new((tilt[0] + 0.008, tilt[1], -height))
    for ia in range(radials):
        ia_next = (ia + 1) % radials
        face = bm.faces.new((ring_verts[rings][ia], ring_verts[rings][ia_next], tip_center))
        face.material_index = 0

    top_center = bm.verts.new((0.0, 0.0, 0.0))
    for ia in range(radials):
        ia_next = (ia + 1) % radials
        face = bm.faces.new((ring_verts[0][ia_next], ring_verts[0][ia], top_center))
        face.material_index = 2

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    trans_mat = Matrix.Translation(Vector((x, y, z_top)))
    bm.transform(trans_mat)

    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()

    for poly in mesh.polygons:
        poly.use_smooth = False
    mesh.update()

    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)
    return obj

visible_stalactite_targets = [

    (-6.2, 4.2, 2.65, 0.74, (0.09, -0.07)),
    (-5.4, 3.2, 3.85, 0.46, (0.05, 0.04)),
    (-6.6, 5.1, 5.60, 0.16, (0.02, -0.02)),
    (-5.8, 2.1, 4.40, 0.38, (0.04, -0.03)),
    (-4.9, 4.6, 4.90, 0.28, (-0.03, 0.05)),
    (-4.4, 6.4, 3.30, 0.58, (-0.06, 0.08)),
    (-4.1, 7.1, 5.75, 0.18, (0.03, -0.02)),
    (-5.7, 0.6, 4.10, 0.44, (0.06, -0.04)),
    (-6.3, 1.4, 5.40, 0.20, (-0.02, 0.03)),
    (-5.1, -1.0, 3.50, 0.52, (0.05, 0.03)),
    (-4.4, -1.6, 5.25, 0.17, (-0.03, 0.02)),

    (6.8, 2.2, 2.80, 0.70, (-0.08, 0.06)),
    (5.9, 3.5, 4.20, 0.42, (-0.04, -0.05)),
    (6.4, 4.1, 5.80, 0.15, (0.02, 0.01)),
    (5.3, 4.9, 3.60, 0.50, (0.04, -0.07)),
    (4.7, 5.8, 5.50, 0.22, (-0.02, 0.04)),
    (4.2, 7.4, 3.40, 0.55, (-0.07, 0.06)),
    (3.8, 8.2, 5.65, 0.18, (0.01, -0.03)),
    (5.8, 0.9, 4.50, 0.36, (-0.04, 0.03)),
    (6.5, 0.2, 5.35, 0.20, (0.03, -0.02)),
    (5.4, -0.9, 3.35, 0.56, (-0.05, 0.04)),
    (4.6, -1.5, 5.15, 0.19, (0.02, -0.03)),
]

for idx, (sx, sy, target_tip_z, sr, stilt) in enumerate(visible_stalactite_targets):
    ceiling_z = get_cavern_ceiling_z(sx, sy)
    z_top = ceiling_z + 0.50
    sh = max(1.8, z_top - target_tip_z)
    spawn_organic_stalactite(sx, sy, z_top, height=sh, radius=sr, tilt=stilt, name=f"VisibleStalactite_{idx}")

def build_cyclopean_portal_arch():

    for side_sign, p_name in [(-1.0, "Portal_Pillar_Left"), (1.0, "Portal_Pillar_Right")]:
        cx = side_sign * 2.85
        cy = -4.35

        num_blocks = 4
        for ib in range(num_blocks):
            bz = 0.65 + ib * 1.25
            rad = 1.15 - ib * 0.10
            scale_xyz = (1.20, 1.15, 0.75)

            bm = bmesh.new()
            bmesh.ops.create_icosphere(bm, subdivisions=2, radius=rad)

            for v in bm.verts:
                p = v.co
                dist = p.length
                if dist > 0.0001:
                    norm = p.normalized()
                    n = continuous_noise_3d(norm.x * 2.5 + ib * 13, norm.y * 2.5, norm.z * 2.5)

                    facet_snap = 1.0 + (math.floor(n * 3.5) / 3.5) * 0.30
                    bottom_flat = 0.70 if p.z < -0.1 else 1.0
                    v.co = norm * (dist * facet_snap * bottom_flat)
                    v.co.x *= scale_xyz[0]
                    v.co.y *= scale_xyz[1]
                    v.co.z *= scale_xyz[2]

            trans_mat = Matrix.Translation((cx, cy, bz))
            bm.transform(trans_mat)

            for f in bm.faces:
                if f.normal.z > 0.44:
                    f.material_index = 2
                elif f.normal.y > 0.15:
                    f.material_index = 1
                else:
                    f.material_index = 0

            bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
            b_mesh = bpy.data.meshes.new(f"{p_name}_Block_{ib}_Mesh")
            b_obj = bpy.data.objects.new(f"{p_name}_Block_{ib}", b_mesh)
            bpy.context.collection.objects.link(b_obj)
            bm.to_mesh(b_mesh)
            bm.free()
            for p in b_mesh.polygons:
                p.use_smooth = False
            b_mesh.update()
            b_obj.data.materials.append(mat_rock_shadow)
            b_obj.data.materials.append(mat_rock_lit)
            b_obj.data.materials.append(mat_cavern_moss)

        for isp in range(3):
            ang = isp * 1.8 + (0.5 if side_sign > 0 else -0.5)
            sp_x = cx + math.cos(ang) * 0.95
            sp_y = cy + math.sin(ang) * 0.65
            sp_z = 0.8 + isp * 1.4
            spawn_wall_cyan_spores(sp_x, sp_y, sp_z, num_shrooms=4, name=f"{p_name}_Spores_{isp}")

    span_blocks = 5
    for ispan in range(span_blocks):
        t_span = (ispan - 2) / 2.0
        bx_center = t_span * 1.45
        by_center = -4.35
        bz_center = 4.25 + (1.0 - abs(t_span)) * 0.45

        bm_b = bmesh.new()
        bmesh.ops.create_icosphere(bm_b, subdivisions=2, radius=1.0)

        sx = 0.95
        sy = 1.15
        sz = 0.85

        for v in bm_b.verts:
            p = v.co
            dist = p.length
            if dist > 0.0001:
                norm = p.normalized()
                n = continuous_noise_3d(norm.x * 2.4 + ispan * 9, norm.y * 2.4, norm.z * 2.4)
                facet_snap = 1.0 + (math.floor(n * 3.5) / 3.5) * 0.28

                tooth = 0.35 if (p.z < -0.2 and abs(p.y) < 0.3) else 0.0
                v.co = norm * (dist * facet_snap)
                v.co.x *= sx
                v.co.y *= sy
                v.co.z = (v.co.z - tooth) * sz

        trans_mat = Matrix.Translation((bx_center, by_center, bz_center))
        bm_b.transform(trans_mat)

        for f in bm_b.faces:
            if f.normal.z > 0.42:
                f.material_index = 2
            elif f.normal.y > 0.15:
                f.material_index = 1
            else:
                f.material_index = 0

        bmesh.ops.recalc_face_normals(bm_b, faces=bm_b.faces)
        b_mesh = bpy.data.meshes.new(f"Portal_Lintel_Block_{ispan}_Mesh")
        b_obj = bpy.data.objects.new(f"Portal_Lintel_Block_{ispan}", b_mesh)
        bpy.context.collection.objects.link(b_obj)
        bm_b.to_mesh(b_mesh)
        bm_b.free()
        for p in b_mesh.polygons:
            p.use_smooth = False
        b_mesh.update()
        b_obj.data.materials.append(mat_rock_shadow)
        b_obj.data.materials.append(mat_rock_lit)
        b_obj.data.materials.append(mat_cavern_moss)

    for vx, v_len in [(-1.2, 1.4), (-0.4, 2.1), (0.5, 1.8), (1.3, 1.2)]:
        spawn_hanging_vine(vx, -4.1, 3.4, length=v_len, segments=10, name=f"Portal_Vine_{vx}")

def build_continuous_u_canyon_grotto_walls():

    bm = bmesh.new()
    u_segs = 64
    v_segs = 28

    grid = []
    for iu in range(u_segs + 1):
        u_frac = iu / u_segs

        if u_frac < 0.22:

            t = u_frac / 0.22
            y0 = -4.5 + t * 10.0
            meander = math.sin(t * math.pi * 2.0) * 0.35 + continuous_noise_3d(-8.5, y0 * 0.4, 0.0) * 0.25
            x0 = -8.5 + meander
            inward_nx = 1.0
            inward_ny = 0.0
        elif u_frac < 0.55:

            t = (u_frac - 0.22) / 0.33
            theta_0 = 1.92
            ang = -theta_0 + t * (2.0 * theta_0)
            t_ang = abs(ang) / theta_0
            r_apex = 5.0
            r_base = 9.1
            dist_base = r_apex + (r_base - r_apex) * (t_ang ** 1.6)
            x0 = 0.0 + math.sin(ang) * dist_base
            y0 = 8.5 + math.cos(ang) * dist_base
            inward_nx = -math.sin(ang)
            inward_ny = -math.cos(ang)
        elif u_frac < 0.77:

            t = (u_frac - 0.55) / 0.22
            y0 = 5.5 - t * 10.0
            meander = math.sin(t * math.pi * 2.0) * 0.35 + continuous_noise_3d(8.5, y0 * 0.4, 0.0) * 0.25
            x0 = 8.5 - meander
            inward_nx = -1.0
            inward_ny = 0.0
        else:

            t = (u_frac - 0.77) / 0.23
            x0 = 8.5 - t * 17.0
            meander = math.sin(t * math.pi) * 0.4
            y0 = -4.5 - meander
            inward_nx = 0.0
            inward_ny = 1.0

        z_ground = get_terrain_surface_z(x0, y0) - 0.30
        z_ceil = get_cavern_ceiling_z(x0, y0) + 0.20

        if u_frac >= 0.77 and abs(x0) < 2.5:
            rel_x = x0 / 2.5
            arch_shape = math.sqrt(max(0.0, 1.0 - rel_x * rel_x))
            tooth_jitter = math.sin(x0 * 5.5) * 0.25
            h_arch = max(0.0, 3.4 * arch_shape + tooth_jitter)
            z_base = z_ground + h_arch
            h_total = max(1.2, z_ceil - z_base)
            portal_atten = abs(x0) / 2.5
        else:
            z_base = z_ground
            h_total = max(3.5, z_ceil - z_base)
            portal_atten = 1.0

        col = []
        for iv in range(v_segs + 1):
            v_frac = iv / v_segs
            curr_z = z_base + v_frac * h_total

            tier_phase = (curr_z % 2.2) / 2.2
            if tier_phase < 0.28:
                shelf_disp = 0.65 * (1.0 - (tier_phase / 0.28))
            else:
                shelf_disp = -0.12 * math.sin((tier_phase - 0.28) / 0.72 * math.pi)

            col_id = int(iu / 2)
            col_pillar = math.sin(col_id * 1.85) * 0.42 + math.cos(col_id * 3.4) * 0.28

            facet_q = math.floor(curr_z * 0.6) * 0.4
            facet_noise = continuous_noise_3d(x0 * 0.5, y0 * 0.5, facet_q) * 0.35

            if 0.22 <= u_frac <= 0.55:
                t_ang_sub = abs(-1.92 + ((u_frac - 0.22) / 0.33) * 3.84) / 1.92
                apex_atten = 0.40 + 0.60 * (t_ang_sub ** 2.0)
            else:
                apex_atten = 1.0

            d_total = (shelf_disp + col_pillar + facet_noise) * apex_atten * portal_atten

            vx = x0 + inward_nx * d_total
            vy = y0 + inward_ny * d_total
            vz = curr_z
            vert = bm.verts.new((vx, vy, vz))
            col.append(vert)
        grid.append(col)

    bm.verts.ensure_lookup_table()
    for iu in range(u_segs):
        for iv in range(v_segs):
            v1 = grid[iu][iv]
            v2 = grid[iu + 1][iv]
            v3 = grid[iu + 1][iv + 1]
            v4 = grid[iu][iv + 1]
            f = bm.faces.new((v1, v2, v3, v4))

            norm = f.normal
            center = f.calc_center_median()
            u_frac = iu / float(u_segs)

            is_waterfall_backdrop = 0.22 <= u_frac <= 0.55 and center.y > 6.0 and center.z < 16.5
            moss_noise = continuous_noise_3d(center.x * 0.45, center.y * 0.45, center.z * 0.45)

            if is_waterfall_backdrop and (norm.z > 0.14 or moss_noise > -0.25 or (center.z < 11.0 and abs(center.x) < 4.8)):
                f.material_index = 2
            elif norm.z > 0.35 and center.z < 12.0:
                f.material_index = 2
            elif norm.y < -0.2 and center.z > 6.0:
                f.material_index = 1
            else:
                f.material_index = 0

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new("Continuous_U_Canyon_Walls_Mesh")
    obj = bpy.data.objects.new("Continuous_U_Canyon_Walls", mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()

    for p in mesh.polygons:
        p.use_smooth = False
    mesh.update()

    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)

    return obj

build_continuous_u_canyon_grotto_walls()
def build_waterfall_flank_cliffs():

    for side_sign, tower_name in [(-1.0, "Waterfall_Cliff_Tower_Left"), (1.0, "Waterfall_Cliff_Tower_Right")]:
        cx = side_sign * 4.65
        cy = 10.60

        tiers_spec = [
            (1.5, 3.2, 3.8, 3.2),
            (4.8, 3.6, 3.4, 2.8),
            (8.5, 4.0, 3.2, 2.6),
            (12.6, 4.2, 3.0, 2.4),
            (16.5, 3.8, 2.8, 2.2),
        ]

        for itier, (tz, th, tw, td) in enumerate(tiers_spec):
            bm = bmesh.new()
            bmesh.ops.create_icosphere(bm, subdivisions=2, radius=1.0)

            for v in bm.verts:
                p = v.co
                dist = p.length
                if dist > 0.0001:
                    norm = p.normalized()
                    n = continuous_noise_3d(norm.x * 2.2 + itier * 7, norm.y * 2.2, norm.z * 2.2)
                    facet_snap = 1.0 + (math.floor(n * 3.0) / 3.0) * 0.28
                    bottom_flat = 0.75 if p.z < -0.1 else 1.0
                    v.co = norm * (dist * facet_snap * bottom_flat)
                    v.co.x *= (tw * 0.5)
                    v.co.y *= (td * 0.5)
                    v.co.z *= (th * 0.5)

            trans_mat = Matrix.Translation((cx, cy, tz))
            bm.transform(trans_mat)

            for f in bm.faces:
                f_norm = f.normal
                f_center = f.calc_center_median()
                if f_norm.z > 0.22 or (abs(f_center.x) < 5.2 and f_norm.y < 0.2 and f_norm.z > 0.05):
                    f.material_index = 2
                elif abs(f_norm.x) > 0.2 and f_norm.y < 0.0:
                    f.material_index = 1
                else:
                    f.material_index = 0

            bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
            t_mesh = bpy.data.meshes.new(f"{tower_name}_Tier_{itier}_Mesh")
            t_obj = bpy.data.objects.new(f"{tower_name}_Tier_{itier}", t_mesh)
            bpy.context.collection.objects.link(t_obj)
            bm.to_mesh(t_mesh)
            bm.free()
            for p in t_mesh.polygons:
                p.use_smooth = False
            t_mesh.update()
            t_obj.data.materials.append(mat_rock_shadow)
            t_obj.data.materials.append(mat_rock_lit)
            t_obj.data.materials.append(mat_cavern_moss)

        for ivine in range(3):
            vx = cx + (ivine - 1) * 0.85
            vy = cy - 1.25
            vz = 6.5 + ivine * 2.8
            spawn_hanging_vine(vx, vy, vz, length=2.2, segments=12, name=f"{tower_name}_Vine_{ivine}")

def spawn_organic_boulder(name, center_x, center_y, center_z, scale=(1.0, 1.0, 1.0), rot_z=0.0, seed=42):

    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=2, radius=1.0)

    rng = random.Random(seed)

    for v in bm.verts:
        p = v.co
        dist = p.length
        if dist > 0.0001:
            norm = p.normalized()
            n = continuous_noise_3d(norm.x * 2.2 + seed, norm.y * 2.2 + seed, norm.z * 2.2 + seed)
            bottom_flatten = 0.72 if p.z < -0.15 else 1.0
            facet_snap = 1.0 + (math.floor(n * 3.5) / 3.5) * 0.28 + rng.uniform(-0.04, 0.04)
            v.co = norm * (dist * facet_snap * bottom_flatten)
            v.co.x *= scale[0]
            v.co.y *= scale[1]
            v.co.z *= scale[2]

    rot_mat = Matrix.Rotation(rot_z, 4, 'Z')
    bm.transform(rot_mat)
    trans_mat = Matrix.Translation((center_x, center_y, center_z))
    bm.transform(trans_mat)

    for f in bm.faces:
        if f.normal.z > 0.48:
            f.material_index = 2
        elif f.normal.y < -0.2:
            f.material_index = 1
        else:
            f.material_index = 0

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()

    for p in mesh.polygons:
        p.use_smooth = False
    mesh.update()

    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)
    return obj

organic_terraces = [

    ("Left_Terrace_B0", -7.0, -2.0, 0.40, (1.5, 1.3, 0.75), 0.3, 110),
    ("Left_Terrace_B1", -6.8, -0.5, 0.42, (1.5, 1.3, 0.80), -0.2, 111),
    ("Left_Terrace_B2", -6.6, 1.0, 0.45, (1.6, 1.4, 0.85), 0.4, 112),
    ("Left_Terrace_B3", -6.5, 2.6, 0.48, (1.5, 1.3, 0.85), -0.3, 113),
    ("Left_Terrace_B4", -6.6, 4.4, 0.52, (1.6, 1.4, 0.90), 0.2, 114),
    ("Left_Terrace_1", -6.5, 6.8, 0.55, (1.6, 1.4, 0.90), 0.2, 101),
    ("Left_Terrace_Low", -5.8, 5.8, 0.40, (1.2, 1.1, 0.65), 0.5, 105),

    ("Left_Terrace_M0", -7.5, -1.2, 1.40, (1.4, 1.2, 0.95), -0.4, 120),
    ("Left_Terrace_M1", -7.3, 0.6, 1.55, (1.5, 1.2, 1.00), 0.3, 121),
    ("Left_Terrace_M2", -7.2, 2.5, 1.80, (1.5, 1.2, 1.05), -0.2, 122),
    ("Left_Terrace_M3", -7.2, 4.4, 2.00, (1.5, 1.2, 1.10), 0.4, 123),
    ("Left_Terrace_2", -6.8, 7.8, 1.40, (1.5, 1.3, 1.10), -0.3, 102),
    ("Left_Terrace_3", -7.0, 8.8, 2.45, (1.4, 1.2, 1.20), 0.4, 103),

    ("Left_Terrace_U1", -7.8, 2.0, 3.20, (1.4, 1.2, 1.15), -0.3, 131),
    ("Left_Terrace_U2", -7.6, 4.0, 3.60, (1.4, 1.1, 1.20), 0.3, 132),
    ("Left_Terrace_4", -7.4, 9.8, 3.65, (1.3, 1.1, 1.30), -0.2, 104),

    ("Right_Terrace_B0", 7.0, -2.0, 0.40, (1.5, 1.3, 0.75), -0.3, 210),
    ("Right_Terrace_B1", 6.8, -0.5, 0.42, (1.5, 1.3, 0.80), 0.2, 211),
    ("Right_Terrace_B2", 6.6, 1.0, 0.45, (1.6, 1.4, 0.85), -0.4, 212),
    ("Right_Terrace_B3", 6.5, 2.6, 0.48, (1.5, 1.3, 0.85), 0.3, 213),
    ("Right_Terrace_1", 6.5, 6.8, 0.55, (1.6, 1.4, 0.90), -0.2, 201),
    ("Right_Terrace_Low", 5.8, 5.8, 0.40, (1.2, 1.1, 0.65), -0.5, 205),

    ("Right_Terrace_M0", 7.5, -1.2, 1.40, (1.4, 1.2, 0.95), 0.4, 220),
    ("Right_Terrace_M1", 7.3, 0.6, 1.55, (1.5, 1.2, 1.00), -0.3, 221),
    ("Right_Terrace_M2", 7.2, 2.5, 1.80, (1.5, 1.2, 1.05), 0.3, 222),
    ("Right_Terrace_M3", 7.2, 4.4, 2.00, (1.5, 1.2, 1.10), -0.4, 223),
    ("Right_Terrace_2", 6.8, 7.8, 1.40, (1.5, 1.3, 1.10), 0.3, 202),
    ("Right_Terrace_3", 7.0, 8.8, 2.45, (1.4, 1.2, 1.20), -0.4, 203),

    ("Right_Terrace_U1", 7.8, 2.0, 3.20, (1.4, 1.2, 1.15), 0.3, 231),
    ("Right_Terrace_U2", 7.6, 4.0, 3.60, (1.4, 1.1, 1.20), -0.3, 232),
    ("Right_Terrace_4", 7.4, 9.8, 3.65, (1.3, 1.1, 1.30), 0.2, 204),
]

for (tname, tx, ty, tz, tscale, trot, tseed) in organic_terraces:
    spawn_organic_boulder(tname, tx, ty, tz, scale=tscale, rot_z=trot, seed=tseed)

def spawn_submerged_boulder(name, center_x, center_y, center_z, scale=(1.0, 1.0, 1.0), rot_z=0.0, seed=42):
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=2, radius=1.0)
    rng = random.Random(seed)
    for v in bm.verts:
        p = v.co
        dist = p.length
        if dist > 0.0001:
            norm = p.normalized()
            n = continuous_noise_3d(norm.x * 2.5 + seed, norm.y * 2.5 + seed, norm.z * 2.5 + seed)
            facet_snap = 1.0 + (math.floor(n * 3.5) / 3.5) * 0.28 + rng.uniform(-0.04, 0.04)
            v.co = norm * (dist * facet_snap)
            v.co.x *= scale[0]
            v.co.y *= scale[1]
            v.co.z *= scale[2]
    rot_mat = Matrix.Rotation(rot_z, 4, 'Z')
    bm.transform(rot_mat)
    trans_mat = Matrix.Translation((center_x, center_y, center_z))
    bm.transform(trans_mat)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = False
    mesh.update()
    ca = mesh.color_attributes.new(name="Color", type='FLOAT_COLOR', domain='POINT')
    for vi in range(len(mesh.vertices)):
        ca.data[vi].color = (0.04, 0.08, 0.12, 1.0)
    obj.data.materials.append(mat_wet_submerged_rock)
    return obj

submerged_lake_boulders = [
    ("Submerged_Boulder_Deep_Center", 0.0, 9.1, -2.65, (1.20, 0.85, 0.55), 0.2, 501),
    ("Submerged_Boulder_Deep_Left", -1.4, 8.9, -2.55, (0.95, 1.05, 0.50), -0.4, 502),
    ("Submerged_Boulder_Deep_Right", 1.5, 9.0, -2.60, (1.05, 0.80, 0.52), 0.3, 503),
    ("Submerged_Boulder_Slope_Left", -2.2, 8.2, -1.85, (0.85, 0.75, 0.45), 0.5, 504),
    ("Submerged_Boulder_Slope_Right", 2.1, 8.3, -1.90, (0.85, 0.75, 0.45), -0.3, 505),
    ("Submerged_Boulder_Mid_Left", -1.5, 7.2, -1.15, (0.75, 0.70, 0.38), 0.6, 510),
    ("Submerged_Boulder_Mid_Right", 1.4, 7.3, -1.18, (0.75, 0.68, 0.40), -0.5, 511),
    ("Submerged_Boulder_Mid_Center", -0.2, 7.5, -1.35, (0.80, 0.75, 0.42), 0.4, 512),
    ("Submerged_Boulder_Shelf_1", 0.3, 6.2, -0.48, (0.65, 0.60, 0.32), 0.3, 520),
    ("Submerged_Boulder_Shelf_2", -1.0, 5.8, -0.35, (0.58, 0.52, 0.28), -0.4, 521),
    ("Submerged_Boulder_Shelf_3", 1.1, 5.9, -0.38, (0.60, 0.50, 0.30), 0.5, 522),
]

for (sname, sx, sy, sz, sscale, srot, sseed) in submerged_lake_boulders:
    spawn_submerged_boulder(sname, sx, sy, sz, scale=sscale, rot_z=srot, seed=sseed)

def spawn_hanging_vine(x, y, z_top, length=1.8, segments=12, name="HangingVine"):
    bm = bmesh.new()
    width = 0.065
    rows = []
    for iseg in range(segments + 1):
        t = iseg / segments
        cz = z_top - t * length
        cx = x + math.sin(t * math.pi * 3.0) * 0.08
        cy = y + (t * 0.035)
        w = width * (1.0 - t * 0.35)
        v_l = bm.verts.new((cx - w * 0.5, cy, cz))
        v_r = bm.verts.new((cx + w * 0.5, cy, cz))
        rows.append((v_l, v_r))

        if iseg % 2 == 1:
            side = 1.0 if (iseg % 4 == 1) else -1.0
            leaf_tip = bm.verts.new((cx + side * (w + 0.15), cy - 0.02, cz - 0.03))
            bm.faces.new((v_l, v_r, leaf_tip))

    for iseg in range(segments):
        bm.faces.new((rows[iseg][0], rows[iseg][1], rows[iseg + 1][1], rows[iseg + 1][0]))

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_cavern_vine)

vine_locations = [

    (-3.6, 11.2, 7.5, 3.2),
    (3.6, 11.2, 7.5, 3.2),
    (-4.5, 10.5, 5.2, 2.4),
    (4.5, 10.5, 5.2, 2.4),
    (-6.9, 8.9, 3.4, 1.45),
    (6.9, 8.9, 3.4, 1.45),
    (-6.5, 7.9, 2.2, 1.25),
    (6.5, 7.9, 2.2, 1.25),
    (-5.8, 6.9, 1.1, 0.95),
    (5.8, 6.9, 1.1, 0.95),

    (-6.8, 4.2, 2.6, 1.5),
    (-6.7, 2.4, 2.4, 1.6),
    (-6.6, 0.6, 2.2, 1.4),
    (-6.8, -1.2, 2.0, 1.3),

    (6.8, 4.4, 2.6, 1.5),
    (6.7, 2.4, 2.4, 1.6),
    (6.6, 0.6, 2.2, 1.4),
    (6.8, -1.2, 2.0, 1.3),
]
for idx, (vx, vy, vz, vl) in enumerate(vine_locations):
    spawn_hanging_vine(vx, vy, vz, length=vl, name=f"Rock_Vine_{idx}")

def spawn_moss_curtain(x, y, z_top, length=1.8, width=0.45, name="MossCurtain"):
    bm = bmesh.new()
    segments_y = 12
    segments_x = 6
    grid = []
    for iy in range(segments_y + 1):
        t = iy / segments_y
        cz = z_top - t * length
        w_curr = width * (1.0 - t * 0.25)
        row = []
        for ix in range(segments_x + 1):
            tx = ix / segments_x
            cx = x + (tx - 0.5) * w_curr
            cy = y + math.sin(t * math.pi * 2.5 + tx * 3.0) * 0.05
            fringe = math.sin(tx * math.pi * 6.0) * 0.06 if t > 0.85 else 0.0
            row.append(bm.verts.new((cx, cy, cz + fringe)))
        grid.append(row)
    for iy in range(segments_y):
        for ix in range(segments_x):
            bm.faces.new((grid[iy][ix], grid[iy][ix+1], grid[iy+1][ix+1], grid[iy+1][ix]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_cavern_moss)
    return obj

moss_curtain_locations = [
    (-3.8, 11.0, 6.8, 2.4, 0.55),
    (3.8, 11.0, 6.8, 2.4, 0.55),
    (-6.8, 5.5, 3.2, 1.8, 0.50),
    (6.8, 5.5, 3.2, 1.8, 0.50),
    (-6.6, 1.8, 2.6, 1.6, 0.45),
    (6.6, 1.8, 2.6, 1.6, 0.45),
]
for idx, (mx, my, mz, ml, mw) in enumerate(moss_curtain_locations):
    spawn_moss_curtain(mx, my, mz, length=ml, width=mw, name=f"Moss_Curtain_{idx}")

def spawn_rock_fern_rosette(x, y, z, scale=1.0, num_fronds=7, name="RockFern"):
    bm = bmesh.new()
    for ifr in range(num_fronds):
        ang = (ifr / float(num_fronds)) * math.pi * 2.0 + (ifr * 0.3)
        length = 0.95 * scale
        width = 0.28 * scale
        segs = 10
        left_v = []
        right_v = []
        for iseg in range(segs + 1):
            t = iseg / segs
            dist = t * length
            w = math.sin(t * math.pi) * width * 0.5
            drop = (t ** 2.2) * 0.45 * scale
            lx = x + math.cos(ang) * dist
            ly = y + math.sin(ang) * dist
            lz = z + (t * 0.15 * scale) - drop

            px = -math.sin(ang) * w
            py = math.cos(ang) * w
            left_v.append(bm.verts.new((lx - px, ly - py, lz)))
            right_v.append(bm.verts.new((lx + px, ly + py, lz)))
        for iseg in range(segs):
            bm.faces.new((left_v[iseg], right_v[iseg], right_v[iseg + 1], left_v[iseg + 1]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_broad_leaf)
    return obj

fern_rosette_locations = [

    (-5.2, 6.6, 0.95, 1.1),
    (5.2, 6.6, 0.95, 1.1),
    (-6.1, 7.6, 1.85, 1.2),
    (6.1, 7.6, 1.85, 1.2),
    (-6.8, 8.6, 2.85, 1.0),
    (6.8, 8.6, 2.85, 1.0),

    (-5.6, -0.6, 0.85, 1.05),
    (5.6, -0.6, 0.85, 1.05),
    (-5.5, 1.2, 0.90, 1.15),
    (5.5, 1.0, 0.90, 1.15),
    (-6.4, 3.2, 2.50, 1.0),
    (6.4, 4.2, 2.60, 1.0),
]
for idx, (fx, fy, fz, fs) in enumerate(fern_rosette_locations):
    spawn_rock_fern_rosette(fx, fy, fz, scale=fs, name=f"Rock_Fern_{idx}")

def spawn_moss_patch(x, y, z, scale=1.0, variant=0, name="WetMossPatch"):
    mats = [mat_moss_wet, mat_cavern_moss, mat_moss_lime]
    rng = random.Random(variant * 97 + int(abs(x*y*z) * 10))
    for i in range(7):
        bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2, radius=0.22*scale,
            location=(x+(i-3)*0.18*scale+rng.uniform(-.08,.08), y, z+rng.uniform(-.10,.12)*scale))
        lobe=bpy.context.object; lobe.name=f"{name}_Lobe_{i}"; lobe.scale=(1.6,.55,rng.uniform(.55,1.1))
        lobe.data.materials.append(mats[(variant+i)%len(mats)])
        for poly in lobe.data.polygons: poly.use_smooth=True
    for i in range(4):
        bpy.ops.mesh.primitive_cylinder_add(vertices=10, radius=.028*scale,
            depth=rng.uniform(.45,.90)*scale, location=(x+(i-1.5)*.28*scale,y,z-.35*scale))
        frond=bpy.context.object; frond.name=f"{name}_Frond_{i}"; frond.rotation_euler[1]=rng.uniform(-.35,.35)
        frond.data.materials.append(mat_moss_fringes)
        for poly in frond.data.polygons: poly.use_smooth=True

for idx, args in enumerate([
    (-5.60,11.8,5.3,1.25,0), (-4.40,12.6,6.1,.90,1), (5.50,11.8,5.0,1.30,2),
    (4.40,12.6,6.2,.82,0), (-6.50,10.8,3.8,.72,2), (6.50,10.8,3.9,.78,1)]):
    spawn_moss_patch(*args, name=f"Waterfall_Back_Moss_{idx}")

def spawn_wall_cyan_spores(x, y, z, num_shrooms=5, name="WallCyanSpores"):
    bm = bmesh.new()
    rng = random.Random(int((abs(x) + abs(y) + abs(z)) * 100))
    for ish in range(num_shrooms):
        ox = rng.uniform(-0.25, 0.25)
        oy = rng.uniform(-0.15, 0.15)
        oz = rng.uniform(-0.20, 0.20)
        r_sh = rng.uniform(0.045, 0.085)
        segs_u = 12
        segs_v = 8
        sh_grid = []
        for iv in range(segs_v + 1):
            tv = iv / segs_v
            curr_r = tv * r_sh
            dz = - (tv ** 2.0) * 0.035
            row = []
            for iu in range(segs_u + 1):
                tu = iu / segs_u
                ang = -math.pi * 0.5 + tu * math.pi
                vx = x + ox + math.cos(ang) * curr_r
                vy = y + oy + math.sin(ang) * curr_r
                vz = z + oz + dz
                row.append(bm.verts.new((vx, vy, vz)))
            sh_grid.append(row)
        for iv in range(segs_v):
            for iu in range(segs_u):
                bm.faces.new((sh_grid[iv][iu], sh_grid[iv][iu+1], sh_grid[iv+1][iu+1], sh_grid[iv+1][iu]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_shroom_pale_cap)
    return obj

wall_spore_locations = [
    (-7.2, -0.5, 2.8),
    (-7.1, 1.8, 3.2),
    (-7.2, 3.8, 3.5),
    (-6.6, 6.2, 2.4),
    (-5.6, 11.8, 6.0),
    (7.2, -0.5, 2.8),
    (7.1, 1.8, 3.2),
    (7.2, 3.8, 3.5),
    (6.6, 6.2, 2.4),
    (5.6, 11.8, 6.0),
]
for idx, (wx, wy, wz) in enumerate(wall_spore_locations):
    spawn_wall_cyan_spores(wx, wy, wz, num_shrooms=6, name=f"Wall_Cyan_Spore_{idx}")

build_cyclopean_portal_arch()
build_waterfall_flank_cliffs()

def spawn_water_ripple_ring(x, y, z, radius, width=0.16, name="WaterRipple"):
    bm = bmesh.new()
    segments = 64
    inner_r = max(0.05, radius - width * 0.5)
    outer_r = radius + width * 0.5
    grid = []
    for i in range(segments + 1):
        ang = (i / segments) * math.pi * 2.0
        v_in = bm.verts.new((x + math.cos(ang) * inner_r, y + math.sin(ang) * inner_r, z))
        v_out = bm.verts.new((x + math.cos(ang) * outer_r, y + math.sin(ang) * outer_r, z))
        grid.append((v_in, v_out))
    for i in range(segments):
        v1_in, v1_out = grid[i]
        v2_in, v2_out = grid[i + 1]
        bm.faces.new((v1_in, v2_in, v2_out, v1_out))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_water_ripple_ring)
    return obj

def spawn_fallen_leaf(x, y, z, rot_z, scale=0.12, is_olive=False, name="FallenLeaf"):
    bm = bmesh.new()
    v0 = bm.verts.new((0.0, -scale * 0.8, 0.0))
    v1 = bm.verts.new((scale * 0.35, 0.0, scale * 0.04))
    v2 = bm.verts.new((0.0, scale * 0.8, scale * 0.02))
    v3 = bm.verts.new((-scale * 0.35, 0.0, scale * 0.04))
    bm.faces.new((v0, v1, v2, v3))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    rot_mat = Matrix.Rotation(rot_z, 4, 'Z')
    bm.transform(rot_mat)
    trans_mat = Matrix.Translation((x, y, z))
    bm.transform(trans_mat)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_leaf_olive if is_olive else mat_leaf_autumn)
    return obj

def spawn_dirt_mound_patch(x, y, z, sx=0.45, sy=0.35, seed=77, name="DirtMound"):
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=1, radius=1.0)
    for v in bm.verts:
        p = v.co
        norm = p.normalized()
        n = continuous_noise_3d(norm.x * 3.0 + seed, norm.y * 3.0 + seed, norm.z * 3.0 + seed)
        v.co = norm * (0.85 + n * 0.15)
        v.co.x *= sx
        v.co.y *= sy
        v.co.z = max(0.0, v.co.z) * 0.05
    trans_mat = Matrix.Translation((x, y, z))
    bm.transform(trans_mat)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_dirt_patch)
    return obj

def build_terrain_and_lake_rim():
    bpy.ops.mesh.primitive_grid_add(x_subdivisions=140, y_subdivisions=140, size=34.0, location=(0.0, 0.0, 0.0))
    terrain = bpy.context.active_object
    terrain.name = "Cavern_Terrain_Grid"

    bm = bmesh.new()
    bm.from_mesh(terrain.data)

    lake_center_x = 0.0
    lake_center_y = 8.5
    lake_radius = 5.2

    for v in bm.verts:
        v.co.z = get_terrain_surface_z(v.co.x, v.co.y)

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(terrain.data)
    bm.free()
    for p in terrain.data.polygons:
        p.use_smooth = True

    color_attr = terrain.data.color_attributes.new(name="Color", type='FLOAT_COLOR', domain='POINT')

    dirt_core = (0.22, 0.12, 0.05, 1.0)
    dirt_moss = (0.15, 0.18, 0.09, 1.0)
    grass_col = (0.08, 0.28, 0.13, 1.0)
    grass_highlight = (0.12, 0.35, 0.16, 1.0)
    lake_rim_pebble = (0.18, 0.20, 0.21, 1.0)
    submerged_shallow = (0.06, 0.11, 0.13, 1.0)
    submerged_abyss = (0.015, 0.035, 0.055, 1.0)

    for vi, v in enumerate(terrain.data.vertices):
        x = v.co.x
        y = v.co.y
        dist_l = math.sqrt((x - lake_center_x)**2 + (y - lake_center_y)**2)

        if dist_l < lake_radius:
            u = dist_l / lake_radius
            depth_f = math.pow(max(0.0, 1.0 - u), 1.5)
            r = submerged_shallow[0] * (1.0 - depth_f) + submerged_abyss[0] * depth_f
            g = submerged_shallow[1] * (1.0 - depth_f) + submerged_abyss[1] * depth_f
            b = submerged_shallow[2] * (1.0 - depth_f) + submerged_abyss[2] * depth_f
            color_attr.data[vi].color = (r, g, b, 1.0)
        elif dist_l < lake_radius + 1.2:
            t_rim = (dist_l - lake_radius) / 1.2
            r = lake_rim_pebble[0] * (1.0 - t_rim) + grass_col[0] * t_rim
            g = lake_rim_pebble[1] * (1.0 - t_rim) + grass_col[1] * t_rim
            b = lake_rim_pebble[2] * (1.0 - t_rim) + grass_col[2] * t_rim
            color_attr.data[vi].color = (r, g, b, 1.0)
        else:
            x_meander = math.sin(y * 0.45) * 0.20
            w_base = 1.65
            w_meander = math.sin(y * 0.68) * 0.35 + math.cos(y * 1.35) * 0.22
            w_path = w_base + w_meander
            dist_p = abs(x - x_meander)

            w_inner = max(0.5, w_path - 0.45)
            w_outer = w_path + 0.85

            n_disp = continuous_noise_3d(x * 0.75, y * 0.75, 0.0) * 0.30
            val = (dist_p + n_disp - w_inner) / (w_outer - w_inner)
            t = min(1.0, max(0.0, val))
            t_smooth = t * t * (3.0 - 2.0 * t)

            n_dirt = continuous_noise_3d(x * 1.5, y * 1.5, 1.0) * 0.5 + 0.5
            curr_dirt = (
                dirt_core[0] * (1.0 - n_dirt * 0.2) + dirt_moss[0] * (n_dirt * 0.2),
                dirt_core[1] * (1.0 - n_dirt * 0.2) + dirt_moss[1] * (n_dirt * 0.2),
                dirt_core[2] * (1.0 - n_dirt * 0.2) + dirt_moss[2] * (n_dirt * 0.2),
            )
            n_grass = continuous_noise_3d(x * 2.0, y * 2.0, 2.0) * 0.5 + 0.5
            curr_grass = (
                grass_col[0] * (1.0 - n_grass * 0.25) + grass_highlight[0] * (n_grass * 0.25),
                grass_col[1] * (1.0 - n_grass * 0.25) + grass_highlight[1] * (n_grass * 0.25),
                grass_col[2] * (1.0 - n_grass * 0.25) + grass_highlight[2] * (n_grass * 0.25),
            )

            r = curr_dirt[0] * (1.0 - t_smooth) + curr_grass[0] * t_smooth
            g = curr_dirt[1] * (1.0 - t_smooth) + curr_grass[1] * t_smooth
            b = curr_dirt[2] * (1.0 - t_smooth) + curr_grass[2] * t_smooth
            color_attr.data[vi].color = (r, g, b, 1.0)

    terrain.data.materials.clear()
    terrain.data.materials.append(mat_terrain_splat)
    terrain.data.update()

    bm_rim = bmesh.new()
    rim_pts = 72
    rim_grid = []
    rim_top_z = 0.08
    rim_bot_z = -0.06

    for ip in range(rim_pts + 1):
        frac = ip / rim_pts
        ang = math.pi * 0.85 + frac * math.pi * 1.30

        r_inner = lake_radius - 0.08
        r_outer = lake_radius + 0.18

        xi = lake_center_x + math.cos(ang) * r_inner
        yi = lake_center_y + math.sin(ang) * r_inner

        xo = lake_center_x + math.cos(ang) * r_outer
        yo = lake_center_y + math.sin(ang) * r_outer

        v_in_top = bm_rim.verts.new((xi, yi, rim_top_z))
        v_out_top = bm_rim.verts.new((xo, yo, rim_top_z))
        v_out_bot = bm_rim.verts.new((xo, yo, rim_bot_z))
        v_in_bot = bm_rim.verts.new((xi, yi, rim_bot_z))
        rim_grid.append((v_in_top, v_out_top, v_out_bot, v_in_bot))

    for ip in range(rim_pts):
        r1 = rim_grid[ip]
        r2 = rim_grid[ip + 1]
        bm_rim.faces.new((r1[0], r2[0], r2[1], r1[1]))
        bm_rim.faces.new((r1[1], r2[1], r2[2], r1[2]))

    bmesh.ops.recalc_face_normals(bm_rim, faces=bm_rim.faces)
    rim_mesh = bpy.data.meshes.new("Lake_Fine_Rim_Mesh")
    rim_obj = bpy.data.objects.new("Lake_Fine_Rim", rim_mesh)
    bpy.context.collection.objects.link(rim_obj)
    bm_rim.to_mesh(rim_mesh)
    bm_rim.free()
    for p in rim_mesh.polygons:
        p.use_smooth = True
    rim_mesh.update()
    rim_obj.data.materials.append(mat_fine_rim)

build_terrain_and_lake_rim()

spawn_water_ripple_ring(0.0, 8.5, 0.082, 1.35, width=0.14, name="WaterRipple_1")
spawn_water_ripple_ring(0.0, 8.5, 0.082, 2.55, width=0.16, name="WaterRipple_2")
spawn_water_ripple_ring(0.0, 8.5, 0.082, 3.85, width=0.18, name="WaterRipple_3")

def spawn_organic_flagstone(idx, sx, sy, sz, rx, ry, rz_scale, pitch, roll, yaw, is_submerged=False, seed=100):
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=2, radius=1.0)
    rng = random.Random(seed + idx * 37)

    chip_angle = rng.uniform(0.0, math.pi * 2.0)
    chip_dir = Vector((math.cos(chip_angle), math.sin(chip_angle), 0.0))

    for v in bm.verts:
        p = v.co
        norm = p.normalized()
        n = continuous_noise_3d(norm.x * 2.8 + seed, norm.y * 2.8 + seed, norm.z * 2.8 + seed)
        facet = 1.0 + (math.floor(n * 3.5) / 3.5) * 0.25 + rng.uniform(-0.04, 0.04)

        dot_chip = norm.dot(chip_dir)
        if dot_chip > 0.65:
            facet *= (1.0 - (dot_chip - 0.65) * 0.75)

        v.co = norm * (facet * 0.95)
        v.co.x *= rx
        v.co.y *= ry
        v.co.z *= rz_scale

        if v.co.z < -0.04:
            v.co.z *= 0.72

    rot_pitch = Matrix.Rotation(pitch, 4, 'X')
    rot_roll = Matrix.Rotation(roll, 4, 'Y')
    rot_yaw = Matrix.Rotation(yaw, 4, 'Z')
    rot_mat = rot_yaw @ rot_roll @ rot_pitch
    bm.transform(rot_mat)

    trans_mat = Matrix.Translation((sx, sy, sz))
    bm.transform(trans_mat)

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)

    mesh = bpy.data.meshes.new(f"PathFlagstone_{idx}_Mesh")
    obj = bpy.data.objects.new(f"PathFlagstone_{idx}", mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()

    for p in mesh.polygons:
        p.use_smooth = False
    mesh.update()

    ca = mesh.color_attributes.new(name="Color", type='FLOAT_COLOR', domain='POINT')
    for vi, v in enumerate(mesh.vertices):
        rel_z = v.co.z - sz
        if is_submerged:
            ca.data[vi].color = (0.05, 0.09, 0.13, 1.0)
        else:
            if rel_z < -0.015:
                ca.data[vi].color = (0.13, 0.10, 0.05, 1.0)
            elif rel_z < 0.025:
                ca.data[vi].color = (0.18, 0.22, 0.16, 1.0)
            else:
                ca.data[vi].color = (0.28, 0.32, 0.35, 1.0)

    if is_submerged:
        obj.data.materials.append(mat_wet_submerged_rock)
    else:
        obj.data.materials.append(mat_stone_slab)
        obj.data.materials.append(mat_rock_shadow)
    return obj

def spawn_path_flagstones():
    flagstone_data = [
        (-0.42, -2.6, 0.026, 0.58, 0.44, 0.16, -0.05, 0.06, 0.25, False, 101),
        (0.35, -1.9, 0.025, 0.52, 0.48, 0.16, 0.06, -0.05, -0.32, False, 102),
        (-0.38, -1.1, 0.028, 0.62, 0.42, 0.17, -0.04, 0.07, 0.45, False, 103),
        (0.32, -0.3, 0.025, 0.48, 0.50, 0.16, 0.07, -0.04, -0.20, False, 104),
        (-0.28, 0.5, 0.027, 0.56, 0.44, 0.17, -0.06, -0.05, 0.35, False, 105),
        (0.36, 1.3, 0.026, 0.50, 0.48, 0.16, 0.05, 0.06, -0.40, False, 106),
        (-0.25, 2.1, 0.027, 0.54, 0.46, 0.17, -0.04, 0.05, 0.18, False, 107),
        (0.30, 2.8, 0.028, 0.50, 0.44, 0.16, 0.06, -0.06, -0.25, False, 108),
        (-0.10, 3.4, 0.022, 0.58, 0.46, 0.17, -0.05, 0.04, 0.15, False, 109),
        (0.18, 4.0, 0.012, 0.52, 0.48, 0.16, 0.04, -0.05, 0.30, True, 110),
        (-0.15, 4.6, -0.012, 0.56, 0.50, 0.17, -0.06, 0.06, -0.20, True, 111),
        (0.20, 5.2, -0.038, 0.50, 0.45, 0.16, 0.05, -0.04, 0.40, True, 112),
    ]
    for idx, (sx, sy, sz, rx, ry, rz, pitch, roll, yaw, is_subm, seed) in enumerate(flagstone_data):
        spawn_organic_flagstone(idx, sx, sy, sz, rx, ry, rz, pitch, roll, yaw, is_submerged=is_subm, seed=seed)

spawn_path_flagstones()

def spawn_river_pebble(x, y, z, sx, sy, rot_z=0.0):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=12, ring_count=8, radius=0.09, location=(x, y, z))
    peb = bpy.context.active_object
    peb.scale = (sx, sy, 0.40)
    peb.rotation_euler.z = rot_z
    bpy.ops.object.transform_apply(scale=True, rotation=True)
    peb.data.materials.append(mat_river_pebble)
    for p in peb.data.polygons:
        p.use_smooth = True

pebble_coords = [
    (-2.1, -2.6, 0.04, 0.26, 0.20, 0.3),
    (-1.8, -1.9, 0.04, 0.32, 0.24, -0.5),
    (-2.2, -1.2, 0.04, 0.28, 0.22, 0.8),
    (-1.9, -0.5, 0.04, 0.35, 0.26, -0.2),
    (-2.3, 0.2, 0.04, 0.30, 0.22, 0.4),
    (-2.0, 0.9, 0.04, 0.34, 0.25, -0.6),
    (-2.3, 1.7, 0.05, 0.32, 0.24, 0.2),
    (-1.9, 2.4, 0.05, 0.28, 0.20, 0.7),
    (-2.2, 3.1, 0.05, 0.35, 0.26, -0.4),
    (-1.8, 3.7, 0.05, 0.26, 0.20, 0.5),

    (2.1, -2.6, 0.04, 0.28, 0.22, -0.4),
    (1.8, -1.9, 0.04, 0.30, 0.24, 0.6),
    (2.2, -1.2, 0.04, 0.34, 0.25, -0.2),
    (1.9, -0.5, 0.04, 0.26, 0.20, 0.5),
    (2.3, 0.2, 0.04, 0.32, 0.24, -0.7),
    (2.0, 0.9, 0.04, 0.35, 0.26, 0.3),
    (2.3, 1.7, 0.05, 0.28, 0.22, -0.5),
    (1.9, 2.4, 0.05, 0.34, 0.25, 0.2),
    (2.2, 3.1, 0.05, 0.30, 0.22, -0.3),
    (1.8, 3.7, 0.05, 0.32, 0.24, 0.6),

    (-1.3, -2.1, 0.035, 0.18, 0.15, 0.2),
    (1.2, -1.5, 0.035, 0.20, 0.16, -0.4),
    (-1.1, -0.2, 0.035, 0.19, 0.14, 0.5),
    (1.3, 0.8, 0.035, 0.22, 0.17, -0.3),
    (-1.2, 1.8, 0.04, 0.18, 0.15, 0.4),
    (1.1, 2.7, 0.04, 0.21, 0.16, -0.6),
    (-0.9, 3.5, 0.04, 0.24, 0.18, 0.1),
    (0.8, 3.6, 0.04, 0.22, 0.17, -0.2),
]
for (px, py, pz, psx, psy, prot) in pebble_coords:
    spawn_river_pebble(px, py, pz, psx, psy, prot)

leaf_coords = [
    (-1.85, -2.2, 0.042, 0.4, 0.13, False, "AutumnLeaf_0"),
    (-1.65, -1.4, 0.042, 1.2, 0.11, True, "OliveLeaf_1"),
    (-2.05, -0.6, 0.042, -0.7, 0.14, False, "AutumnLeaf_2"),
    (-1.75, 0.3, 0.042, 2.1, 0.12, True, "OliveLeaf_3"),
    (-2.10, 1.2, 0.045, -1.5, 0.13, False, "AutumnLeaf_4"),
    (-1.70, 2.0, 0.045, 0.8, 0.11, True, "OliveLeaf_5"),
    (-1.95, 2.9, 0.045, -0.4, 0.14, False, "AutumnLeaf_6"),

    (1.85, -2.0, 0.042, -0.8, 0.12, True, "OliveLeaf_7"),
    (1.65, -1.2, 0.042, 1.5, 0.14, False, "AutumnLeaf_8"),
    (2.05, -0.3, 0.042, -1.1, 0.11, True, "OliveLeaf_9"),
    (1.75, 0.6, 0.042, 0.6, 0.13, False, "AutumnLeaf_10"),
    (2.10, 1.5, 0.045, -2.0, 0.12, True, "OliveLeaf_11"),
    (1.70, 2.3, 0.045, 1.1, 0.14, False, "AutumnLeaf_12"),
    (1.95, 3.2, 0.045, -0.5, 0.12, True, "OliveLeaf_13"),
]
for (lx, ly, lz, lrot, lscale, lolive, lname) in leaf_coords:
    spawn_fallen_leaf(lx, ly, lz, lrot, scale=lscale, is_olive=lolive, name=lname)

dirt_mound_coords = [
    (-2.15, -1.6, 0.038, 0.48, 0.36, 71, "DirtMound_0"),
    (-2.25, 0.0, 0.038, 0.52, 0.38, 72, "DirtMound_1"),
    (-2.20, 1.5, 0.042, 0.46, 0.34, 73, "DirtMound_2"),
    (-2.30, 2.8, 0.042, 0.50, 0.38, 74, "DirtMound_3"),

    (2.15, -1.4, 0.038, 0.50, 0.36, 81, "DirtMound_4"),
    (2.25, 0.3, 0.038, 0.48, 0.35, 82, "DirtMound_5"),
    (2.20, 1.8, 0.042, 0.54, 0.40, 83, "DirtMound_6"),
    (2.30, 3.0, 0.042, 0.46, 0.36, 84, "DirtMound_7"),
]
for (mx, my, mz, msx, msy, mseed, mname) in dirt_mound_coords:
    spawn_dirt_mound_patch(mx, my, mz, sx=msx, sy=msy, seed=mseed, name=mname)

def spawn_organic_transition_rock(x, y, z, sx=0.55, sy=0.45, sz=0.35, rot_z=0.0, seed=42):

    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=2, radius=1.0)

    rng = random.Random(seed)
    for v in bm.verts:
        p = v.co
        dist = p.length
        if dist > 0.0001:
            norm = p.normalized()
            n = continuous_noise_3d(norm.x * 2.5 + seed, norm.y * 2.5 + seed, norm.z * 2.5 + seed)
            bottom_flatten = 0.72 if p.z < -0.1 else 1.0
            facet_snap = 1.0 + (math.floor(n * 3.0) / 3.0) * 0.30 + rng.uniform(-0.05, 0.05)
            v.co = norm * (dist * facet_snap * bottom_flatten)
            v.co.x *= sx
            v.co.y *= sy
            v.co.z *= sz

    rot_mat = Matrix.Rotation(rot_z, 4, 'Z')
    bm.transform(rot_mat)
    trans_mat = Matrix.Translation((x, y, z))
    bm.transform(trans_mat)

    for f in bm.faces:
        if f.normal.z > 0.48:
            f.material_index = 1
        else:
            f.material_index = 0

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"TransitionRock_{seed}_Mesh")
    obj = bpy.data.objects.new(f"TransitionRock_{seed}", mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()

    for p in mesh.polygons:
        p.use_smooth = False
    mesh.update()

    obj.data.materials.append(mat_angular_rock)
    obj.data.materials.append(mat_cavern_moss)
    return obj

organic_rock_coords = [
    (-2.4, 1.1, 0.10, 0.52, 0.42, 0.30, 0.4, 301),
    (-2.5, 2.3, 0.12, 0.58, 0.46, 0.34, -0.6, 302),
    (-2.7, 3.4, 0.15, 0.64, 0.50, 0.38, 0.8, 303),
    (2.3, 1.0, 0.10, 0.50, 0.40, 0.28, -0.4, 304),
    (2.4, 2.2, 0.12, 0.56, 0.44, 0.32, 0.6, 305),
    (2.6, 3.3, 0.15, 0.62, 0.48, 0.36, -0.8, 306),

    (-2.3, -0.8, 0.08, 0.50, 0.42, 0.28, 0.3, 310),
    (-2.4, 0.2, 0.09, 0.54, 0.44, 0.32, -0.5, 312),
    (2.3, -0.8, 0.08, 0.48, 0.40, 0.26, -0.3, 311),
    (2.4, 0.2, 0.09, 0.52, 0.42, 0.30, 0.5, 313),
]
for (ax, ay, az, asx, asy, asz, arot, aseed) in organic_rock_coords:
    spawn_organic_transition_rock(ax, ay, az, asx, asy, asz, arot, aseed)

def build_stylized_parasol_mushroom(root_x, root_y, scale=1.0, lean_angle=-0.22, name="ParasolMushroom"):

    ground_z = get_terrain_surface_z(root_x, root_y)
    root_z = ground_z - 0.15 * scale

    bm_stem = bmesh.new()
    stem_segments = 24
    radial_segments = 20
    h_stem = 2.4 * scale
    r_base = 0.20 * scale

    stem_rows = []
    for ih in range(stem_segments + 1):
        t = ih / stem_segments
        curr_z = root_z + t * h_stem
        curr_x = root_x + (t ** 1.3) * math.sin(lean_angle) * h_stem
        curr_y = root_y + (t ** 1.3) * 0.25 * scale

        root_flare = (((0.25 - t) / 0.25) ** 2.0) * 0.28 * scale if t < 0.25 else 0.0
        r_curr = r_base * (1.0 - t * 0.42) + root_flare

        row = []
        for ir in range(radial_segments):
            angle = 2.0 * math.pi * ir / radial_segments
            vx = curr_x + math.cos(angle) * r_curr
            vy = curr_y + math.sin(angle) * r_curr
            vz = curr_z
            row.append(bm_stem.verts.new((vx, vy, vz)))
        stem_rows.append(row)

    for ih in range(stem_segments):
        for ir in range(radial_segments):
            ir_next = (ir + 1) % radial_segments
            bm_stem.faces.new((stem_rows[ih][ir], stem_rows[ih][ir_next], stem_rows[ih + 1][ir_next], stem_rows[ih + 1][ir]))

    bmesh.ops.recalc_face_normals(bm_stem, faces=bm_stem.faces)

    tip_x = root_x + math.sin(lean_angle) * h_stem
    tip_y = root_y + 0.25 * scale
    tip_z = root_z + h_stem
    tip_pos = Vector((tip_x, tip_y, tip_z))

    tangent = Vector((1.30 * math.sin(lean_angle) * h_stem, 1.30 * 0.25 * scale, h_stem)).normalized()

    stem_mesh = bpy.data.meshes.new(f"{name}_Stem_Mesh")
    stem_obj = bpy.data.objects.new(f"{name}_Stem", stem_mesh)
    bpy.context.collection.objects.link(stem_obj)
    bm_stem.to_mesh(stem_mesh)
    bm_stem.free()
    for p in stem_mesh.polygons:
        p.use_smooth = True
    stem_mesh.update()
    stem_obj.data.materials.append(mat_shroom_stem)

    r_cap = 1.15 * scale
    bm_cap = bmesh.new()
    cap_radials = 36
    cap_rings = 16

    cap_grid = []
    for iring in range(cap_rings + 1):
        t_r = iring / cap_rings
        ring_r = t_r * r_cap
        drop_z = (t_r ** 2.2) * 0.62 * scale
        curr_z = 0.40 * scale - drop_z

        row = []
        for ia in range(cap_radials):
            angle = 2.0 * math.pi * ia / cap_radials
            rim_wave = 0.0
            if t_r > 0.80:
                rim_wave = math.sin(angle * 6.0) * 0.03 * scale
            vx = math.cos(angle) * (ring_r + rim_wave)
            vy = math.sin(angle) * (ring_r + rim_wave)
            vz = curr_z + rim_wave * 0.3
            row.append(bm_cap.verts.new((vx, vy, vz)))
        cap_grid.append(row)

    for iring in range(cap_rings):
        for ia in range(cap_radials):
            ia_next = (ia + 1) % cap_radials
            f = bm_cap.faces.new((cap_grid[iring][ia], cap_grid[iring][ia_next], cap_grid[iring + 1][ia_next], cap_grid[iring + 1][ia]))
            f.material_index = 0

    rim_verts = cap_grid[-1]
    neck_verts = []
    for ia in range(cap_radials):
        angle = 2.0 * math.pi * ia / cap_radials
        pleat = math.sin(angle * 18.0) * 0.02 * scale
        vx = math.cos(angle) * (0.22 * scale + pleat)
        vy = math.sin(angle) * (0.22 * scale + pleat)
        vz = 0.0
        neck_verts.append(bm_cap.verts.new((vx, vy, vz)))

    for ia in range(cap_radials):
        ia_next = (ia + 1) % cap_radials
        f_gill = bm_cap.faces.new((rim_verts[ia], neck_verts[ia], neck_verts[ia_next], rim_verts[ia_next]))
        f_gill.material_index = 1

    bmesh.ops.recalc_face_normals(bm_cap, faces=bm_cap.faces)

    rot_mat = Vector((0.0, 0.0, 1.0)).rotation_difference(tangent).to_matrix().to_4x4()
    trans_pos = tip_pos - tangent * (0.10 * scale)
    trans_mat = Matrix.Translation(trans_pos)
    bm_cap.transform(trans_mat @ rot_mat)

    cap_mesh = bpy.data.meshes.new(f"{name}_Cap_Mesh")
    cap_obj = bpy.data.objects.new(f"{name}_Cap", cap_mesh)
    bpy.context.collection.objects.link(cap_obj)
    bm_cap.to_mesh(cap_mesh)
    bm_cap.free()
    for p in cap_mesh.polygons:
        p.use_smooth = True
    cap_mesh.update()
    cap_obj.data.materials.append(mat_shroom_pale_cap)
    cap_obj.data.materials.append(mat_shroom_gills)
    cap_obj.parent = stem_obj

build_stylized_parasol_mushroom(-3.60, 3.20, scale=1.05, lean_angle=0.10, name="Left_Shroom_Large")
build_stylized_parasol_mushroom(-2.95, 2.45, scale=0.72, lean_angle=0.08, name="Left_Shroom_Medium")
build_stylized_parasol_mushroom(-4.10, 2.15, scale=0.48, lean_angle=0.06, name="Left_Shroom_Small")

def spawn_broad_leaf_spray(x, y, z, scale=1.0, num_leaves=7, name="BroadLeafSpray"):
    bm = bmesh.new()
    for il in range(num_leaves):
        ang = (il / float(num_leaves)) * math.pi * 1.8 - 0.4
        length = 1.20 * scale
        width = 0.40 * scale

        segs = 8
        left_v = []
        right_v = []
        for iseg in range(segs + 1):
            t = iseg / segs
            dist = t * length
            w = math.sin(t * math.pi) * width * 0.5
            drop = (t ** 1.8) * 0.38 * scale
            lx = x + math.cos(ang) * dist
            ly = y + math.sin(ang) * dist
            lz = z + (t * 0.28 * scale) - drop

            px = -math.sin(ang) * w
            py = math.cos(ang) * w
            left_v.append(bm.verts.new((lx - px, ly - py, lz)))
            right_v.append(bm.verts.new((lx + px, ly + py, lz)))

        for iseg in range(segs):
            bm.faces.new((left_v[iseg], right_v[iseg], right_v[iseg + 1], left_v[iseg + 1]))

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_broad_leaf)

spawn_broad_leaf_spray(-3.3, 2.1, 0.10, scale=1.15, name="Left_Fern_Spray_1")
spawn_broad_leaf_spray(-3.9, 3.0, 0.12, scale=0.95, name="Left_Fern_Spray_2")

spawn_broad_leaf_spray(-2.3, -0.2, 0.08, scale=0.85, name="Path_Creeping_Leaf_L")
spawn_broad_leaf_spray(2.3, -0.2, 0.08, scale=0.85, name="Path_Creeping_Leaf_R")

def spawn_glowing_stalk(x, y, z, h=0.75, name="GlowingStalk"):
    bpy.ops.mesh.primitive_cylinder_add(vertices=16, radius=0.016, depth=h, location=(x, y, z + h * 0.5))
    stem = bpy.context.active_object
    stem.name = f"{name}_Stem"
    stem.data.materials.append(mat_cavern_vine)
    for p in stem.data.polygons:
        p.use_smooth = True

    bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=12, radius=0.055, location=(x + 0.03, y, z + h))
    tip = bpy.context.active_object
    tip.name = f"{name}_Tip"
    tip.data.materials.append(mat_glowing_tip)
    for p in tip.data.polygons:
        p.use_smooth = True

def spawn_dandelion(x, y, z, height=0.82, bloom=True, seed_head=False, name="Dandelion"):
    bm = bmesh.new()
    rings = []
    for i in range(13):
        t = i / 12.0
        center = Vector((x + 0.07 * math.sin(t * math.pi), y, z + height * t))
        radius = 0.028 * (1.0 - 0.25 * t)
        rings.append([bm.verts.new((center.x + math.cos(math.tau * j / 10) * radius,
                                    center.y + math.sin(math.tau * j / 10) * radius, center.z)) for j in range(10)])
    for i in range(12):
        for j in range(10):
            bm.faces.new((rings[i][j], rings[i][(j + 1) % 10], rings[i + 1][(j + 1) % 10], rings[i + 1][j]))

    for li in range(5):
        a = li * math.tau / 5.0
        left = []
        right = []
        for k in range(7):
            t = k / 6.0
            length = 0.42 * t
            width = math.sin(t * math.pi) * 0.055
            cx = x + math.cos(a) * length
            cy = y + math.sin(a) * length
            cz = z + 0.02
            px = -math.sin(a) * width
            py = math.cos(a) * width
            left.append(bm.verts.new((cx - px, cy - py, cz)))
            right.append(bm.verts.new((cx + px, cy + py, cz)))
        for k in range(6):
            f = bm.faces.new((left[k], right[k], right[k + 1], left[k + 1]))
            f.material_index = 1

    top_ring = rings[-1]
    head = Vector((x + 0.07 * math.sin(math.pi), y, z + height))

    bract_rim = []
    rim_count = 14
    for j in range(rim_count):
        a = math.tau * j / rim_count
        rv = bm.verts.new(head + Vector((math.cos(a) * 0.052, math.sin(a) * 0.052, 0.010)))
        bract_rim.append(rv)

    for j in range(10):
        rj0 = int(j * rim_count / 10)
        rj1 = int((j + 1) * rim_count / 10) % rim_count
        f_cup = bm.faces.new((top_ring[j], top_ring[(j + 1) % 10], bract_rim[rj1], bract_rim[rj0]))
        f_cup.material_index = 1

    for j in range(rim_count):
        a = math.tau * j / rim_count
        tip_pos = head + Vector((math.cos(a) * 0.072, math.sin(a) * 0.072, -0.012))
        tip_v = bm.verts.new(tip_pos)
        f_bract = bm.faces.new((bract_rim[j], bract_rim[(j + 1) % rim_count], tip_v))
        f_bract.material_index = 1

    if bloom:
        floret_layers = [
            (32, 0.038, 0.155, 0.012, 0.024, 0.016, 0.022, 0.0),
            (24, 0.028, 0.115, 0.018, 0.038, 0.048, 0.018, 0.5),
            (16, 0.018, 0.072, 0.025, 0.052, 0.066, 0.014, 0.25),
        ]
        for count, r_base, r_tip, z_base, z_mid, z_tip, width, phase in floret_layers:
            r_mid = (r_base + r_tip) * 0.52
            for i in range(count):
                a = math.tau * (i + phase) / count
                ca = math.cos(a)
                sa = math.sin(a)
                pa_x = -sa * (width * 0.5)
                pa_y = ca * (width * 0.5)

                p_base_l = head + Vector((ca * r_base - pa_x * 0.6, sa * r_base - pa_y * 0.6, z_base))
                p_base_r = head + Vector((ca * r_base + pa_x * 0.6, sa * r_base + pa_y * 0.6, z_base))
                p_mid_l = head + Vector((ca * r_mid - pa_x, sa * r_mid - pa_y, z_mid))
                p_mid_r = head + Vector((ca * r_mid + pa_x, sa * r_mid + pa_y, z_mid))
                p_tip = head + Vector((ca * r_tip, sa * r_tip, z_tip))

                vl1 = bm.verts.new(p_base_l)
                vr1 = bm.verts.new(p_base_r)
                vl2 = bm.verts.new(p_mid_l)
                vr2 = bm.verts.new(p_mid_r)
                vt = bm.verts.new(p_tip)

                f1 = bm.faces.new((vl1, vr1, vr2, vl2))
                f1.material_index = 2
                f2 = bm.faces.new((vl2, vr2, vt))
                f2.material_index = 2

                thick = Vector((0.0, 0.0, -0.0022))
                vl1_b = bm.verts.new(p_base_l + thick)
                vr1_b = bm.verts.new(p_base_r + thick)
                vl2_b = bm.verts.new(p_mid_l + thick)
                vr2_b = bm.verts.new(p_mid_r + thick)
                vt_b = bm.verts.new(p_tip + thick)

                f1_b = bm.faces.new((vl2_b, vr2_b, vr1_b, vl1_b))
                f1_b.material_index = 2
                f2_b = bm.faces.new((vt_b, vr2_b, vl2_b))
                f2_b.material_index = 2

        dome_rings = 4
        dome_segs = 16
        dome_r = 0.038
        dome_grid = []
        for ir in range(dome_rings + 1):
            u = ir / dome_rings
            ring_radius = dome_r * math.sin(u * math.pi * 0.5)
            curr_z = 0.026 + 0.016 * math.cos(u * math.pi * 0.5)
            row = []
            for ia in range(dome_segs):
                ang = math.tau * ia / dome_segs
                row.append(bm.verts.new(head + Vector((math.cos(ang) * ring_radius, math.sin(ang) * ring_radius, curr_z))))
            dome_grid.append(row)

        for ir in range(dome_rings):
            for ia in range(dome_segs):
                ia_next = (ia + 1) % dome_segs
                fd = bm.faces.new((dome_grid[ir][ia], dome_grid[ir][ia_next], dome_grid[ir + 1][ia_next], dome_grid[ir + 1][ia]))
                fd.material_index = 2

    if seed_head:
        seed_count = 52
        golden_angle = math.pi * (3.0 - math.sqrt(5.0))
        for i in range(seed_count):
            t = (i + 0.5) / seed_count
            incl = math.acos(1.0 - 1.85 * t)
            azim = golden_angle * i
            dir_v = Vector((math.sin(incl) * math.cos(azim), math.sin(incl) * math.sin(azim), math.cos(incl))).normalized()
            p_stalk_base = head + dir_v * 0.032
            p_stalk_tip = head + dir_v * 0.175

            v_sb = bm.verts.new(p_stalk_base)
            v_st = bm.verts.new(p_stalk_tip)
            ortho_u = dir_v.cross(Vector((0.0, 0.0, 1.0)) if abs(dir_v.z) < 0.9 else Vector((1.0, 0.0, 0.0))).normalized()
            ortho_v = dir_v.cross(ortho_u).normalized()

            v_s1 = bm.verts.new(p_stalk_base + ortho_u * 0.003)
            f_stalk = bm.faces.new((v_sb, v_s1, v_st))
            f_stalk.material_index = 0

            pappus_spokes = 8
            spoke_r = 0.038
            for ps in range(pappus_spokes):
                pang = math.tau * ps / pappus_spokes
                p_spoke_tip = p_stalk_tip + (ortho_u * math.cos(pang) + ortho_v * math.sin(pang)) * spoke_r + dir_v * 0.012
                v_pt = bm.verts.new(p_spoke_tip)
                pang_next = math.tau * (ps + 0.5) / pappus_spokes
                v_pw = bm.verts.new(p_stalk_tip + (ortho_u * math.cos(pang_next) + ortho_v * math.sin(pang_next)) * (spoke_r * 0.4))
                fp = bm.faces.new((v_st, v_pt, v_pw))
                fp.material_index = 3

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for poly in mesh.polygons:
        poly.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_dandelion_stem)
    obj.data.materials.append(mat_dandelion_leaf)
    obj.data.materials.append(mat_dandelion_flower)
    obj.data.materials.append(mat_dandelion_seed)

for idx, (dx, dy, dh, bloom, seed_head) in enumerate([
    (-3.4,1.9,0.72,True,False), (-4.2,2.0,0.88,True,False), (-2.8,2.9,0.66,True,False),
    (-4.0,3.5,0.80,False,True), (2.7,1.7,0.68,True,False), (3.25,2.55,0.84,False,True)]):
    spawn_dandelion(dx, dy, get_terrain_surface_z(dx, dy), dh, bloom, seed_head, f"Dandelion_{idx}")

def build_stylized_lathe_mushroom(root_x, root_y, scale=1.0, lean_angle=-0.15, name="Right_Dome_Mushroom"):

    ground_z = get_terrain_surface_z(root_x, root_y)
    root_z = ground_z - 0.12 * scale

    bm_stem = bmesh.new()
    stem_segments = 24
    radial_segments = 24
    h_stem = 1.95 * scale
    r_stem = 0.15 * scale

    stem_rows = []
    for ih in range(stem_segments + 1):
        t = ih / stem_segments
        curr_z = root_z + t * h_stem

        curr_x = root_x + (t ** 1.30) * math.sin(lean_angle) * (0.35 * h_stem)
        curr_y = root_y + (t ** 1.30) * math.cos(lean_angle) * (0.22 * h_stem)

        root_flare = (((0.28 - t) / 0.28) ** 2.0) * 0.20 * scale if t < 0.28 else 0.0
        r_curr = r_stem * (1.0 - t * 0.32) + root_flare

        row = []
        for ir in range(radial_segments):
            angle = 2.0 * math.pi * ir / radial_segments
            vx = curr_x + math.cos(angle) * r_curr
            vy = curr_y + math.sin(angle) * r_curr
            vz = curr_z
            row.append(bm_stem.verts.new((vx, vy, vz)))
        stem_rows.append(row)

    for ih in range(stem_segments):
        for ir in range(radial_segments):
            ir_next = (ir + 1) % radial_segments
            bm_stem.faces.new((stem_rows[ih][ir], stem_rows[ih][ir_next], stem_rows[ih + 1][ir_next], stem_rows[ih + 1][ir]))

    bmesh.ops.recalc_face_normals(bm_stem, faces=bm_stem.faces)

    tip_x = root_x + math.sin(lean_angle) * (0.35 * h_stem)
    tip_y = root_y + math.cos(lean_angle) * (0.22 * h_stem)
    tip_z = root_z + h_stem
    tip_pos = Vector((tip_x, tip_y, tip_z))

    tangent = Vector((1.30 * math.sin(lean_angle) * (0.35 * h_stem), 1.30 * math.cos(lean_angle) * (0.22 * h_stem), h_stem)).normalized()

    stem_mesh = bpy.data.meshes.new(f"{name}_Stem_Mesh")
    stem_obj = bpy.data.objects.new(f"{name}_Stem", stem_mesh)
    bpy.context.collection.objects.link(stem_obj)
    bm_stem.to_mesh(stem_mesh)
    bm_stem.free()
    for p in stem_mesh.polygons:
        p.use_smooth = True
    stem_mesh.update()
    stem_obj.data.materials.append(mat_shroom_stem)

    bm_cap = bmesh.new()
    cap_radials = 36
    cap_rings = 20
    r_cap = 0.62 * scale

    cap_grid = []
    for ir in range(cap_rings + 1):
        u = ir / cap_rings
        ring_r = u * r_cap
        drop_z = (u ** 2.3) * (0.36 * scale)
        curr_z = 0.35 * scale - drop_z

        row = []
        for ia in range(cap_radials):
            angle = 2.0 * math.pi * ia / cap_radials
            vx = math.cos(angle) * ring_r
            vy = math.sin(angle) * ring_r
            vz = curr_z
            row.append(bm_cap.verts.new((vx, vy, vz)))
        cap_grid.append(row)

    for ir in range(cap_rings):
        for ia in range(cap_radials):
            ia_next = (ia + 1) % cap_radials
            f = bm_cap.faces.new((cap_grid[ir][ia], cap_grid[ir][ia_next], cap_grid[ir + 1][ia_next], cap_grid[ir + 1][ia]))
            f.material_index = 0

    rim_verts = cap_grid[-1]
    neck_verts = []
    for ia in range(cap_radials):
        angle = 2.0 * math.pi * ia / cap_radials
        vx = math.cos(angle) * (0.16 * scale)
        vy = math.sin(angle) * (0.16 * scale)
        vz = 0.0
        neck_verts.append(bm_cap.verts.new((vx, vy, vz)))

    for ia in range(cap_radials):
        ia_next = (ia + 1) % cap_radials
        f_cavity = bm_cap.faces.new((rim_verts[ia], neck_verts[ia], neck_verts[ia_next], rim_verts[ia_next]))
        f_cavity.material_index = 1

    bmesh.ops.recalc_face_normals(bm_cap, faces=bm_cap.faces)

    rot_mat = Vector((0.0, 0.0, 1.0)).rotation_difference(tangent).to_matrix().to_4x4()
    trans_pos = tip_pos - tangent * (0.08 * scale)
    trans_mat = Matrix.Translation(trans_pos)
    bm_cap.transform(trans_mat @ rot_mat)

    cap_mesh = bpy.data.meshes.new(f"{name}_Cap_Mesh")
    cap_obj = bpy.data.objects.new(f"{name}_Cap", cap_mesh)
    bpy.context.collection.objects.link(cap_obj)
    bm_cap.to_mesh(cap_mesh)
    bm_cap.free()
    for p in cap_mesh.polygons:
        p.use_smooth = True
    cap_mesh.update()
    cap_obj.data.materials.append(mat_shroom_vibrant_cap)
    cap_obj.data.materials.append(mat_shroom_gills)
    cap_obj.parent = stem_obj

build_stylized_lathe_mushroom(4.20, 3.40, scale=0.78, lean_angle=-0.12, name="Right_Dome_Mushroom_1")
build_stylized_lathe_mushroom(3.60, 2.70, scale=0.60, lean_angle=-0.08, name="Right_Dome_Mushroom_2")
build_stylized_lathe_mushroom(4.75, 3.90, scale=0.45, lean_angle=-0.14, name="Right_Dome_Mushroom_3")

TALL_GRASS_CLEARANCE = 0.42
TALL_GRASS_MAX_RELOCATION = 2.4
tall_grass_exclusion_zones = []

def register_tall_grass_exclusion(name, x, y, radius):

    tall_grass_exclusion_zones.append((name, x, y, radius + TALL_GRASS_CLEARANCE))

def resolve_tall_grass_position(x, y):

    def is_clear(cx, cy):
        return all(math.hypot(cx - bx, cy - by) >= radius
                   for _, bx, by, radius in tall_grass_exclusion_zones)

    if is_clear(x, y):
        return x, y

    for ring in range(1, 14):
        distance = ring * 0.18
        if distance > TALL_GRASS_MAX_RELOCATION:
            break
        for spoke in range(16):
            angle = spoke * math.tau / 16.0 + ring * 0.41
            candidate_x = x + math.cos(angle) * distance
            candidate_y = y + math.sin(angle) * distance
            if is_clear(candidate_x, candidate_y):
                return candidate_x, candidate_y

    raise RuntimeError(f"No clear terrain location for tall grass at ({x}, {y})")

register_tall_grass_exclusion("Rustic_Mossy_Bench", 2.90, 1.05, 1.15)
register_tall_grass_exclusion("Right_Dome_Mushroom_1", 4.20, 3.40, 0.82)
register_tall_grass_exclusion("Right_Dome_Mushroom_2", 3.60, 2.70, 0.66)
register_tall_grass_exclusion("Right_Dome_Mushroom_3", 4.75, 3.90, 0.52)

def spawn_tall_grass_clump(x, y, z, scale=1.0, num_blades=14, name="TallGrass"): 
    bm = bmesh.new()
    for ib in range(num_blades):
        ang = (ib / float(num_blades)) * math.pi * 2.0 + (ib * 0.2)
        h = (0.75 + (ib % 3) * 0.18) * scale
        tilt = 0.22 * scale

        v_base_l = bm.verts.new((x + math.cos(ang) * 0.05, y + math.sin(ang) * 0.05, z))
        v_base_r = bm.verts.new((x + math.cos(ang + 0.3) * 0.05, y + math.sin(ang + 0.3) * 0.05, z))
        v_mid_l = bm.verts.new((x + math.cos(ang) * (0.05 + tilt * 0.4), y + math.sin(ang) * (0.05 + tilt * 0.4), z + h * 0.5))
        v_mid_r = bm.verts.new((x + math.cos(ang + 0.2) * (0.05 + tilt * 0.4), y + math.sin(ang + 0.2) * (0.05 + tilt * 0.4), z + h * 0.5))
        v_tip = bm.verts.new((x + math.cos(ang) * (0.05 + tilt), y + math.sin(ang) * (0.05 + tilt), z + h))

        bm.faces.new((v_base_l, v_base_r, v_mid_r, v_mid_l))
        bm.faces.new((v_mid_l, v_mid_r, v_tip))

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_tall_grass)
    spawn_dirt_mound_patch(x, y, z, sx=0.18 * scale, sy=0.15 * scale, seed=int(abs(x) * 10 + abs(y) * 5), name=f"{name}_Mound")
    spawn_river_pebble(x + 0.12 * scale, y + 0.05 * scale, z, 0.08 * scale, 0.06 * scale)
    spawn_river_pebble(x - 0.10 * scale, y - 0.08 * scale, z, 0.06 * scale, 0.05 * scale)
    return obj

tall_grass_coords = [
    (3.8, 2.6), (4.4, 3.5), (4.9, 3.1), (4.6, 4.1),

    (-2.8, -1.0), (-3.1, -0.2), (-2.8, 0.8),
    (2.8, -1.0), (3.1, -0.2), (2.8, 0.8),
]
for idx, (tx, ty) in enumerate(tall_grass_coords):
    tx, ty = resolve_tall_grass_position(tx, ty)
    tz = get_terrain_surface_z(tx, ty)
    spawn_tall_grass_clump(tx, ty, tz, scale=0.85, name=f"Right_TallGrass_{idx}")

output_path = os.path.abspath("assets/models/cavern_kit.glb")
os.makedirs(os.path.dirname(output_path), exist_ok=True)

blend_dir = os.path.abspath("assets/models/source_blend")
os.makedirs(blend_dir, exist_ok=True)
blend_path = os.path.join(blend_dir, "cavern_kit.blend")
bpy.ops.wm.save_as_mainfile(filepath=blend_path)
print(f">>> [BLENDER 4.5] Saved native Blender project to: {blend_path}")

bpy.ops.export_scene.gltf(
    filepath=output_path,
    export_format='GLB',
    use_selection=False,
    export_apply=True,
    export_materials='EXPORT',
    export_attributes=True,
)

print(f">>> [BLENDER 4.5] Successfully exported Anti-Low-Poly Cavern Kit to: {output_path}")
