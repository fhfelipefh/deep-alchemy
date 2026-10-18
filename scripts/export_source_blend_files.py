import bpy
import os

os.makedirs("assets/models/source_blend", exist_ok=True)
models = [
    "dungeon_kit.glb",
    "faceted_rocks_kit.glb",
    "alchemy_props.glb",
    "player.glb",
]

for model in models:
    in_glb = os.path.abspath(os.path.join("assets/models", model))
    if not os.path.exists(in_glb):
        print(f"Skipping {model}: not found.")
        continue
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=in_glb)
    blend_name = os.path.splitext(model)[0] + ".blend"
    out_blend = os.path.abspath(os.path.join("assets/models/source_blend", blend_name))
    bpy.ops.wm.save_as_mainfile(filepath=out_blend)
    print(f">>> [BLENDER] Exported native project: {out_blend}")
