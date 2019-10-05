import struct
import sys

import bpy
from bpy import data as D
from bpy import context as C
from mathutils import *
from math import *


def export(objects, out):
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
                vertices.append(
                    mesh.vertices[mesh.loops[loop_index].vertex_index])

            # Write number of floating point values
            # bincode uses 64 bit values for vector length
            out.write(struct.pack('<II', len(vertices), 0))

            for v in vertices:
                v = mat @ v.co
                out.write(struct.pack('ff', v[0], v[2]))


input_filename = D.filepath
out_filename = D.filepath.replace(
    'source-assets', 'gen-resources').replace('.blend', '.dat')

with open(out_filename, 'wb') as out:
    print('Writing', out_filename)
    export(C.scene.objects, out)
