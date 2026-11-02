"""Synthetic Cargo structures only; no compiler/native execution or admission."""
import json
import unittest

from generic_gate import document
from generic_workspace_tests import inspect_inventory
from mux2_gate_inputs import Json, Node


def fixture() -> tuple[Node, tuple[Node,...]]:
    packages: list[Json] = []
    messages: list[Node] = []
    for package,kind,doctest in (('alpha',['lib'],True),('beta',['cdylib','rlib'],False)):
        manifest = '/src/rust/crates/'+package+'/Cargo.toml'
        target: dict[str,Json] = {'name':'same_name','kind':[value for value in kind],'src_path':'/src/rust/crates/'+package+'/src/lib.rs',
                                'test':True,'doctest':doctest}
        packages.append({'id':package,'name':package,'manifest_path':manifest,'targets':[target]})
        binary = '/src/rust/target/debug/deps/'+package+'-hash'
        messages.append(Node({'reason':'compiler-artifact','package_id':package,'manifest_path':manifest,
                              'target':target,'profile':{'test':True},'fresh':False,'executable':binary,'filenames':[binary]}))
    metadata = Node({'workspace_root':'/src/rust','target_directory':'/src/rust/target',
                     'workspace_members':['alpha','beta'],'packages':packages})
    messages.append(Node({'reason':'build-finished','success':True}))
    return metadata,tuple(messages)


def edge_fixture(kind: str, *, selected: bool = True) -> tuple[Node, tuple[Node,...]]:
    metadata,messages = fixture()
    declared = metadata.field('packages').array()[0].field('targets').array()[0].mapping()
    declared['kind'] = [kind]
    declared['test'] = selected
    declared['doctest'] = False
    source = {'bin':'src/bin/tool.rs','example':'examples/demo.rs','bench':'benches/measure.rs',
              'test':'tests/integration.rs','staticlib':'src/lib.rs'}[kind]
    declared['src_path'] = '/src/rust/crates/alpha/'+source
    artifact = messages[0].mapping()
    folder = 'examples' if kind == 'example' else 'deps'
    binary = '/src/rust/target/debug/'+folder+'/alpha-hash'
    artifact['executable'] = binary
    artifact['filenames'] = [binary]
    return metadata,messages


class InventoryTests(unittest.TestCase):
    def test_package_qualified_identity_and_unobserved_worlds(self) -> None:
        metadata,messages = fixture()
        result = inspect_inventory(metadata,messages,4)
        self.assertEqual([row.target.package for row in result.executables],['alpha','beta'])
        self.assertEqual([str(row.cwd) for row in result.executables],['/src/rust/crates/alpha','/src/rust/crates/beta'])
        self.assertEqual(len(result.doctests),1)
        self.assertEqual(result.admission,'INCOMPLETE_SOURCE_BUILD_HANDOFF_REQUIRED')
        self.assertEqual(result.worlds,'INCOMPLETE_UNCLASSIFIED_NATIVE_WORLDS')
        self.assertEqual(result.doctest_execution,'NOT_OBSERVED')
        self.assertEqual([(row.step,row.profile,row.filter,row.ignored,row.threads) for row in result.executables],
                         [(4,'debug',None,'exclude',8)]*2)

    def test_missing_duplicate_foreign_failed_and_profile_reject(self) -> None:
        metadata,messages = fixture()
        for changed in (messages[1:],(messages[0],*messages),messages[:-1],
                        (*messages[:-1],Node({'reason':'build-finished','success':False}))):
            with self.subTest(messages=changed), self.assertRaises(ValueError):
                _ = inspect_inventory(metadata,changed,4)
        with self.assertRaises(ValueError):
            _ = inspect_inventory(metadata,messages,5)
        for field,value in (('fresh',True),('package_id','foreign'),('executable','/tmp/foreign')):
            metadata,messages = fixture()
            messages[0].mapping()[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                _ = inspect_inventory(metadata,messages,4)

    def test_selected_bins_examples_benches_staticlibs_and_integration_tests(self) -> None:
        for kind in ('bin','example','bench','staticlib','test'):
            with self.subTest(kind=kind):
                metadata,messages = edge_fixture(kind)
                result = inspect_inventory(metadata,messages,4)
                self.assertEqual(len(result.executables),2)
                self.assertEqual(result.executables[0].target.kinds,(kind,))
                with self.assertRaises(ValueError):
                    _ = inspect_inventory(metadata,messages[1:],4)

    def test_omitted_selected_bench_cannot_pass(self) -> None:
        metadata,messages = edge_fixture('bench')
        with self.assertRaises(ValueError):
            _ = inspect_inventory(metadata,messages[1:],4)

    def test_disabled_targets_are_not_test_executables(self) -> None:
        for kind in ('bin','example','bench','staticlib','test'):
            with self.subTest(kind=kind):
                metadata,messages = edge_fixture(kind,selected=False)
                messages[0].field('profile').mapping()['test'] = False
                result = inspect_inventory(metadata,messages,4)
                self.assertEqual([row.target.package for row in result.executables],['beta'])
                messages[0].field('profile').mapping()['test'] = True
                with self.assertRaises(ValueError):
                    _ = inspect_inventory(metadata,messages,4)

    def test_required_features_cannot_be_guessed_for_conditional_targets(self) -> None:
        for kind in ('bin','example','bench','test'):
            metadata,messages = edge_fixture(kind)
            metadata.field('packages').array()[0].field('targets').array()[0].mapping()['required-features'] = ['feature-x']
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                _ = inspect_inventory(metadata,messages,4)

    def test_example_and_normal_binary_directories_are_not_interchangeable(self) -> None:
        for kind,folder in (('example','deps'),('bin','examples')):
            metadata,messages = edge_fixture(kind)
            binary = '/src/rust/target/debug/'+folder+'/alpha-hash'
            messages[0].mapping().update(executable=binary,filenames=[binary])
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                _ = inspect_inventory(metadata,messages,4)

    def test_same_package_same_name_different_kind_remains_distinct(self) -> None:
        metadata,messages = fixture()
        package = metadata.field('packages').array()[0]
        declared = document(json.dumps(package.field('targets').array()[0].value)).mapping()
        declared.update(kind=['bin'],src_path='/src/rust/crates/alpha/src/main.rs',doctest=False)
        targets = package.field('targets').value
        assert isinstance(targets,list)
        targets.append(declared)
        artifact = document(json.dumps(messages[0].value)).mapping()
        binary = '/src/rust/target/debug/deps/alpha-bin-hash'
        artifact.update(target=declared,executable=binary,filenames=[binary])
        result = inspect_inventory(metadata,(*messages[:-1],Node(artifact),messages[-1]),4)
        self.assertEqual(len(result.executables),3)
        self.assertEqual(len({(row.target.package_id,row.target.name,row.target.kinds) for row in result.executables}),3)
        artifact['package_id'] = 'beta'
        with self.assertRaises(ValueError):
            _ = inspect_inventory(metadata,(*messages[:-1],Node(artifact),messages[-1]),4)

    def test_release_separate_and_conditional_never_silently_omitted(self) -> None:
        metadata,messages = fixture()
        release = tuple(document(json.dumps(row.value).replace('/debug/','/release/')) for row in messages)
        result = inspect_inventory(metadata,release,5)
        self.assertEqual({row.profile for row in result.executables},{'release'})
        metadata.field('packages').array()[0].field('targets').array()[0].mapping()['required-features'] = ['optional']
        with self.assertRaises(ValueError):
            _ = inspect_inventory(metadata,messages,4)


if __name__ == '__main__':
    _ = unittest.main()
