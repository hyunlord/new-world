"""Pure synthetic reader controls; no native executable or Cargo invocation."""
import json
import tempfile
import unittest
from pathlib import Path

from generic_gate import document
from generic_process import file_hash
from generic_whole_gate_qualification_build import authenticate
from generic_whole_gate_renderer_builds import authenticate_builds
from mux2_gate_inputs import Json, Node
from renderer_build_test_fixture import complete_receipt, make_fixture


class BridgeBuildTests(unittest.TestCase):
    def test_positive_bridge_parser_shape(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            f = make_fixture(Path(folder).resolve())
            authenticate(f.session, f.build, f.process, 'sim_bridge', 'derived_off')

    def test_thirteen_negative_bridge_shapes(self) -> None:
        cases = ('recipe', 'closure', 'permissions', 'source', 'config', 'artifact_missing',
                 'artifact_source', 'fresh', 'profile', 'kind', 'package', 'executable', 'terminal')
        for case in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory() as folder:
                f = make_fixture(Path(folder).resolve())
                artifact = Node(f.messages[0])
                if case == 'recipe':
                    f.build.mapping()['argv'] = ['not-cargo']
                elif case == 'closure':
                    f.build.mapping()['effective_inputs_before'] = {}
                elif case == 'permissions':
                    f.binary.chmod(0o644)
                elif case == 'source':
                    f.build.mapping()['source_sha'] = '0'*40
                elif case == 'config':
                    _ = (f.cargo_home/'config.toml').write_text('[net]\noffline=true\n')
                elif case == 'artifact_missing':
                    _ = f.messages.pop(0)
                elif case == 'artifact_source':
                    artifact.field('target').mapping()['src_path'] = 'foreign.rs'
                elif case == 'fresh':
                    artifact.mapping()['fresh'] = True
                elif case == 'profile':
                    f.build.mapping()['profile'] = 'release'
                elif case == 'kind':
                    artifact.field('target').mapping()['kind'] = ['bin']
                elif case == 'package':
                    artifact.mapping()['package_id'] = 'foreign-package'
                elif case == 'executable':
                    artifact.mapping()['executable'] = str(f.binary)
                elif case == 'terminal':
                    Node(f.messages[-1]).mapping()['success'] = False
                f.logs()
                with self.assertRaises(ValueError):
                    authenticate(f.session, f.build, f.process, 'sim_bridge', 'derived_off')

    def test_complete_build_chain_join(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            f = make_fixture(Path(folder).resolve())
            probe = complete_receipt(f, 'worldsim_probe')
            bridge = complete_receipt(f, 'sim_bridge')
            selector = Path(f.session.field('selector_path').text())
            selected = document(selector.read_text())
            tool = selected.field('selectors').field('cargo')
            selected.mapping()['environment'] = {'GODOT_BIN': tool.field('path').value}
            selected.mapping()['external_inputs'] = {tool.field('path').text(): tool.field('sha256').value}
            _ = selector.write_text(json.dumps(selected.value))
            f.session.mapping()['selector_sha256'] = file_hash(selector)
            f.session.mapping()['environment'] = selected.field('environment').value
            inputs = f.build.field('effective_inputs_before').mapping()
            tracked: dict[str, Json] = {name: ['100644', digest, digest] for name, digest in inputs.items()}
            arm = Node({'source_sha': f.build.field('source_sha').value, 'original_source': tracked,
                'effective_source': {'manifest': {name: ['100644', digest] for name, digest in inputs.items()}}})
            evidence = Node({'probe': probe.value, 'bridge': bridge.value})
            joined = authenticate_builds(f.session, arm, 'derived_off', evidence)
            self.assertEqual(joined.bridge, f.binary)
            self.assertEqual(joined.probe.name, 'worldsim_probe')
            self.assertEqual(joined.root, Path(f.build.field('root').text()))
            arm.mapping()['source_sha'] = '0'*40
            with self.assertRaises(ValueError):
                _ = authenticate_builds(f.session, arm, 'derived_off', evidence)


if __name__ == '__main__':
    _ = unittest.main()
