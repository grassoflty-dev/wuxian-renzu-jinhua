"""Brotli decoding for read-only preview.5 checks; never installs dependencies.

Prefer Google's Brotli Python package on every platform. Preserve the existing
system-library path on Linux/macOS, without probing arbitrary DLLs on Windows.
The caller still verifies compressed/decoded hashes and the complete bundle.
"""
import ctypes
import ctypes.util
import importlib
import sys


def _python_decoder(module):
    def decode(raw, expected_size):
        # Request small output buffers (the C extension may round upward),
        # rather than allocating the whole expansion before checking it.
        # Retain at most the declared payload size; require an ended stream.
        decoder = module.Decompressor()
        parts = []
        total = 0
        pending = raw
        while True:
            try:
                chunk = decoder.process(
                    pending, output_buffer_limit=min(65536, expected_size - total + 1))
            except Exception as exc:
                raise ValueError('invalid Brotli payload (Brotli 1.2.0 required)') from exc
            total += len(chunk)
            if total > expected_size:
                raise ValueError('Brotli decoded size mismatch')
            parts.append(chunk)
            if decoder.is_finished():
                if total != expected_size:
                    raise ValueError('Brotli decoded size mismatch')
                return b''.join(parts)
            if not chunk and decoder.can_accept_more_data():
                # All compressed input was supplied. An empty drain that
                # needs input means the stream is truncated.
                raise ValueError('incomplete Brotli payload')
            pending = b''
    return decode


def _system_decoder():
    library = ctypes.util.find_library('brotlidec')
    if not library:
        raise OSError('system Brotli decoder not found')
    codec = ctypes.CDLL(library)
    decoder = codec.BrotliDecoderDecompress
    decoder.argtypes = [ctypes.c_size_t, ctypes.c_void_p,
                        ctypes.POINTER(ctypes.c_size_t), ctypes.c_void_p]
    decoder.restype = ctypes.c_int

    def decode(raw, expected_size):
        dst = ctypes.create_string_buffer(expected_size)
        n = ctypes.c_size_t(expected_size)
        src = ctypes.create_string_buffer(raw)
        if decoder(len(raw), src, ctypes.byref(n), dst) != 1 or n.value != expected_size:
            raise ValueError('invalid Brotli payload or decoded size mismatch')
        return dst.raw
    return decode


def get_decoder():
    """Return one decoder; failures decoding data never select a weaker backend."""
    try:
        module = importlib.import_module('brotli')
    except ImportError as exc:
        missing = exc
    else:
        return _python_decoder(module)
    if sys.platform != 'win32':
        try:
            return _system_decoder()
        except (OSError, AttributeError) as exc:
            missing = exc
    raise RuntimeError(
        'Brotli decoder unavailable. Install the official Brotli==1.2.0 wheel '
        'from https://pypi.org/project/Brotli/ using the SAME Python interpreter: '
        'python -m pip install --only-binary=:all: --index-url https://pypi.org/simple Brotli==1.2.0. '
        'No EXE was executed; no verification checks were skipped.'
    ) from missing
