import io, struct, unittest, zlib
from unittest.mock import patch
from PIL import Image
import build_icon as tool

class NativeIconTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = (tool.ROOT / tool.SOURCE).read_bytes()
        cls.ico, cls.record = tool.render(cls.source)

    def test_original_is_exact_transparent_rgba(self):
        self.assertEqual(tool.sha(self.source), tool.SOURCE_SHA256)
        with Image.open(io.BytesIO(self.source)) as image:
            self.assertEqual((image.format, image.mode, image.size), ("PNG", "RGBA", (1254, 1254)))
            self.assertEqual(image.getchannel("A").getextrema(), (0, 255))
            self.assertGreater(image.getchannel("A").histogram()[0], 800_000)

    def test_conversion_is_byte_identical_twice_and_matches_committed_output(self):
        self.assertEqual(tool.render(self.source), (self.ico, self.record))
        self.assertEqual((tool.ROOT / tool.OUTPUT).read_bytes(), self.ico)

    def test_every_ico_entry_is_bounded_unique_and_matches_exact_resized_rgba(self):
        self.assertEqual(struct.unpack_from("<HHH", self.ico), (0, 1, len(tool.SIZES)))
        original = Image.open(io.BytesIO(self.source)).convert("RGBA")
        cursor = 6 + 16 * len(tool.SIZES)
        for index, (size, expected) in enumerate(zip(tool.SIZES, self.record["frames"], strict=True)):
            w, h, palette, reserved, planes, bpp, length, start = struct.unpack_from("<BBBBHHII", self.ico, 6 + 16 * index)
            self.assertEqual((w or 256, h or 256, palette, reserved, planes, bpp), (size, size, 0, 0, 1, 32))
            self.assertEqual(start, cursor)
            data = self.ico[start:start + length]
            self.assertEqual((len(data), tool.sha(data)), (expected["bytes"], expected["sha256"]))
            with Image.open(io.BytesIO(data)) as frame:
                self.assertEqual(frame.mode, "RGBA")
                self.assertEqual(frame.size, (size, size))
                self.assertEqual(frame.tobytes(), original.resize((size, size), Image.Resampling.LANCZOS).tobytes())
                self.assertEqual(tool.sha(frame.tobytes()), expected["rgbaSha256"])
                self.assertEqual(frame.getchannel("A").getextrema()[0], 0)
            cursor += length
        self.assertEqual(cursor, len(self.ico))

    def test_standard_ico_decoder_sees_all_sizes_and_preserves_each_frame(self):
        with Image.open(io.BytesIO(self.ico)) as icon:
            self.assertEqual(icon.format, "ICO")
            self.assertEqual(icon.info["sizes"], {(size, size) for size in tool.SIZES})
            for frame in self.record["frames"]:
                image = icon.ico.getimage(tuple(frame["size"])).convert("RGBA")
                self.assertEqual(tool.sha(image.tobytes()), frame["rgbaSha256"])

    def test_tauri_first_entry_is_full_resolution_not_a_small_taskbar_frame(self):
        # The pinned Tauri code generator selects ICO entries()[0], whereas
        # Pillow normally opens the largest frame. Test the actual first entry.
        w, h, _, _, _, _, length, offset = struct.unpack_from("<BBBBHHII", self.ico, 6)
        self.assertEqual((w or 256, h or 256), (256, 256))
        with Image.open(io.BytesIO(self.ico[offset:offset + length])) as first:
            expected = Image.open(io.BytesIO(self.source)).convert("RGBA").resize((256, 256), Image.Resampling.LANCZOS)
            self.assertEqual(first.tobytes(), expected.tobytes())

    def test_original_png_chunks_and_c2pa_block_are_retained_in_exact_source(self):
        # Pillow image.info is not a complete metadata inventory. The original
        # includes caBX/C2PA; preserve it and verify structure, not its signature.
        cursor, c2pa = 8, []
        self.assertEqual(self.source[:8], b"\x89PNG\r\n\x1a\n")
        while cursor < len(self.source):
            length = struct.unpack_from(">I", self.source, cursor)[0]
            kind = self.source[cursor + 4:cursor + 8]
            data = self.source[cursor + 8:cursor + 8 + length]
            crc = struct.unpack_from(">I", self.source, cursor + 8 + length)[0]
            self.assertEqual(zlib.crc32(kind + data) & 0xffffffff, crc)
            if kind == b"caBX":
                c2pa.append(data)
            cursor += 12 + length
        self.assertEqual(cursor, len(self.source))
        self.assertEqual([len(data) for data in c2pa], [23654])
        self.assertEqual(tool.sha(self.source), tool.SOURCE_SHA256)

    def test_changed_source_and_unpinned_encoder_fail_closed(self):
        with self.assertRaisesRegex(ValueError, "E_ICON_SOURCE_SHA"):
            tool.render(self.source[:-1] + bytes([self.source[-1] ^ 1]))
        with patch.object(tool, "PILLOW_VERSION", "0.0"):
            with self.assertRaisesRegex(ValueError, "E_ICON_PILLOW_VERSION"):
                tool.render(self.source)

if __name__ == "__main__":
    unittest.main()
