import bpy
import bmesh
import math
import os
from mathutils import Vector, Matrix, Euler

# Clear all objects in the current scene
for obj in bpy.data.objects:
    bpy.data.objects.remove(obj, do_unlink=True)
for mesh in bpy.data.meshes:
    bpy.data.meshes.remove(mesh)
for mat in bpy.data.materials:
    bpy.data.materials.remove(mat)
for arm in bpy.data.armatures:
    bpy.data.armatures.remove(arm)

def create_mat(name, color, roughness=0.5, metallic=0.0, emission=None):
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    bsdf = nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = color
        bsdf.inputs["Roughness"].default_value = roughness
        bsdf.inputs["Metallic"].default_value = metallic
        if emission:
            bsdf.inputs["Emission Color"].default_value = emission
            bsdf.inputs["Emission Strength"].default_value = 2.0
    return mat

mat_coat = create_mat("M_Coat", (0.35, 0.22, 0.14, 1.0), roughness=0.7)
mat_vest = create_mat("M_Vest", (0.12, 0.22, 0.38, 1.0), roughness=0.6)
mat_pants = create_mat("M_Pants", (0.18, 0.16, 0.15, 1.0), roughness=0.8)
mat_boots = create_mat("M_Boots", (0.15, 0.10, 0.08, 1.0), roughness=0.4)
mat_skin = create_mat("M_Skin", (0.85, 0.65, 0.52, 1.0), roughness=0.5)
mat_leather = create_mat("M_Leather", (0.28, 0.16, 0.09, 1.0), roughness=0.4)
mat_brass = create_mat("M_Brass", (0.85, 0.65, 0.20, 1.0), roughness=0.3, metallic=0.9)
mat_potion_glow = create_mat("M_PotionGlow", (0.05, 0.85, 0.95, 1.0), roughness=0.1, emission=(0.1, 0.9, 1.0, 1.0))
mat_canvas = create_mat("M_Canvas", (0.65, 0.58, 0.45, 1.0), roughness=0.9)
mat_hat = create_mat("M_Hat", (0.28, 0.18, 0.12, 1.0), roughness=0.7)
mat_hat_band = create_mat("M_HatBand", (0.1, 0.08, 0.08, 1.0), roughness=0.5)

# 1. Create Armature
armature_data = bpy.data.armatures.new("Armature_Player")
armature_obj = bpy.data.objects.new("Armature_Player", armature_data)
bpy.context.collection.objects.link(armature_obj)
bpy.context.view_layer.objects.active = armature_obj
armature_obj.select_set(True)

bpy.ops.object.mode_set(mode='EDIT')
eb = armature_data.edit_bones

root = eb.new("Root")
root.head = (0, 0, 0)
root.tail = (0, 0, 0.9)

spine = eb.new("Spine")
spine.head = (0, 0, 0.9)
spine.tail = (0, 0, 1.25)
spine.parent = root

chest = eb.new("Chest")
chest.head = (0, 0, 1.25)
chest.tail = (0, 0, 1.55)
chest.parent = spine

neck = eb.new("Neck")
neck.head = (0, 0, 1.55)
neck.tail = (0, 0, 1.65)
neck.parent = chest

head_bone = eb.new("Head")
head_bone.head = (0, 0, 1.65)
head_bone.tail = (0, 0, 1.95)
head_bone.parent = neck

# Arms
shoulder_l = eb.new("UpperArm.L")
shoulder_l.head = (0.22, 0, 1.50)
shoulder_l.tail = (0.48, 0, 1.22)
shoulder_l.parent = chest

forearm_l = eb.new("Forearm.L")
forearm_l.head = (0.48, 0, 1.22)
forearm_l.tail = (0.68, 0, 0.95)
forearm_l.parent = shoulder_l

hand_l = eb.new("Hand.L")
hand_l.head = (0.68, 0, 0.95)
hand_l.tail = (0.75, 0, 0.85)
hand_l.parent = forearm_l

shoulder_r = eb.new("UpperArm.R")
shoulder_r.head = (-0.22, 0, 1.50)
shoulder_r.tail = (-0.48, 0, 1.22)
shoulder_r.parent = chest

forearm_r = eb.new("Forearm.R")
forearm_r.head = (-0.48, 0, 1.22)
forearm_r.tail = (-0.68, 0, 0.95)
forearm_r.parent = shoulder_r

hand_r = eb.new("Hand.R")
hand_r.head = (-0.68, 0, 0.95)
hand_r.tail = (-0.75, 0, 0.85)
hand_r.parent = forearm_r

