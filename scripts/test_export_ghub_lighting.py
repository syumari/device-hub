import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('lighting_export', Path(__file__).with_name('export-ghub-lighting.py'))
exporter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(exporter)


class LightingPreservationTest(unittest.TestCase):
    def test_keeps_frames_and_references_without_unrelated_settings(self):
        effect = {'keyframeInfo': {'keyframeMap': {'0': {'length': 5, 'curveType': 'INCIRC'}}}}
        data = {'account': {'token': 'must-not-export'}, 'applications': ['private-app'],
                'lighting_effects/effect': effect, 'lighting_prefabs': {'list': [{'id': 'effect'}]},
                'cards': {'cards': [{'id': 'light', 'softwareLightingSettings': {'effects': [{'id': 'effect'}]}},
                                    {'id': 'buttons', 'buttonSettings': {}}]},
                'profiles': {'profiles': [{'id': 'profile', 'lightingCard': 'light', 'name': 'private-name',
                                           'applicationId': 'private-app',
                                           'assignments': [{'cardId': 'light', 'slotId': 'lighting'},
                                                           {'cardId': 'buttons', 'slotId': 'button'}]}]}}
        result = exporter.extract(data)
        self.assertEqual(result['effects']['effect'], effect)
        self.assertEqual(result['profiles'][0]['assignments'], [{'cardId': 'light', 'slotId': 'lighting'}])
        self.assertEqual(len(result['cards']), 1)
        self.assertFalse(result['playbackVerified'])
        serialized = str(result)
        for private in ('must-not-export', 'private-app', 'private-name', 'buttonSettings'):
            self.assertNotIn(private, serialized)


if __name__ == '__main__':
    unittest.main()
