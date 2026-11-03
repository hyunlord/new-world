# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: imported by original play owner for source-owned shell bookkeeping.
"""Validate original Bash bookkeeping while retaining full native environments."""
import sys
from pathlib import Path

from generic_gate import require
from mux2_gate_inputs import Node
from mux2_gate_lock import FD_ENV

import gate_play_protocol as protocol


def validate_environment(observed: Node, expected: dict[str,str], cwd: Path,
                         root: Path, play_root: Path) -> None:
    actual={key:Node(value).text() for key,value in observed.mapping().items()}
    wanted=dict(expected)
    for key in (protocol.REQUEST,protocol.CONTROL,protocol.EVIDENCE,FD_ENV):
        require(key in actual,'/environment','missing live owner environment')
        _=actual.pop(key)
    # Only values written by the exact nested Bash/Python launcher are projected.
    require(actual.get('PWD')==str(cwd),'/environment/PWD','shell cwd differs from original requested cwd')
    previous=actual.get('OLDPWD')
    allowed={str(root),*(str(play_root/version) for version in ('main','m1','v7'))}
    if 'OLDPWD' in wanted:allowed.add(wanted['OLDPWD'])
    require(previous is None or previous in allowed,'/environment/OLDPWD','foreign previous working directory')
    level=actual.get('SHLVL','')
    require(level.isdecimal() and 1<=int(level)<=4,'/environment/SHLVL','unexpected original shell nesting')
    invoked=actual.get('_','')
    require(invoked=='python3' or (Path(invoked).is_absolute() and Path(invoked).resolve()==Path(sys.executable).resolve()),
            '/environment/_','foreign helper interpreter')
    for key in ('PWD','OLDPWD','SHLVL','_'):
        _=actual.pop(key,None);_=wanted.pop(key,None)
    # Python's documented C-locale coercion is limited to this exact value.
    if 'LC_CTYPE' not in wanted and actual.get('LC_CTYPE')=='UTF-8' and wanted.get('LC_ALL') in {None,'C'}:
        _=actual.pop('LC_CTYPE')
    require(actual==wanted,'/environment','original semantic child environment differs')