# Legs
thigh_l = eb.new("Thigh.L")
thigh_l.head = (0.15, 0, 0.9)
thigh_l.tail = (0.16, 0, 0.48)
thigh_l.parent = root

shin_l = eb.new("Shin.L")
shin_l.head = (0.16, 0, 0.48)
shin_l.tail = (0.16, 0, 0.1)
shin_l.parent = thigh_l

foot_l = eb.new("Foot.L")
foot_l.head = (0.16, 0, 0.1)
foot_l.tail = (0.16, 0.18, 0.0)
foot_l.parent = shin_l

thigh_r = eb.new("Thigh.R")
thigh_r.head = (-0.15, 0, 0.9)
thigh_r.tail = (-0.16, 0, 0.48)
thigh_r.parent = root

shin_r = eb.new("Shin.R")
shin_r.head = (-0.16, 0, 0.48)
shin_r.tail = (-0.16, 0, 0.1)
shin_r.parent = thigh_r

foot_r = eb.new("Foot.R")
foot_r.head = (-0.16, 0, 0.1)
foot_r.tail = (-0.16, 0.18, 0.0)
foot_r.parent = shin_r

bpy.ops.object.mode_set(mode='OBJECT')

def make_box(name, size, loc, mat, parent=None, bone_name=None):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = size
    bpy.ops.object.transform_apply(scale=True)
    obj.data.materials.append(mat)
    if parent:
        obj.parent = parent
        if bone_name:
            obj.parent_type = 'BONE'
            obj.parent_bone = bone_name
    return obj

def make_cylinder(name, radius, depth, loc, rot, mat, parent=None, bone_name=None):
    bpy.ops.mesh.primitive_cylinder_add(radius=radius, depth=depth, location=loc, rotation=rot)
    obj = bpy.context.active_object
    obj.name = name
    obj.data.materials.append(mat)
    if parent:
        obj.parent = parent
        if bone_name:
            obj.parent_type = 'BONE'
            obj.parent_bone = bone_name
    return obj

# 2. Body Meshes
torso = make_box("Player_Torso", (0.42, 0.28, 0.65), (0, 0, 1.25), mat_coat, armature_obj, "Chest")
vest = make_box("Player_Vest", (0.38, 0.26, 0.50), (0, 0.02, 1.25), mat_vest, armature_obj, "Chest")
belt = make_box("Player_Belt", (0.44, 0.30, 0.10), (0, 0, 0.92), mat_leather, armature_obj, "Spine")
buckle = make_box("Player_Buckle", (0.10, 0.04, 0.08), (0, 0.16, 0.92), mat_brass, armature_obj, "Spine")
coat_skirt = make_box("Player_CoatTail", (0.48, 0.34, 0.55), (0, -0.02, 0.65), mat_coat, armature_obj, "Spine")

vial1 = make_cylinder("Belt_Potion_Cyan", 0.035, 0.12, (0.24, 0.05, 0.92), (0, 0.2, 0), mat_potion_glow, armature_obj, "Spine")
vial2 = make_cylinder("Belt_Potion_Amber", 0.035, 0.12, (-0.24, 0.05, 0.92), (0, -0.2, 0), mat_brass, armature_obj, "Spine")
pouch = make_box("Belt_Pouch", (0.12, 0.08, 0.12), (0.18, -0.15, 0.92), mat_leather, armature_obj, "Spine")

backpack = make_box("Player_Backpack", (0.36, 0.24, 0.45), (0, -0.25, 1.35), mat_canvas, armature_obj, "Chest")
bedroll = make_cylinder("Backpack_Bedroll", 0.08, 0.42, (0, -0.25, 1.62), (0, math.radians(90), 0), mat_leather, armature_obj, "Chest")
pack_strap_l = make_box("PackStrap_L", (0.05, 0.26, 0.40), (0.15, -0.05, 1.35), mat_leather, armature_obj, "Chest")
pack_strap_r = make_box("PackStrap_R", (0.05, 0.26, 0.40), (-0.15, -0.05, 1.35), mat_leather, armature_obj, "Chest")

arm_up_l = make_cylinder("ArmUp_L", 0.075, 0.35, (0.35, 0, 1.35), (0, math.radians(35), 0), mat_coat, armature_obj, "UpperArm.L")
arm_low_l = make_cylinder("ArmLow_L", 0.065, 0.32, (0.58, 0, 1.08), (0, math.radians(35), 0), mat_coat, armature_obj, "Forearm.L")
hand_mesh_l = make_box("Hand_L", (0.08, 0.10, 0.12), (0.72, 0, 0.90), mat_skin, armature_obj, "Hand.L")

