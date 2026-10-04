import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import source_provenance as source


class SourceProvenanceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        for rel in (source.PROOF_PATH, source.MANIFEST_PATH):
            p = self.root / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source.REPO / rel, p)
        shutil.copytree(source.REPO / source.SOURCE_DIR, self.root / source.SOURCE_DIR)
        self.rows = source.load_proof(self.root)
        self.name = next(iter(self.rows))
        self.file = self.root / source.SOURCE_DIR / self.name

    def tearDown(self):
        self.temp.cleanup()

    def rejects(self):
        with self.assertRaises(source.ProvenanceError):
            source.read_approved_source(self.name, self.root)

    def test_all_approved_sources_verify_without_a_git_repository(self):
        self.assertFalse((self.root / '.git').exists())
        self.assertEqual(sum(len(source.read_approved_source(name, self.root)) for name in self.rows), source.EXPECTED_BYTES)

    def test_changed_source_bytes_fail(self):
        self.file.write_bytes(self.file.read_bytes() + b'changed')
        self.rejects()

    def test_missing_source_fails(self):
        self.file.unlink()
        self.rejects()

    def test_unexpected_source_fails(self):
        (self.file.parent / 'unexpected.png').write_bytes(b'other')
        self.rejects()

    def test_unexpected_empty_directory_fails(self):
        (self.file.parent / 'unexpected').mkdir()
        self.rejects()

    def test_unapproved_and_escaping_paths_fail(self):
        for name in ('../secret', '/absolute', 'batch04/concept.png', self.name.replace('/', '\\')):
            with self.assertRaises(source.ProvenanceError):
                source.read_approved_source(name, self.root)

    def test_symlink_file_fails_even_with_correct_bytes(self):
        self.file.unlink()
        self.file.symlink_to(source.REPO / source.SOURCE_DIR / self.name)
        self.rejects()

    def test_symlink_parent_fails(self):
        directory = self.file.parent
        shutil.rmtree(directory)
        directory.symlink_to(source.REPO / source.SOURCE_DIR / directory.name, target_is_directory=True)
        self.rejects()

    def test_proof_tampering_fails(self):
        p = self.root / source.PROOF_PATH
        p.write_bytes(p.read_bytes() + b' ')
        self.rejects()

    def test_manifest_tampering_fails(self):
        p = self.root / source.MANIFEST_PATH
        p.write_bytes(p.read_bytes() + b' ')
        self.rejects()

    def test_root_tree_is_recomputed_independently(self):
        # Exercise the structural verifier separately from the immutable-byte pin.
        p = self.root / source.PROOF_PATH
        proof = json.loads(p.read_bytes())
        proof['trees'][0]['entries'][0]['sha'] = '0' * 40
        data = (json.dumps(proof, ensure_ascii=False, sort_keys=True, indent=2) + '\n').encode()
        self.assertEqual(len(data), 44708)
        p.write_bytes(data)
        with patch.object(source, 'PROOF_SHA256', source.sha256(data)):
            self.rejects()


if __name__ == '__main__':
    unittest.main()
