import struct
import sys

import bpy
from bpy import data as D
from bpy import context as C
from mathutils import *
from math import *


def convert(objects):
    polygons = []

    for obj in objects:
        if obj.type != 'MESH':
            continue

        mat = obj.matrix_world
        mesh = obj.to_mesh()

        for poly in mesh.polygons:
            print("Polygon index: %d, length: %d" %
                  (poly.index, poly.loop_total))
            vertices = []

            for loop_index in poly.loop_indices:
                v = mat @ mesh.vertices[mesh.loops[loop_index].vertex_index].co
                vertices.append(v)

            polygons.append(vertices)

    return polygons


def export(polygons, out):
    # bincode uses 64 bit values for vector length
    out.write(struct.pack('<II', len(polygons), 0))
    for polygon in polygons:
        vertices = polygon
        out.write(struct.pack('<II', len(vertices), 0))
        for v in vertices:
            out.write(struct.pack('ff', v[0], v[2]))


input_filename = D.filepath
out_filename = D.filepath.replace(
    'source-assets', 'gen-resources').replace('.blend', '.dat')

polygons = convert(C.scene.objects)

with open(out_filename, 'wb') as out:
    print('Writing', out_filename)
    export(polygons, out)
