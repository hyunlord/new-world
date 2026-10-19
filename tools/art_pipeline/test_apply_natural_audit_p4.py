"""Behavioral checks for rejecting audited installed actors."""
import unittest
from PIL import Image
import apply_natural_audit_p4 as audit
from install_v4 import PlannedSprite
from dataclasses import replace
from hair_registry_p4 import HairRegistry, ResolvedHair


def actor(name, status='accepted', activity='idle', kind=''):
    image = Image.new('RGBA', (3, 5), (30, 20, 10, 255))
    return PlannedSprite(name, 'adult_a', 'S', activity, 0, status, 'generated:test', '', False,
                         kind, 'U1_04' if kind == 'idle_icon' else '', image, image)


class NaturalAuditTests(unittest.TestCase):
    def test_rejected_accepted_frame_cannot_be_fallback_source(self):
        sprites = (actor('V_a1_01'), actor('V_a1_02'))
        reviews = {'V_a1_01': 'reject', 'V_a1_02': 'ok'}
        self.assertEqual([s.slot_id for s in audit.safe_sources(sprites, reviews)], ['V_a1_02'])

    def test_rejected_existing_idle_icon_stays_explicit_fallback(self):
        old = actor('V_a2_13', 'fallback', 'build', 'idle_icon')
        result = audit.replacement(old, (), frozenset({'U1_04'}))
        self.assertEqual(result, old)
        self.assertEqual(result.qa_status, 'fallback')

    def test_rejected_real_frame_without_safe_source_is_blocked(self):
        self.assertIsNone(audit.replacement(actor('V_a2_13', activity='build'), (), frozenset({'U1_04'})))

    def test_fresh_ok_procedural_is_not_promoted_to_source(self):
        self.assertEqual(audit.safe_sources((actor('V_a1_01', 'procedural'),), {'V_a1_01': 'ok'}), ())

    def test_identical_replacement_copies_approved_source_mask(self):
        source = actor('V_a1_01')
        target = replace(actor('V_a2_13', 'fallback', 'build'), source_sprite=source.slot_id)
        mask = Image.new('RGBA', source.sprite.size, (3, 1, 0, 255))
        key = (source.body, source.activity, audit.pixel_digest(source.sprite.tobytes()))
        hair = HairRegistry({key: ResolvedHair(mask, 'installed_map', 'source.png', 'hash', 'ok')}, {})
        resolved = audit.replacement_mask(target, (source,), hair)
        self.assertEqual(resolved.image.tobytes(), mask.tobytes())
        self.assertEqual(resolved.path, 'source.png')

    def test_asleep_target_does_not_copy_standing_hair(self):
        source = actor('V_a1_01')
        target = replace(actor('V_a3_13', 'fallback', 'asleep'), source_sprite=source.slot_id)
        mask = Image.new('RGBA', source.sprite.size, (3, 1, 0, 255))
        key = (source.body, source.activity, audit.pixel_digest(source.sprite.tobytes()))
        hair = HairRegistry({key: ResolvedHair(mask, 'installed_map', 'source.png', 'hash', 'ok')}, {})
        self.assertEqual(audit.replacement_mask(target, (source,), hair).image.getchannel('R').getextrema(), (0, 0))


if __name__ == '__main__':
    unittest.main()
