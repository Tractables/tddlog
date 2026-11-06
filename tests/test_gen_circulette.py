"""scripts/gen_circulette.py: a design converted twice gives the same netlist.

rkt_of_verilog.py at Circulette's commit numbers a design's inputs and
outputs in the order of pyrtl's sets, which follows object ids, so two
conversions of one design could number its ports otherwise.
install_circulette.sh patches it to sort them (docs/baselines.md, the
Circulette section). The test runs where CIRCULETTE_BENCHMARKS names that
patched checkout and CIRCULETTE_PYTHON its Python with pyrtl, yosys on
PATH, and is skipped otherwise.
"""
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import bench  # noqa: E402

# Designs with several inputs and outputs, memories, registers and constants.
DESIGNS = ['ct_mux2', 'ct_ALU_Top', 'ct_FIFO', 'ct_fulladder4', 'ct_mac2', 'ct_openfpga_dual_port_ram_1k',
           'ct_serial-crc', 'ct_jsrflipflop']
RUNS = 3
CIRCULETTE = os.environ.get('CIRCULETTE_BENCHMARKS', '')
PYTHON = os.environ.get('CIRCULETTE_PYTHON', '')


@unittest.skipUnless(CIRCULETTE and PYTHON, 'CIRCULETTE_BENCHMARKS and CIRCULETTE_PYTHON name no Circulette')
class SameBytesTests(unittest.TestCase):
    def test_a_design_converts_to_the_same_bytes_every_time(self):
        script = str(bench.source_tree.script('gen_circulette.py'))
        for name in DESIGNS:
            gargs = bench.CIRCULETTE[name][1]
            with self.subTest(name=name), tempfile.TemporaryDirectory() as d:
                sums = set()
                for k in range(RUNS):
                    # one name in every run: the netlist names its design after the file
                    cql = os.path.join(d, str(k), name + '.cql')
                    os.makedirs(os.path.dirname(cql))
                    subprocess.run([sys.executable, script, *gargs[1:], '--circulette', CIRCULETTE, '--cql', cql,
                                    '--python', PYTHON], check=True, capture_output=True)
                    sums.add(hashlib.sha256(Path(cql).read_bytes()).hexdigest())
                self.assertEqual(len(sums), 1)


if __name__ == '__main__':
    unittest.main()
