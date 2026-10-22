# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by python -m art.pipeline5.prod1_process --review-retry
"""Exchange complete accepted directories without exposing a partial replacement."""
import ctypes
import os
import sys
from pathlib import Path


def exchange(stage: Path, target: Path) -> None:
    """Use the host's atomic directory-exchange primitive or leave both untouched."""
    library = ctypes.CDLL(None, use_errno=True)
    result: int
    # Host platform is open-ended input; unsupported values are reported explicitly.
    match sys.platform:  # noqa: MATCH_OK
        case 'darwin':
            swap = library.renameatx_np
            swap.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint]
            swap.restype = ctypes.c_int
            result = swap(-2, os.fsencode(stage), -2, os.fsencode(target), 2)
        case 'linux':
            swap = library.renameat2
            swap.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint]
            swap.restype = ctypes.c_int
            result = swap(-100, os.fsencode(stage), -100, os.fsencode(target), 2)
        case _:
            message = 'atomic review replacement requires macOS or Linux directory exchange'
            raise OSError(message)
    if result != 0:
        code = ctypes.get_errno()
        raise OSError(code, os.strerror(code), str(target))