arm_up_r = make_cylinder("ArmUp_R", 0.075, 0.35, (-0.35, 0, 1.35), (0, math.radians(-35), 0), mat_coat, armature_obj, "UpperArm.R")
arm_low_r = make_cylinder("ArmLow_R", 0.065, 0.32, (-0.58, 0, 1.08), (0, math.radians(-35), 0), mat_coat, armature_obj, "Forearm.R")
hand_mesh_r = make_box("Hand_R", (0.08, 0.10, 0.12), (-0.72, 0, 0.90), mat_skin, armature_obj, "Hand.R")

leg_up_l = make_cylinder("LegUp_L", 0.09, 0.45, (0.16, 0, 0.68), (0, 0, 0), mat_pants, armature_obj, "Thigh.L")
leg_low_l = make_cylinder("LegLow_L", 0.08, 0.40, (0.16, 0, 0.30), (0, 0, 0), mat_pants, armature_obj, "Shin.L")
boot_l = make_box("Boot_L", (0.12, 0.24, 0.18), (0.16, 0.04, 0.09), mat_boots, armature_obj, "Foot.L")

leg_up_r = make_cylinder("LegUp_R", 0.09, 0.45, (-0.16, 0, 0.68), (0, 0, 0), mat_pants, armature_obj, "Thigh.R")
leg_low_r = make_cylinder("LegLow_R", 0.08, 0.40, (-0.16, 0, 0.30), (0, 0, 0), mat_pants, armature_obj, "Shin.R")
boot_r = make_box("Boot_R", (0.12, 0.24, 0.18), (-0.16, 0.04, 0.09), mat_boots, armature_obj, "Foot.R")

# 3. Head & Fedora Hat (Child node tagged "Head_Hat" for RenderLayer 1)
bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=12, radius=0.14, location=(0, 0, 1.76))
head_face = bpy.context.active_object
head_face.name = "Head_Face"
head_face.data.materials.append(mat_skin)

bpy.ops.mesh.primitive_cylinder_add(radius=0.32, depth=0.02, location=(0, 0.02, 1.83), rotation=(math.radians(-5), 0, 0))
hat_brim = bpy.context.active_object
hat_brim.data.materials.append(mat_hat)

bpy.ops.mesh.primitive_cylinder_add(radius=0.18, depth=0.15, location=(0, 0.02, 1.90), rotation=(math.radians(-5), 0, 0))
hat_crown = bpy.context.active_object
hat_crown.data.materials.append(mat_hat)

bpy.ops.mesh.primitive_cylinder_add(radius=0.185, depth=0.03, location=(0, 0.02, 1.85), rotation=(math.radians(-5), 0, 0))
hat_band = bpy.context.active_object
hat_band.data.materials.append(mat_hat_band)

bpy.ops.mesh.primitive_torus_add(major_radius=0.05, minor_radius=0.015, location=(0.07, 0.16, 1.84), rotation=(math.radians(70), 0, 0))
goggle_l = bpy.context.active_object
goggle_l.data.materials.append(mat_brass)

bpy.ops.mesh.primitive_torus_add(major_radius=0.05, minor_radius=0.015, location=(-0.07, 0.16, 1.84), rotation=(math.radians(70), 0, 0))
goggle_r = bpy.context.active_object
goggle_r.data.materials.append(mat_brass)

# Join head & hat parts into "Head_Hat"
bpy.ops.object.select_all(action='DESELECT')
head_face.select_set(True)
hat_brim.select_set(True)
hat_crown.select_set(True)
hat_band.select_set(True)
goggle_l.select_set(True)
goggle_r.select_set(True)
bpy.context.view_layer.objects.active = head_face
bpy.ops.object.join()
head_hat = bpy.context.active_object
head_hat.name = "Head_Hat"
head_hat.parent = armature_obj
head_hat.parent_type = 'BONE'
head_hat.parent_bone = "Head"

# Smooth shading
for obj in bpy.data.objects:
    if obj.type == 'MESH':
        for poly in obj.data.polygons:
            poly.use_smooth = True

target_path = r"C:\Users\fhgam\Documents\GitHub\gamelab\assets\models\player.glb"
os.makedirs(os.path.dirname(target_path), exist_ok=True)

bpy.ops.export_scene.gltf(
    filepath=target_path,
    export_format='GLB',
    use_selection=False,
    export_apply=False,
    export_yup=True
)

print("SUCCESS: Player model exported to", target_path)
