"""Projection dispatch tests; synthetic branch positives do not authenticate a live pair."""
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_renderer_qualification_owner as owner
import test_generic_renderer_qualification_owner as source_tests
from generic_gate import GateContractError
from generic_original_harness_projection import OriginalHarnessContext
from test_generic_setup_bridge import unused_wait


class OriginalIntegrationTests(unittest.TestCase):
    def test_exact_request_context_required_before_native_lane(self) -> None:
        _, prepared, _ = source_tests.SourceTest().fixture()
        context = OriginalHarnessContext(Path('/session'), 'a'*64, 'base')
        request = owner.RendererRequest(context.session_path, context.session_sha256, 'base', 'original_off',
                                        prepared, unused_wait(prepared), Path('/out'), context)
        owner.request_context(request)
        wrong = (None, replace(context, arm='candidate'), replace(context, session_path=Path('/other')),
                 replace(context, session_sha256='b'*64))
        with patch.object(owner, 'held_lane') as lane:
            for changed in wrong:
                with self.subTest(context=changed), self.assertRaises(GateContractError):
                    _ = owner.RendererOwner(replace(request, original_harness=changed), -1).run()
            lane.assert_not_called()
        for mode in ('derived_off', 'derived_on'):
            with self.subTest(mode=mode), self.assertRaises(GateContractError):
                owner.request_context(replace(request, mode=mode))

    def test_original_dispatch_keeps_shared_h_checks_without_derived_o_checks(self) -> None:
        session, prepared, _ = source_tests.SourceTest().fixture()
        context = OriginalHarnessContext(Path('/session'), 'a'*64, 'base')
        # Deliberately omit O-only derived arm data: original validation belongs to the dedicated verifier.
        arm = session.field('arms').field('base').mapping()
        _ = arm.pop('effective_source')
        _ = arm.pop('source_inputs')
        with patch.object(OriginalHarnessContext, 'validate_prepared') as projection, \
                patch.object(owner, 'matrix') as matrix, patch.object(owner, 'immutable_inputs') as immutable:
            owner.mode_source(session, 'base', 'original_off', prepared, context)
            projection.assert_called_once_with(session, prepared)
            matrix.assert_not_called()
            immutable.assert_called_once_with(prepared)
            session.field('harness').field('inputs').mapping()['tools/v8/generic_original_harness_projection.py'] = '0'*64
            with self.assertRaisesRegex(GateContractError, 'loaded producer/reader'):
                owner.mode_source(session, 'base', 'original_off', prepared, context)

    def test_real_verifier_rejects_unretained_prepared_pair(self) -> None:
        session, prepared, _ = source_tests.SourceTest().fixture()
        context = OriginalHarnessContext(Path('/session'), 'a'*64, 'base')
        with self.assertRaisesRegex(GateContractError, 'actual retained workspace build pair'):
            owner.mode_source(session, 'base', 'original_off', prepared, context)

    def test_wrong_arm_and_derived_context_never_dispatch_verifier(self) -> None:
        session, prepared, _ = source_tests.SourceTest().fixture()
        context = OriginalHarnessContext(Path('/session'), 'a'*64, 'candidate')
        with patch.object(OriginalHarnessContext, 'validate_prepared') as projection:
            for mode in ('original_off', 'derived_off', 'derived_on'):
                with self.subTest(mode=mode), self.assertRaises(GateContractError):
                    owner.mode_source(session, 'base', mode, prepared, context)
            projection.assert_not_called()


if __name__ == '__main__':
    _ = unittest.main()
