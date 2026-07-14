#ifndef CIVIS_FFI_H
#define CIVIS_FFI_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdint.h>

#define CHUNK_EDGE 16

#define CHUNK_VOXELS (uintptr_t)((CHUNK_EDGE * CHUNK_EDGE) * CHUNK_EDGE)

uint32_t civis_version(void);

#endif  /* CIVIS_FFI_H */
