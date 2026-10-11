"""Pass 5 capture schema requires every inherited and new cold frame."""
import unittest

from measure_p5 import long_capture_set


class CaptureSetTests(unittest.TestCase):
    def test_exact_thirteen_frames(self) -> None:
        stems = [f'l_d{day}_1200_F2' for day in (10, 18, 24, 32, 40)] + [
            'l_d24_1200_F1', 'l_d24_2300_F2N', 'l_d10_1500_F2', 'l_d10_1500_F5']
        self.assertEqual(long_capture_set(stems).status, 'FAIL')
        stems += [f'l_d{day}_0200_F2N' for day in (23, 25, 27, 29)]
        self.assertEqual(long_capture_set(stems).status, 'PASS')
        self.assertEqual(long_capture_set(stems[:-1]).status, 'FAIL')
        self.assertEqual(long_capture_set(stems + [stems[0]]).status, 'FAIL')
        self.assertEqual(long_capture_set(stems[:-1] + ['unexpected']).status, 'FAIL')


if __name__ == '__main__':
    unittest.main()
