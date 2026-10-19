import bpy
import os

def export_subset(source_glb, object_names, dest_glb):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    in_path = os.path.abspath(os.path.join("assets/models", source_glb))
    bpy.ops.import_scene.gltf(filepath=in_path)
    
    for obj in bpy.data.objects:
        obj.select_set(False)
        
    found_any = False
    for obj in bpy.data.objects:
        for name in object_names:
            if name in obj.name:
                obj.select_set(True)
                found_any = True
                break
                
    if not found_any:
        for obj in bpy.data.objects:
            obj.select_set(True)
            
    out_path = os.path.abspath(os.path.join("assets/models", dest_glb))
    bpy.ops.export_scene.gltf(
        filepath=out_path,
        use_selection=True,
        export_format='GLB'
    )
    print("Exported", dest_glb)

export_subset("alchemy_props.glb", ["Bench", "Seat", "Backrest", "Leg", "Stretcher", "Vine", "Rustic"], "prop_wooden_bench.glb")
export_subset("cavern_kit.glb", ["Shroom", "Mushroom", "Cap", "Stem"], "prop_mushroom_cyan.glb")
export_subset("cavern_kit.glb", ["Cliff", "Terrace", "Tower", "Wall"], "env_cave_wall.glb")
export_subset("faceted_rocks_kit.glb", ["Rock_Angular", "Boulder", "Shelf"], "prop_faceted_rock.glb")
export_subset("dungeon_kit.glb", ["Tile_Corridor_Straight"], "tile_corridor_straight.glb")
export_subset("dungeon_kit.glb", ["Tile_Corridor_Corner"], "tile_corridor_corner.glb")
export_subset("dungeon_kit.glb", ["Tile_Solid_Pillar"], "tile_solid_pillar.glb")
export_subset("dungeon_kit.glb", ["Shrine", "Crucible"], "macro_alchemical_shrine.glb")
export_subset("dungeon_kit.glb", ["Crystal"], "macro_crystal_cluster.glb")
